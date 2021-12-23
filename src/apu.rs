use crate::constants::*;
use crate::emulator::GameBoyEmulator;
use crate::emulator::RequestSource;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::AudioSubsystem;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

const SOURCE: RequestSource = RequestSource::APU;

struct SingleWriteLock<T: std::cmp::PartialEq + std::marker::Copy> {
    rw: Arc<RwLock<T>>,
    holding: T,
}

impl<T: std::cmp::PartialEq + std::marker::Copy> SingleWriteLock<T> {
    fn new(rw: Arc<RwLock<T>>, initial: T) -> SingleWriteLock<T> {
        let holding = initial;
        SingleWriteLock { rw, holding }
    }
    fn set(&mut self, val: T) {
        if val != self.holding {
            *self.rw.write().unwrap() = val;
        }
    }
}

struct Channel1 {
    frequency: Arc<AtomicU32>,
    phase: f32,
    enable: Arc<AtomicBool>,
    volume: Arc<AtomicU8>,
    duty: Arc<Mutex<f32>>,
    so1_enable: Arc<AtomicU8>,
    so1_value: Arc<RwLock<f32>>,
    so2_enable: Arc<AtomicU8>,
    so2_value: Arc<RwLock<f32>>,
    all_sound_enable: Arc<AtomicBool>,
}

impl AudioCallback for Channel1 {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        let frequency = self.frequency.load(Ordering::Relaxed);
        let duty = *self.duty.lock().unwrap();
        let mut right = true;
        let so1_mod =
            self.so1_enable.load(Ordering::Relaxed) as f32 * (*self.so1_value.read().unwrap());
        let so2_mod =
            self.so2_enable.load(Ordering::Relaxed) as f32 * (*self.so2_value.read().unwrap());
        let mut so_mod = so1_mod;
        let enable =
            self.enable.load(Ordering::Relaxed) & self.all_sound_enable.load(Ordering::Relaxed);
        for x in out.iter_mut() {
            let vol = self.volume.load(Ordering::Relaxed);
            if !enable || self.phase < duty {
                *x = 0.0;
            } else {
                *x = (vol as f32 * so_mod) / 100.0;
            }
            right = !right;
            if right {
                self.phase = (self.phase
                    + (131072.0 / ((MAX_FREQ_VAL - frequency) as f32)) / SAMPLES_PER_SECOND as f32)
                    % 1.0;
                so_mod = so1_mod;
            } else {
                so_mod = so2_mod;
            }
        }
    }
}

type Channel2 = Channel1;

struct Channel3 {
    output_level: Arc<AtomicU8>,
    pointer: Arc<AtomicUsize>,
    wave_ram: Arc<Mutex<[u8; 16]>>,
    phase: f32,
    enable: Arc<AtomicBool>,
    frequency: Arc<AtomicU32>,
    so1_enable: Arc<AtomicU8>,
    so1_value: Arc<RwLock<f32>>,
    so2_enable: Arc<AtomicU8>,
    so2_value: Arc<RwLock<f32>>,
    all_sound_enable: Arc<AtomicBool>,
}

impl AudioCallback for Channel3 {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        let output_shift = VOLUME_SHIFT_CONVERSION
            [((self.output_level.load(Ordering::Relaxed) >> 5) & 0b11) as usize];
        let mut right = true;
        let so1_mod =
            self.so1_enable.load(Ordering::Relaxed) as f32 * (*self.so1_value.read().unwrap());
        let so2_mod =
            self.so2_enable.load(Ordering::Relaxed) as f32 * (*self.so2_value.read().unwrap());
        let mut so_mod = so1_mod;
        let enable =
            self.enable.load(Ordering::Relaxed) & self.all_sound_enable.load(Ordering::Relaxed);
        if !enable {
            for x in out.iter_mut() {
                *x = 0.0;
            }
        } else {
            let mut old_phase = self.phase;
            let mut pointer = self.pointer.load(Ordering::Relaxed);
            let wave_ram = self.wave_ram.lock().unwrap();
            for x in out.iter_mut() {
                if pointer % 2 == 0 {
                    *x = ((wave_ram[pointer / 2] >> 4) >> output_shift) as f32 * so_mod / 100.0;
                } else {
                    *x = ((wave_ram[(pointer - 1) / 2] & 0xF) >> output_shift) as f32 * so_mod
                        / 100.0;
                }
                right = !right;
                if right {
                    self.phase = (self.phase
                        + CYCLES_PER_SAMPLE as f32
                            / ((MAX_FREQ_VAL - self.frequency.load(Ordering::Relaxed)) as f32))
                        % 1.0;
                    if self.phase < old_phase {
                        pointer = (pointer + 1) % 32;
                        self.pointer.store(pointer, Ordering::Relaxed);
                    }
                    old_phase = self.phase;
                    so_mod = so1_mod;
                } else {
                    so_mod = so2_mod;
                }
            }
        }
    }
}

struct Channel4 {
    phase: f32,
    lsfr: Arc<AtomicU16>,
    enable: Arc<AtomicBool>,
    frequency: Arc<AtomicU32>,
    width: Arc<AtomicBool>,
    volume: Arc<AtomicU8>,
    out_1_bit: bool,
    so1_enable: Arc<AtomicU8>,
    so1_value: Arc<RwLock<f32>>,
    so2_enable: Arc<AtomicU8>,
    so2_value: Arc<RwLock<f32>>,
    all_sound_enable: Arc<AtomicBool>,
}

impl Channel4 {
    fn noise_lsfr(&mut self) {
        let mut lsfr = self.lsfr.load(Ordering::Relaxed);
        let new_bit = (lsfr & 1) ^ ((lsfr >> 1) & 1);
        lsfr >>= 1;
        lsfr += new_bit << 14;
        if self.width.load(Ordering::Relaxed) {
            lsfr &= 0b0111111;
            lsfr += new_bit << 6;
        }
        self.out_1_bit = (lsfr & 1) == 0;
        self.lsfr.store(lsfr, Ordering::Relaxed);
    }
}

impl AudioCallback for Channel4 {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        let enable =
            self.enable.load(Ordering::Relaxed) & self.all_sound_enable.load(Ordering::Relaxed);
        let volume = self.volume.load(Ordering::Relaxed);
        let frequency = self.frequency.load(Ordering::Relaxed);
        let mut right = true;
        let so1_mod =
            self.so1_enable.load(Ordering::Relaxed) as f32 * (*self.so1_value.read().unwrap());
        let so2_mod =
            self.so2_enable.load(Ordering::Relaxed) as f32 * (*self.so2_value.read().unwrap());
        let mut so_mod = so1_mod;
        for x in out.iter_mut() {
            let old_phase = self.phase;
            if !enable {
                *x = 0.0;
            } else if self.out_1_bit {
                *x = volume as f32 * so_mod / 100.0;
            } else {
                *x = 0.0;
            }
            right = !right;
            if right {
                let phase_add = CYCLES_PER_SAMPLE as f32 / frequency as f32;
                self.phase = (self.phase + phase_add) % 1.0;

                if self.phase < old_phase {
                    self.noise_lsfr();
                } else if phase_add >= 1.0 {
                    for _ in 0..phase_add as u8 {
                        self.noise_lsfr();
                    }
                }
                so_mod = so1_mod;
            } else {
                so_mod = so2_mod;
            }
        }
    }
}

pub struct AudioProcessingUnit {
    _audio_subsystem: AudioSubsystem,
    channel_1_frequency: Arc<AtomicU32>,
    channel_2_frequency: Arc<AtomicU32>,
    channel_1_sweep_count: u8,
    channel_1_sweep_enable: bool,
    cycle_count_1: u32,
    cycle_count_2: u32,
    cycle_count_3: u32,
    cycle_count_4: u32,
    so1_level_swl: SingleWriteLock<f32>,
    so2_level_swl: SingleWriteLock<f32>,
    all_sound_enable: Arc<AtomicBool>,
    nr51_data: u8,
    channel_1_enable: Arc<AtomicBool>,
    channel_1_volume: Arc<AtomicU8>,
    channel_1_volume_count: u8,
    channel_1_duty: Arc<Mutex<f32>>,
    channel_1_so1_enable: Arc<AtomicU8>,
    channel_1_so2_enable: Arc<AtomicU8>,

    channel_2_enable: Arc<AtomicBool>,
    channel_2_volume: Arc<AtomicU8>,
    channel_2_volume_count: u8,
    channel_2_duty: Arc<Mutex<f32>>,
    channel_2_so1_enable: Arc<AtomicU8>,
    channel_2_so2_enable: Arc<AtomicU8>,

    channel_3_pointer: Arc<AtomicUsize>,
    channel_3_enable: Arc<AtomicBool>,
    channel_3_frequency: Arc<AtomicU32>,
    channel_3_output_level: Arc<AtomicU8>,
    channel_3_so1_enable: Arc<AtomicU8>,
    channel_3_so2_enable: Arc<AtomicU8>,

    wave_ram: Arc<Mutex<[u8; 16]>>,
    channel_4_volume_count: u8,
    channel_4_lsfr: Arc<AtomicU16>,
    channel_4_enable: Arc<AtomicBool>,
    channel_4_frequency: Arc<AtomicU32>,
    channel_4_width: Arc<AtomicBool>,
    channel_4_volume: Arc<AtomicU8>,
    channel_4_so1_enable: Arc<AtomicU8>,
    channel_4_so2_enable: Arc<AtomicU8>,

    _channel_1_device: AudioDevice<Channel1>,
    _channel_2_device: AudioDevice<Channel2>,
    _channel_3_device: AudioDevice<Channel3>,
    _channel_4_device: AudioDevice<Channel4>,
}

impl AudioProcessingUnit {
    pub fn new(audio_subsystem: AudioSubsystem) -> AudioProcessingUnit {
        let cycle_count_1 = 0;
        let cycle_count_2 = 0;
        let cycle_count_3 = 0;
        let cycle_count_4 = 0;
        let so1_level = Arc::new(RwLock::new(0.0));
        let so2_level = Arc::new(RwLock::new(0.0));
        let all_sound_enable = Arc::new(AtomicBool::new(true));
        let channel_1_sweep_count = 0;
        let channel_1_enable = Arc::new(AtomicBool::new(false));
        let channel_1_volume = Arc::new(AtomicU8::new(0));
        let channel_1_volume_cb = Arc::clone(&channel_1_volume);
        let channel_1_volume_count = 0;
        let channel_1_sweep_enable = false;
        let channel_1_frequency = Arc::new(AtomicU32::new(0));
        let channel_1_frequency_cb = Arc::clone(&channel_1_frequency);
        let channel_1_enable_cb = Arc::clone(&channel_1_enable);
        let channel_1_duty = Arc::new(Mutex::new(0.0));
        let channel_1_duty_cb = Arc::clone(&channel_1_duty);
        let channel_1_so1_enable = Arc::new(AtomicU8::new(0));
        let channel_1_so1_enable_cb = Arc::clone(&channel_1_so1_enable);
        let channel_1_so2_enable = Arc::new(AtomicU8::new(0));
        let channel_1_so2_enable_cb = Arc::clone(&channel_1_so2_enable);
        let channel_1_so1_level_cb = Arc::clone(&so1_level);
        let channel_1_so2_level_cb = Arc::clone(&so2_level);
        let channel_1_all_sound_enable_cb = Arc::clone(&all_sound_enable);

        let channel_2_volume = Arc::new(AtomicU8::new(0));
        let channel_2_volume_cb = Arc::clone(&channel_2_volume);
        let channel_2_volume_count = 0;
        let channel_2_frequency = Arc::new(AtomicU32::new(0));
        let channel_2_frequency_cb = Arc::clone(&channel_2_frequency);
        let channel_2_enable = Arc::new(AtomicBool::new(false));
        let channel_2_enable_cb = Arc::clone(&channel_2_enable);
        let channel_2_duty = Arc::new(Mutex::new(0.0));
        let channel_2_duty_cb = Arc::clone(&channel_2_duty);
        let channel_2_so1_enable = Arc::new(AtomicU8::new(0));
        let channel_2_so1_enable_cb = Arc::clone(&channel_2_so1_enable);
        let channel_2_so2_enable = Arc::new(AtomicU8::new(0));
        let channel_2_so2_enable_cb = Arc::clone(&channel_2_so2_enable);
        let channel_2_so1_level_cb = Arc::clone(&so1_level);
        let channel_2_so2_level_cb = Arc::clone(&so2_level);
        let channel_2_all_sound_enable_cb = Arc::clone(&all_sound_enable);

        let channel_3_pointer = Arc::new(AtomicUsize::new(0));
        let channel_3_pointer_cb = Arc::clone(&channel_3_pointer);
        let wave_ram = Arc::new(Mutex::new([0; 16]));
        let channel_3_output_level = Arc::new(AtomicU8::new(0));
        let wave_ram_cb = Arc::clone(&wave_ram);
        let channel_3_output_level_cb = Arc::clone(&channel_3_output_level);
        let channel_3_enable = Arc::new(AtomicBool::new(false));
        let channel_3_enable_cb = Arc::clone(&channel_3_enable);
        let channel_3_frequency = Arc::new(AtomicU32::new(0));
        let channel_3_frequency_cb = Arc::clone(&channel_3_frequency);
        let channel_3_so1_enable = Arc::new(AtomicU8::new(0));
        let channel_3_so1_enable_cb = Arc::clone(&channel_3_so1_enable);
        let channel_3_so2_enable = Arc::new(AtomicU8::new(0));
        let channel_3_so2_enable_cb = Arc::clone(&channel_3_so2_enable);
        let channel_3_so1_level_cb = Arc::clone(&so1_level);
        let channel_3_so2_level_cb = Arc::clone(&so2_level);
        let channel_3_all_sound_enable_cb = Arc::clone(&all_sound_enable);

        let channel_4_volume_count = 0;
        let channel_4_lsfr = Arc::new(AtomicU16::new(0));
        let channel_4_enable = Arc::new(AtomicBool::new(false));
        let channel_4_frequency = Arc::new(AtomicU32::new(0));
        let channel_4_width = Arc::new(AtomicBool::new(false));
        let channel_4_volume = Arc::new(AtomicU8::new(0));
        let channel_4_lsfr_cb = Arc::clone(&channel_4_lsfr);
        let channel_4_enable_cb = Arc::clone(&channel_4_enable);
        let channel_4_frequency_cb = Arc::clone(&channel_4_frequency);
        let channel_4_width_cb = Arc::clone(&channel_4_width);
        let channel_4_volume_cb = Arc::clone(&channel_4_volume);
        let channel_4_so1_enable = Arc::new(AtomicU8::new(0));
        let channel_4_so1_enable_cb = Arc::clone(&channel_4_so1_enable);
        let channel_4_so2_enable = Arc::new(AtomicU8::new(0));
        let channel_4_so2_enable_cb = Arc::clone(&channel_4_so2_enable);
        let channel_4_so1_level_cb = Arc::clone(&so1_level);
        let channel_4_so2_level_cb = Arc::clone(&so2_level);
        let channel_4_all_sound_enable_cb = Arc::clone(&all_sound_enable);

        let so1_level_swl = SingleWriteLock::new(so1_level, 0.0);
        let so2_level_swl = SingleWriteLock::new(so2_level, 0.0);

        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLES_PER_SECOND as i32),
            channels: Some(2),
            samples: Some(32),
        };
        let channel_1_device = audio_subsystem
            .open_playback(None, &desired_spec, |_spec| {
                // initialize the audio callback
                Channel1 {
                    phase: 0.0,
                    frequency: channel_1_frequency_cb,
                    enable: channel_1_enable_cb,
                    volume: channel_1_volume_cb,
                    duty: channel_1_duty_cb,
                    so1_enable: channel_1_so1_enable_cb,
                    so1_value: channel_1_so1_level_cb,
                    so2_enable: channel_1_so2_enable_cb,
                    so2_value: channel_1_so2_level_cb,
                    all_sound_enable: channel_1_all_sound_enable_cb,
                }
            })
            .unwrap();

        let channel_2_device = audio_subsystem
            .open_playback(None, &desired_spec, |_spec| {
                // initialize the audio callback
                Channel2 {
                    phase: 0.0,
                    frequency: channel_2_frequency_cb,
                    enable: channel_2_enable_cb,
                    volume: channel_2_volume_cb,
                    duty: channel_2_duty_cb,
                    so1_enable: channel_2_so1_enable_cb,
                    so1_value: channel_2_so1_level_cb,
                    so2_enable: channel_2_so2_enable_cb,
                    so2_value: channel_2_so2_level_cb,
                    all_sound_enable: channel_2_all_sound_enable_cb,
                }
            })
            .unwrap();

        let channel_3_device = audio_subsystem
            .open_playback(None, &desired_spec, |_spec| {
                // initialize the audio callback
                Channel3 {
                    output_level: channel_3_output_level_cb,
                    pointer: channel_3_pointer_cb,
                    wave_ram: wave_ram_cb,
                    phase: 0.0,
                    enable: channel_3_enable_cb,
                    frequency: channel_3_frequency_cb,
                    so1_enable: channel_3_so1_enable_cb,
                    so1_value: channel_3_so1_level_cb,
                    so2_enable: channel_3_so2_enable_cb,
                    so2_value: channel_3_so2_level_cb,
                    all_sound_enable: channel_3_all_sound_enable_cb,
                }
            })
            .unwrap();
        let channel_4_device = audio_subsystem
            .open_playback(None, &desired_spec, |_spec| {
                // initialize the audio callback
                Channel4 {
                    phase: 0.0,
                    lsfr: channel_4_lsfr_cb,
                    enable: channel_4_enable_cb,
                    frequency: channel_4_frequency_cb,
                    width: channel_4_width_cb,
                    volume: channel_4_volume_cb,
                    out_1_bit: true,
                    so1_enable: channel_4_so1_enable_cb,
                    so1_value: channel_4_so1_level_cb,
                    so2_enable: channel_4_so2_enable_cb,
                    so2_value: channel_4_so2_level_cb,
                    all_sound_enable: channel_4_all_sound_enable_cb,
                }
            })
            .unwrap();
        channel_1_device.resume();
        channel_2_device.resume();
        channel_3_device.resume();
        channel_4_device.resume();
        AudioProcessingUnit {
            _audio_subsystem: audio_subsystem,
            channel_1_frequency,
            channel_2_frequency,
            cycle_count_1,
            cycle_count_2,
            cycle_count_3,
            cycle_count_4,
            so1_level_swl,
            so2_level_swl,
            all_sound_enable,
            nr51_data: 0,
            channel_1_sweep_count,
            channel_1_sweep_enable,
            channel_1_enable,
            channel_1_volume,
            channel_1_volume_count,
            channel_1_duty,
            channel_1_so1_enable,
            channel_1_so2_enable,
            channel_2_enable,
            channel_2_volume,
            channel_2_volume_count,
            channel_2_duty,
            channel_2_so1_enable,
            channel_2_so2_enable,
            channel_3_pointer,
            channel_3_enable,
            channel_3_frequency,
            channel_3_output_level,
            channel_3_so1_enable,
            channel_3_so2_enable,
            wave_ram,
            channel_4_volume_count,
            channel_4_lsfr,
            channel_4_enable,
            channel_4_frequency,
            channel_4_width,
            channel_4_volume,
            channel_4_so1_enable,
            channel_4_so2_enable,
            _channel_1_device: channel_1_device,
            _channel_2_device: channel_2_device,
            _channel_3_device: channel_3_device,
            _channel_4_device: channel_4_device,
        }
    }
}
impl GameBoyEmulator {
    fn volume_envelope(&mut self, channel: u8) {
        let (volume_reg, channel_volume_count, channel_volume) = match channel {
            1 => (
                self.mem_unit.get_memory(NR12_ADDR, SOURCE),
                &mut self.apu.channel_1_volume_count,
                &self.apu.channel_1_volume,
            ),
            2 => (
                self.mem_unit.get_memory(NR22_ADDR, SOURCE),
                &mut self.apu.channel_2_volume_count,
                &self.apu.channel_2_volume,
            ),
            4 => (
                self.mem_unit.get_memory(NR42_ADDR, SOURCE),
                &mut self.apu.channel_4_volume_count,
                &self.apu.channel_4_volume,
            ),
            _ => panic!(
                "Wow, how did you get here? You gave a channel for volume envelope that's bad."
            ),
        };
        let volume_time = volume_reg & 0b111;
        let vol_inc = ((volume_reg >> 3) & 1) == 1;
        let mut channel_volume_val = channel_volume.load(Ordering::Relaxed);
        if volume_time != 0 && *channel_volume_count >= (volume_time - 1) {
            channel_volume_val = if vol_inc && channel_volume_val < 15 {
                channel_volume_val + 1
            } else if !vol_inc && channel_volume_val > 0 {
                channel_volume_val - 1
            } else {
                channel_volume_val
            };
            *channel_volume_count = 0;
        } else {
            *channel_volume_count = std::cmp::min(*channel_volume_count + 1, 254);
        }
        channel_volume.store(channel_volume_val, Ordering::Relaxed);
    }
    fn sweep_channel_1(&mut self, frequency: &mut u32) {
        let (sweep_shift, sweep_inc, sweep_time) = {
            let sweep_reg = self.mem_unit.get_memory(NR10_ADDR, SOURCE);
            (sweep_reg & 0b111, (sweep_reg >> 3 & 1) == 0, sweep_reg >> 4)
        };
        if sweep_time != 0 && self.apu.channel_1_sweep_count >= (sweep_time - 1) && sweep_shift != 0
        {
            let old_frequency = *frequency;
            *frequency = if sweep_inc {
                *frequency + (*frequency >> sweep_shift)
            } else {
                *frequency - (*frequency >> sweep_shift)
            };
            self.apu.channel_1_sweep_count = 0;
            if *frequency >= MAX_FREQ_VAL {
                *frequency = old_frequency;
                self.apu.channel_1_enable.store(false, Ordering::Relaxed);
            } else {
                self.mem_unit
                    .write_memory(NR13_ADDR, (*frequency & 0xFF) as u8, SOURCE);
                self.mem_unit.write_memory(
                    NR14_ADDR,
                    (self.mem_unit.get_memory(NR14_ADDR, SOURCE) & 0b11111000)
                        | ((*frequency >> 8) & 0b111) as u8,
                    SOURCE,
                );
            }
        } else {
            self.apu.channel_1_sweep_count = std::cmp::min(self.apu.channel_1_sweep_count + 1, 254);
        }
    }
    fn length_unit_8(&mut self, channel: u8, length: &mut u8) {
        let (cc_reg, length_addr, enable_reg, nr52_bit_mask) = match channel {
            1 => (
                self.mem_unit.get_memory(NR14_ADDR, SOURCE),
                NR11_ADDR,
                &self.apu.channel_1_enable,
                0b11111110,
            ),
            2 => (
                self.mem_unit.get_memory(NR24_ADDR, SOURCE),
                NR21_ADDR,
                &self.apu.channel_2_enable,
                0b11111101,
            ),
            4 => (
                self.mem_unit.get_memory(NR44_ADDR, SOURCE),
                NR41_ADDR,
                &self.apu.channel_4_enable,
                0b11110111,
            ),
            _ => {
                panic!("Wow, how did you get here? You gave a channel for length unit that's bad.")
            }
        };
        let counter_consec = (cc_reg >> 6) & 1;
        if counter_consec == 1 {
            if *length <= 1 {
                enable_reg.store(false, Ordering::Relaxed);
                self.mem_unit.write_memory(
                    NR52_ADDR,
                    self.mem_unit.get_memory(NR52_ADDR, SOURCE) & nr52_bit_mask,
                    SOURCE,
                );
            } else {
                self.mem_unit.write_memory(
                    length_addr,
                    self.mem_unit.get_memory(length_addr, SOURCE) & 0b11000000
                        | (MAX_8_LENGTH - *length),
                    SOURCE,
                );
                *length -= 1;
            }
        }
    }

    fn length_unit_16(&mut self, length: &mut u16) {
        let cc_reg = self.mem_unit.get_memory(NR34_ADDR, SOURCE);
        let counter_consec = cc_reg >> 6 & 1;
        if counter_consec == 1 {
            if *length == 0 {
                self.apu.channel_3_enable.store(false, Ordering::Relaxed);
                self.mem_unit.write_memory(
                    NR52_ADDR,
                    self.mem_unit.get_memory(NR52_ADDR, SOURCE) & 0b11111011,
                    SOURCE,
                );
            } else {
                self.mem_unit.write_memory(
                    NR31_ADDR,
                    self.mem_unit.get_memory(NR31_ADDR, SOURCE) & 0b11000000
                        | (MAX_16_LENGTH - *length) as u8,
                    SOURCE,
                );
                *length -= 1;
            }
        }
    }

    fn channel_1_advance(&mut self) {
        let initialize = (self.mem_unit.get_memory(NR14_ADDR, SOURCE) >> 7) == 1;
        let mut length = MAX_8_LENGTH - (self.mem_unit.get_memory(NR11_ADDR, SOURCE) & 0b111111);

        self.apu
            .channel_1_so1_enable
            .store(self.apu.nr51_data & 1, Ordering::Relaxed);
        self.apu
            .channel_1_so2_enable
            .store((self.apu.nr51_data >> 4) & 1, Ordering::Relaxed);
        if initialize {
            if length == 0 {
                length = MAX_8_LENGTH;
            }
            self.apu.channel_1_enable.store(true, Ordering::Relaxed);
            self.mem_unit.write_memory(
                NR52_ADDR,
                self.mem_unit.get_memory(NR52_ADDR, SOURCE) | 1,
                SOURCE,
            );
            self.apu.channel_1_sweep_count = 0;
            self.apu.channel_1_volume_count = 0;
            self.apu.channel_1_volume.store(
                self.mem_unit.get_memory(NR12_ADDR, SOURCE) >> 4,
                Ordering::Relaxed,
            );
            let (sweep_shift, sweep_time) = {
                let sweep_reg = self.mem_unit.get_memory(NR10_ADDR, SOURCE);
                (sweep_reg & 0b11, sweep_reg >> 4)
            };
            self.apu.channel_1_sweep_enable = !(sweep_shift == 0 || sweep_time == 0);
            self.apu.cycle_count_1 = 0;
            self.mem_unit.write_memory(
                NR14_ADDR,
                self.mem_unit.get_memory(NR14_ADDR, SOURCE) & 0b01111111,
                SOURCE,
            );
        }
        if self.apu.channel_1_enable.load(Ordering::Relaxed) {
            *self.apu.channel_1_duty.lock().unwrap() = DUTY_CONVERSION
                [((self.mem_unit.get_memory(NR11_ADDR, SOURCE) >> 6) & 0b11) as usize];
            let mut channel_1_frequency =
                (((self.mem_unit.get_memory(NR14_ADDR, SOURCE) & 0b111) as u32) << 8)
                    + self.mem_unit.get_memory(NR13_ADDR, SOURCE) as u32;

            if self.apu.cycle_count_1 % CYCLE_COUNT_128HZ == 0 && self.apu.channel_1_sweep_enable {
                self.sweep_channel_1(&mut channel_1_frequency);
            }
            if self.apu.cycle_count_1 == 0 {
                self.volume_envelope(1);
            }

            if self.apu.cycle_count_1 % CYCLE_COUNT_256HZ == 0 {
                self.length_unit_8(1, &mut length);
            }
            self.apu
                .channel_1_frequency
                .store(channel_1_frequency, Ordering::Relaxed);
        }
    }
    fn channel_2_advance(&mut self) {
        let initialize = (self.mem_unit.get_memory(NR24_ADDR, SOURCE) >> 7) == 1;
        let mut length = 64 - (self.mem_unit.get_memory(NR21_ADDR, SOURCE) & 0b111111);
        self.apu
            .channel_2_so1_enable
            .store((self.apu.nr51_data >> 1) & 1, Ordering::Relaxed);
        self.apu
            .channel_2_so2_enable
            .store((self.apu.nr51_data >> 5) & 1, Ordering::Relaxed);
        if initialize {
            //println!("2 initialize!");
            if length == 0 {
                length = MAX_8_LENGTH;
            }
            self.apu.channel_2_enable.store(true, Ordering::Relaxed);
            self.mem_unit.write_memory(
                NR52_ADDR,
                self.mem_unit.get_memory(NR52_ADDR, SOURCE) | 0b10,
                SOURCE,
            );
            self.apu.channel_2_volume_count = 0;
            self.apu.channel_2_volume.store(
                self.mem_unit.get_memory(NR22_ADDR, SOURCE) >> 4,
                Ordering::Relaxed,
            );
            self.mem_unit.write_memory(
                NR24_ADDR,
                self.mem_unit.get_memory(NR24_ADDR, SOURCE) & 0b01111111,
                SOURCE,
            );
            self.apu.cycle_count_2 = 0;
        }
        *self.apu.channel_2_duty.lock().unwrap() =
            DUTY_CONVERSION[((self.mem_unit.get_memory(NR21_ADDR, SOURCE) >> 6) & 0b11) as usize];
        self.apu.channel_2_frequency.store(
            (((self.mem_unit.get_memory(NR24_ADDR, SOURCE) & 0b111) as u32) << 8)
                + self.mem_unit.get_memory(NR23_ADDR, SOURCE) as u32,
            Ordering::Relaxed,
        );

        if self.apu.cycle_count_2 == 0 {
            self.volume_envelope(2);
        }

        if self.apu.cycle_count_2 % CYCLE_COUNT_256HZ == 0 {
            self.length_unit_8(2, &mut length);
        }
    }

    fn channel_3_advance(&mut self) {
        let initialize = (self.mem_unit.get_memory(NR34_ADDR, SOURCE) >> 7) == 1;
        let mut length = MAX_16_LENGTH - self.mem_unit.get_memory(NR31_ADDR, SOURCE) as u16;
        self.apu
            .channel_3_so1_enable
            .store((self.apu.nr51_data >> 2) & 1, Ordering::Relaxed);
        self.apu
            .channel_3_so2_enable
            .store((self.apu.nr51_data >> 6) & 1, Ordering::Relaxed);
        if initialize {
            //println!("3 initialize!");
            if length == 0 {
                length = MAX_16_LENGTH;
            }
            self.apu.cycle_count_3 = 0;
            self.apu.channel_3_pointer.store(0, Ordering::Relaxed);
            self.mem_unit.write_memory(
                NR34_ADDR,
                self.mem_unit.get_memory(NR34_ADDR, SOURCE) & 0b01111111,
                SOURCE,
            );
            self.apu.channel_3_enable.store(true, Ordering::Relaxed);
            self.mem_unit.write_memory(
                NR52_ADDR,
                self.mem_unit.get_memory(NR52_ADDR, SOURCE) | 0b100,
                SOURCE,
            );
        }
        if self.mem_unit.get_memory(NR30_ADDR, SOURCE) >> 7 == 0 {
            self.apu.channel_3_enable.store(false, Ordering::Relaxed);
        }
        *self.apu.wave_ram.lock().unwrap() = self.mem_unit.get_wave_ram();
        self.apu.channel_3_output_level.store(
            self.mem_unit.get_memory(NR32_ADDR, SOURCE),
            Ordering::Relaxed,
        );
        self.apu.channel_3_frequency.store(
            (((self.mem_unit.get_memory(NR34_ADDR, SOURCE) & 0b111) as u32) << 8)
                + self.mem_unit.get_memory(NR33_ADDR, SOURCE) as u32,
            Ordering::Relaxed,
        );
        if self.apu.cycle_count_3 % 16384 == 0 {
            self.length_unit_16(&mut length);
        }
    }
    fn channel_4_advance(&mut self) {
        let initialize = (self.mem_unit.get_memory(NR44_ADDR, SOURCE) >> 7) == 1;
        let mut length = 64 - (self.mem_unit.get_memory(NR41_ADDR, SOURCE) & 0b11111);
        self.apu
            .channel_4_so1_enable
            .store((self.apu.nr51_data >> 3) & 1, Ordering::Relaxed);
        self.apu
            .channel_4_so2_enable
            .store((self.apu.nr51_data >> 7) & 1, Ordering::Relaxed);
        if initialize {
            //println!("4 initialize!");
            if length == 0 {
                length = MAX_8_LENGTH;
            }
            self.mem_unit.write_memory(
                NR44_ADDR,
                self.mem_unit.get_memory(NR44_ADDR, SOURCE) & 0b01111111,
                SOURCE,
            );
            self.apu.channel_4_volume_count = 0;
            self.apu.channel_4_volume.store(
                self.mem_unit.get_memory(NR42_ADDR, SOURCE) >> 4,
                Ordering::Relaxed,
            );
            self.apu.channel_4_lsfr.store(0x7FFF, Ordering::Relaxed);
            self.apu.channel_4_enable.store(true, Ordering::Relaxed);
            self.mem_unit.write_memory(
                NR52_ADDR,
                self.mem_unit.get_memory(NR52_ADDR, SOURCE) | 0b1000,
                SOURCE,
            );
            self.apu.cycle_count_4 = 0;
        }
        let poly_counter_reg = self.mem_unit.get_memory(NR43_ADDR, SOURCE);
        self.apu
            .channel_4_width
            .store(((poly_counter_reg >> 3) & 1) == 1, Ordering::Relaxed);
        let new_freq = {
            let shift_clock_freq = poly_counter_reg >> 4;
            let possible_ratio = poly_counter_reg & 0b111;
            let s_factor: u32 = 1 << (shift_clock_freq + 4);
            if possible_ratio == 0 {
                s_factor >> 1
            } else {
                s_factor * possible_ratio as u32
            }
        };
        self.apu
            .channel_4_frequency
            .store(new_freq, Ordering::Relaxed);
        if self.apu.cycle_count_4 == 0 {
            self.volume_envelope(4);
        }

        if self.apu.cycle_count_4 % CYCLE_COUNT_256HZ == 0 {
            self.length_unit_8(4, &mut length);
        }
    }
    pub fn apu_advance(&mut self) {
        self.apu.nr51_data = self.mem_unit.get_memory(NR51_ADDR, SOURCE);
        self.apu
            .so1_level_swl
            .set(((self.mem_unit.get_memory(NR50_ADDR, SOURCE) >> 4) & 0b11) as f32 / 7.0);
        self.apu
            .so2_level_swl
            .set((self.mem_unit.get_memory(NR50_ADDR, SOURCE) & 0b11) as f32 / 7.0);
        self.apu.all_sound_enable.store(
            (self.mem_unit.get_memory(NR52_ADDR, SOURCE) >> 7) == 1,
            Ordering::Relaxed,
        );
        self.channel_1_advance();
        self.channel_2_advance();
        self.channel_3_advance();
        self.channel_4_advance();
        self.apu.cycle_count_1 = (self.apu.cycle_count_1 + ADVANCE_CYCLES) % CYCLE_COUNT_512HZ;
        self.apu.cycle_count_2 = (self.apu.cycle_count_2 + ADVANCE_CYCLES) % CYCLE_COUNT_512HZ;
        self.apu.cycle_count_3 = (self.apu.cycle_count_3 + ADVANCE_CYCLES) % CYCLE_COUNT_512HZ;
        self.apu.cycle_count_4 = (self.apu.cycle_count_4 + ADVANCE_CYCLES) % CYCLE_COUNT_512HZ;
    }
}

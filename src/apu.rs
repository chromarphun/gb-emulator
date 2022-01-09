use crate::constants::*;
use crate::emulator::GameBoyEmulator;
use crate::emulator::RequestSource;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::AudioSubsystem;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

const SOURCE: RequestSource = RequestSource::APU;

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
        let output_shift = self.output_level.load(Ordering::Relaxed);
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
                    // self.phase = (self.phase
                    //     + CYCLES_PER_SAMPLE as f32
                    //         / ((MAX_FREQ_VAL - self.frequency.load(Ordering::Relaxed)) as f32))
                    //     % 1.0;
                    self.phase = (self.phase
                        + (2097152.0
                            / ((MAX_FREQ_VAL - self.frequency.load(Ordering::Relaxed)) as f32))
                            / SAMPLES_PER_SECOND as f32)
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
                let phase_add = frequency as f32 / SAMPLES_PER_SECOND as f32;
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
    pub length_counters: [u16; 4],
    length_enables: [bool; 4],
    sequence_counter: u8,
    initial_volumes: [u8; 4],
    vol_inc_flags: [bool; 4],
    vol_periods: [u8; 4],
    volumes: [Arc<AtomicU8>; 4],
    vol_timers: [u8; 4],
    pub apu_power: bool,
    channel_1_frequency: Arc<AtomicU32>,
    channel_1_shadow_frequency: u32,
    channel_2_frequency: Arc<AtomicU32>,
    channel_1_sweep_timer: u8,
    channel_1_sweep_enable: bool,
    channel_1_sweep_inc: bool,
    channel_1_sweep_period: u8,
    channel_1_sweep_shift: u8,
    channel_1_neg_after_trigger: bool,
    cycle_count: u32,

    so1_level: Arc<RwLock<f32>>,
    so2_level: Arc<RwLock<f32>>,
    pub all_sound_enable: Arc<AtomicBool>,
    channel_1_enable: Arc<AtomicBool>,

    channel_1_duty: Arc<Mutex<f32>>,
    channel_1_so1_enable: Arc<AtomicU8>,
    channel_1_so2_enable: Arc<AtomicU8>,

    channel_2_enable: Arc<AtomicBool>,

    channel_2_duty: Arc<Mutex<f32>>,
    channel_2_so1_enable: Arc<AtomicU8>,
    channel_2_so2_enable: Arc<AtomicU8>,

    channel_3_pointer: Arc<AtomicUsize>,
    channel_3_enable: Arc<AtomicBool>,
    channel_3_frequency: Arc<AtomicU32>,
    pub channel_3_output_level: Arc<AtomicU8>,
    channel_3_so1_enable: Arc<AtomicU8>,
    channel_3_so2_enable: Arc<AtomicU8>,

    pub wave_ram: Arc<Mutex<[u8; 16]>>,
    channel_4_lsfr: Arc<AtomicU16>,
    channel_4_enable: Arc<AtomicBool>,
    channel_4_frequency: Arc<AtomicU32>,
    channel_4_width: Arc<AtomicBool>,
    channel_4_so1_enable: Arc<AtomicU8>,
    channel_4_so2_enable: Arc<AtomicU8>,

    _channel_1_device: AudioDevice<Channel1>,
    _channel_2_device: AudioDevice<Channel2>,
    _channel_3_device: AudioDevice<Channel3>,
    _channel_4_device: AudioDevice<Channel4>,
}

impl AudioProcessingUnit {
    pub fn new(audio_subsystem: AudioSubsystem) -> AudioProcessingUnit {
        let cycle_count = 0;
        let so1_level = Arc::new(RwLock::new(0.0));
        let so2_level = Arc::new(RwLock::new(0.0));
        let all_sound_enable = Arc::new(AtomicBool::new(true));
        let channel_1_sweep_timer = 1;
        let channel_1_enable = Arc::new(AtomicBool::new(false));
        let channel_1_volume = Arc::new(AtomicU8::new(0));
        let channel_1_volume_cb = Arc::clone(&channel_1_volume);

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

        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLES_PER_SECOND as i32),
            channels: Some(2),
            samples: Some(SAMPLE_BUFFER_SIZE),
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
            length_counters: [0; 4],
            length_enables: [false; 4],
            sequence_counter: 0,
            initial_volumes: [0; 4],
            vol_inc_flags: [false; 4],
            vol_periods: [0; 4],
            volumes: [
                channel_1_volume,
                channel_2_volume,
                Arc::new(AtomicU8::new(0)),
                channel_4_volume,
            ],
            vol_timers: [1; 4],
            apu_power: false,
            channel_1_frequency,
            channel_1_shadow_frequency: 0,
            channel_2_frequency,
            cycle_count,

            so1_level,
            so2_level,
            all_sound_enable,

            channel_1_sweep_timer,
            channel_1_sweep_enable,
            channel_1_enable,

            channel_1_duty,
            channel_1_so1_enable,
            channel_1_so2_enable,
            channel_1_sweep_inc: false,
            channel_1_sweep_period: 0,
            channel_1_sweep_shift: 0,
            channel_1_neg_after_trigger: false,
            channel_2_enable,
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
            channel_4_lsfr,
            channel_4_enable,
            channel_4_frequency,
            channel_4_width,
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
    pub fn disable_channel(&mut self, channel: usize) {
        let (enable_channel, mask) = match channel {
            1 => (&self.apu.channel_1_enable, 0b11111110),
            2 => (&self.apu.channel_2_enable, 0b11111101),
            3 => (&self.apu.channel_3_enable, 0b11111011),
            4 => (&self.apu.channel_4_enable, 0b11110111),
            _ => panic!("Wow, how did you get here? You gave a channel for disable that's bad."),
        };
        (*enable_channel).store(false, Ordering::Relaxed);
        self.write_memory(NR52_ADDR, self.get_memory(NR52_ADDR, SOURCE) & mask, SOURCE);
    }
    fn enable_channel(&mut self, channel: usize) {
        let (enable_channel, mask) = match channel {
            1 => (&self.apu.channel_1_enable, 0b00000001),
            2 => (&self.apu.channel_2_enable, 0b00000010),
            3 => (&self.apu.channel_3_enable, 0b00000100),
            4 => (&self.apu.channel_4_enable, 0b00001000),
            _ => panic!("Wow, how did you get here? You gave a channel for enable that's bad."),
        };
        (*enable_channel).store(true, Ordering::Relaxed);
        self.write_memory(NR52_ADDR, self.get_memory(NR52_ADDR, SOURCE) | mask, SOURCE);
    }
    pub fn apu_power_up(&mut self) {
        self.apu.cycle_count = 0;
        self.apu.sequence_counter = 0;
        self.apu.all_sound_enable.store(true, Ordering::Relaxed);
        self.apu.apu_power = true;
    }

    fn update_frequency_addr(&mut self, channel: usize) {
        let (low_reg_addr, high_reg_addr, frequency_val) = match channel {
            1 => (
                NR13_ADDR,
                NR14_ADDR,
                self.apu.channel_1_frequency.load(Ordering::Relaxed),
            ),
            _ => panic!("bad channel!"),
        };
        self.apu.channel_1_shadow_frequency = frequency_val;
        let low_reg_val = (frequency_val & 0xFF) as u8;
        let high_reg_part_val = ((frequency_val >> 8) & 0x7) as u8;
        self.write_memory(low_reg_addr, low_reg_val, SOURCE);
        let high_reg_val = (self.get_memory(high_reg_addr, SOURCE) & 0xF8) | high_reg_part_val;
        self.write_memory(high_reg_addr, high_reg_val, SOURCE);
    }
    pub fn update_frequency_internal_low(&mut self, channel: usize, val: u8) {
        //println!("trying to change freq of {}", channel);
        let freq_atomic = match channel {
            1 => &mut self.apu.channel_1_frequency,
            2 => &mut self.apu.channel_2_frequency,
            3 => &mut self.apu.channel_3_frequency,
            _ => panic!("bad channel!"),
        };
        let mut new_freq = (*freq_atomic).load(Ordering::Relaxed);
        new_freq &= 0x700;
        new_freq |= val as u32;
        (*freq_atomic).store(new_freq, Ordering::Relaxed);
        if channel == 1 && !self.apu.channel_1_sweep_enable {
            self.apu.channel_1_shadow_frequency = new_freq;
        }
    }
    fn volume_unit(&mut self, channel: usize) {
        let current_vol = self.apu.volumes[channel - 1].load(Ordering::Relaxed);
        let new_vol = if self.apu.vol_inc_flags[channel - 1] {
            if current_vol < 0xF {
                current_vol + 1
            } else {
                current_vol
            }
        } else {
            if current_vol > 0 {
                current_vol - 1
            } else {
                current_vol
            }
        };
        self.apu.volumes[channel - 1].store(new_vol, Ordering::Relaxed);
    }
    fn sweep_unit(&mut self, clocked: bool) {
        let op_val = self.apu.channel_1_shadow_frequency >> self.apu.channel_1_sweep_shift;
        let mut new_freq = if self.apu.channel_1_sweep_inc {
            self.apu.channel_1_shadow_frequency + op_val
        } else {
            self.apu.channel_1_shadow_frequency - op_val
        };
        if new_freq >= MAX_FREQ_VAL {
            self.disable_channel(1);
        } else {
            if self.apu.channel_1_sweep_shift != 0 && clocked {
                self.apu
                    .channel_1_frequency
                    .store(new_freq, Ordering::Relaxed);
                self.update_frequency_addr(1);

                let second_op_val = new_freq >> self.apu.channel_1_sweep_shift;
                new_freq = if self.apu.channel_1_sweep_inc {
                    new_freq + second_op_val
                } else {
                    new_freq - second_op_val
                };
                if new_freq >= MAX_FREQ_VAL {
                    self.disable_channel(1);
                }
            }
        }
        if !self.apu.channel_1_sweep_inc {
            self.apu.channel_1_neg_after_trigger = true;
        }
    }
    fn length_unit(&mut self, channel: usize) {
        self.apu.length_counters[channel - 1] -= 1;
        if self.apu.length_counters[channel - 1] == 0 {
            self.disable_channel(channel);
        }
    }
    fn dac_check(&mut self, channel: usize) {
        if channel == 3 {
            let on_off = self.get_memory(NR30_ADDR, SOURCE);
            if on_off >> 7 == 0 {
                self.disable_channel(3);
            }
            return;
        }
        let vol_env_addr = match channel {
            1 => NR12_ADDR,
            2 => NR22_ADDR,
            4 => NR42_ADDR,
            _ => {
                panic!(
                    "Wow, how did you get here? You gave a channel for DAC check unit that's bad."
                )
            }
        };
        if (self.get_memory(vol_env_addr, SOURCE) >> 3) == 0 {
            self.disable_channel(channel);
        }
    }
    pub fn wave_ram_write(&mut self, addr: usize, val: u8) {
        if self.get_memory(NR30_ADDR, SOURCE) >> 7 == 0 {
            self.apu.wave_ram.lock().unwrap()[addr - 0xFF30] = val;
            self.write_memory(addr, val, SOURCE);
        }
    }
    pub fn wave_ram_read(&self, addr: usize) -> u8 {
        let addr_send = if self.get_memory(NR30_ADDR, SOURCE) >> 7 == 0 {
            addr
        } else {
            self.apu.channel_3_pointer.load(Ordering::Relaxed) + 0xFF32
        };
        self.get_memory(addr_send, SOURCE)
    }
    pub fn nrx1_write(&mut self, channel: usize, val: u8) {
        let (mask, max) = match channel {
            1 => {
                *self.apu.channel_1_duty.lock().unwrap() = DUTY_CONVERSION[(val >> 6) as usize];
                (0x3F, 64)
            }
            2 => {
                *self.apu.channel_2_duty.lock().unwrap() = DUTY_CONVERSION[(val >> 6) as usize];
                (0x3F, 64)
            }
            3 => (0xFF, 256),
            4 => (0x3F, 64),
            _ => panic!("Bad channel!"),
        };

        self.apu.length_counters[channel - 1] = max - (val as u16 & mask);
    }
    pub fn vol_env_write(&mut self, channel: usize, val: u8) {
        self.apu.initial_volumes[channel - 1] = val >> 4;
        self.apu.vol_inc_flags[channel - 1] = ((val >> 3) & 1) == 1;
        self.apu.vol_periods[channel - 1] = val & 0x7;
        if val >> 3 == 0 {
            self.disable_channel(channel);
        }
    }
    pub fn nrx4_write(&mut self, channel: usize, val: u8) {
        let old_enable = self.apu.length_enables[channel - 1];
        self.apu.length_enables[channel - 1] = (val >> 6) & 1 == 1;

        let extra_length = !old_enable
            && self.apu.length_enables[channel - 1]
            && (self.apu.sequence_counter % 2 == 1)
            && self.apu.length_counters[channel - 1] > 0;
        if extra_length {
            self.length_unit(channel);
        }

        let trigger = (val >> 7) & 1 == 1;
        if channel != 4 {
            let freq_atomic = match channel {
                1 => &mut self.apu.channel_1_frequency,
                2 => &mut self.apu.channel_2_frequency,
                3 => &mut self.apu.channel_3_frequency,
                _ => panic!("bad channel!"),
            };
            let mut new_freq = (*freq_atomic).load(Ordering::Relaxed);
            new_freq &= 0xFF;
            new_freq |= (val as u32 & 0x7) << 8;
            (*freq_atomic).store(new_freq, Ordering::Relaxed);
            if (channel == 1) && (!self.apu.channel_1_sweep_enable || trigger) {
                self.apu.channel_1_shadow_frequency = new_freq;
            }
        }

        if trigger {
            match channel {
                1 => self.trigger_channel_1(),
                2 => self.trigger_channel_2(),
                3 => self.trigger_channel_3(),
                4 => self.trigger_channel_4(),
                _ => {
                    panic!("bad channel!")
                }
            }
        }
    }
    pub fn poly_count_var_update(&mut self, val: u8) {
        let val = val as u32;
        let ratio = val & 0x7;
        let shift_clock_freq = val >> 4;
        let dividing_factor = if ratio == 0 {
            1 << shift_clock_freq
        } else {
            (1 << (shift_clock_freq + 1)) * ratio
        };
        self.apu
            .channel_4_frequency
            .store(524288 / dividing_factor, Ordering::Relaxed);
        self.apu
            .channel_4_width
            .store(((val >> 3) & 1) == 1, Ordering::Relaxed);
    }
    pub fn sweep_var_update(&mut self, val: u8) {
        let (sweep_shift, sweep_period, sweep_inc) =
            { (val & 0b111, (val >> 4) & 0b111, ((val >> 3) & 1) == 0) };
        self.apu.channel_1_sweep_shift = sweep_shift;
        self.apu.channel_1_sweep_period = sweep_period;
        self.apu.channel_1_sweep_inc = sweep_inc;
        if sweep_inc && self.apu.channel_1_neg_after_trigger {
            self.disable_channel(1);
        }
    }
    pub fn nr50_write(&mut self, val: u8) {
        *self.apu.so1_level.write().unwrap() = (val & 0x7) as f32 / 7.0;
        *self.apu.so2_level.write().unwrap() = ((val >> 4) & 0x7) as f32 / 7.0;
    }
    pub fn nr51_write(&mut self, val: u8) {
        self.apu
            .channel_1_so1_enable
            .store(val & 1, Ordering::Relaxed);
        self.apu
            .channel_2_so1_enable
            .store((val >> 1) & 1, Ordering::Relaxed);
        self.apu
            .channel_3_so1_enable
            .store((val >> 2) & 1, Ordering::Relaxed);
        self.apu
            .channel_4_so1_enable
            .store((val >> 3) & 1, Ordering::Relaxed);

        self.apu
            .channel_1_so2_enable
            .store((val >> 4) & 1, Ordering::Relaxed);
        self.apu
            .channel_2_so2_enable
            .store((val >> 5) & 1, Ordering::Relaxed);
        self.apu
            .channel_3_so2_enable
            .store((val >> 6) & 1, Ordering::Relaxed);
        self.apu
            .channel_4_so2_enable
            .store((val >> 7) & 1, Ordering::Relaxed);
    }
    fn refill_check(&mut self, channel: usize) {
        let max_length = if channel == 3 {
            MAX_16_LENGTH
        } else {
            MAX_8_LENGTH
        };
        if self.apu.length_counters[channel - 1] == 0 {
            self.apu.length_counters[channel - 1] = max_length;
            if self.apu.length_enables[channel - 1] && self.apu.sequence_counter % 2 == 1 {
                self.length_unit(channel);
            }
        }
    }
    fn trigger_channel_1(&mut self) {
        self.apu.channel_1_neg_after_trigger = false;
        self.enable_channel(1);
        self.dac_check(1);

        self.refill_check(1);

        self.apu.channel_1_sweep_timer = if self.apu.channel_1_sweep_period == 0 {
            8
        } else {
            self.apu.channel_1_sweep_period
        };

        self.apu.vol_timers[CH1_IND] = if self.apu.vol_periods[CH1_IND] == 0 {
            8
        } else {
            self.apu.vol_periods[CH1_IND]
        };
        if self.apu.sequence_counter == 6 {
            self.apu.vol_periods[CH1_IND] += 1;
        }
        self.apu.volumes[CH1_IND].store(self.apu.initial_volumes[CH1_IND], Ordering::Relaxed);
        self.apu.channel_1_sweep_enable =
            self.apu.channel_1_sweep_shift != 0 || self.apu.channel_1_sweep_period != 0;
        self.apu.channel_1_shadow_frequency = self.apu.channel_1_frequency.load(Ordering::Relaxed);

        if self.apu.channel_1_sweep_shift != 0 {
            self.sweep_unit(false);
        }
    }

    fn trigger_channel_2(&mut self) {
        self.apu.vol_timers[CH2_IND] = if self.apu.vol_periods[CH2_IND] == 0 {
            8
        } else {
            self.apu.vol_periods[CH2_IND]
        };
        if self.apu.sequence_counter == 6 {
            self.apu.vol_periods[CH2_IND] += 1;
        }
        self.apu.volumes[CH2_IND].store(self.apu.initial_volumes[CH2_IND], Ordering::Relaxed);
        self.enable_channel(2);
        self.dac_check(2);
        self.refill_check(2);
    }
    fn trigger_channel_3(&mut self) {
        self.apu.channel_3_pointer.store(0, Ordering::Relaxed);
        self.enable_channel(3);
        self.refill_check(3);
        self.dac_check(3);
    }
    fn trigger_channel_4(&mut self) {
        self.apu.vol_timers[CH4_IND] = if self.apu.vol_periods[CH4_IND] == 0 {
            8
        } else {
            self.apu.vol_periods[CH4_IND]
        };
        if self.apu.sequence_counter == 6 {
            self.apu.vol_periods[CH4_IND] += 1;
        }
        self.apu.volumes[CH4_IND].store(self.apu.initial_volumes[CH4_IND], Ordering::Relaxed);
        self.apu.channel_4_lsfr.store(0x7FFF, Ordering::Relaxed);
        self.enable_channel(4);
        self.refill_check(4);
        self.dac_check(4);
    }

    pub fn apu_advance(&mut self) {
        self.apu.cycle_count = (self.apu.cycle_count + ADVANCE_CYCLES) % CYCLE_COUNT_64HZ;

        if (self.apu.cycle_count % CYCLE_COUNT_512HZ) == 0 && self.apu.apu_power {
            self.apu.sequence_counter = (self.apu.sequence_counter + 1) % 8;
            if self.apu.sequence_counter % 2 == 1 {
                if self.apu.length_enables[CH1_IND] && self.apu.length_counters[CH1_IND] > 0 {
                    self.length_unit(1);
                }
                if self.apu.length_enables[CH2_IND] && self.apu.length_counters[CH2_IND] > 0 {
                    self.length_unit(2);
                }
                if self.apu.length_enables[CH3_IND] && self.apu.length_counters[CH3_IND] > 0 {
                    self.length_unit(3);
                }
                if self.apu.length_enables[CH4_IND] && self.apu.length_counters[CH4_IND] > 0 {
                    self.length_unit(4);
                }
            }
            if self.apu.sequence_counter % 4 == 3 {
                self.apu.channel_1_sweep_timer -= 1;
                if self.apu.channel_1_sweep_timer == 0 {
                    self.apu.channel_1_sweep_timer = if self.apu.channel_1_sweep_period == 0 {
                        8
                    } else {
                        self.apu.channel_1_sweep_period
                    };
                    if self.apu.channel_1_sweep_enable && self.apu.channel_1_sweep_period != 0 {
                        self.sweep_unit(true)
                    }
                }
            }
            if self.apu.sequence_counter == 7 {
                for channel_ind in [0, 1, 3].iter() {
                    self.apu.vol_timers[*channel_ind] -= 1;
                    if self.apu.vol_timers[*channel_ind] == 0 {
                        if self.apu.vol_periods[*channel_ind] != 0 {
                            self.volume_unit(*channel_ind + 1);
                        }
                        self.apu.vol_timers[*channel_ind] =
                            if self.apu.vol_periods[*channel_ind] == 0 {
                                8
                            } else {
                                self.apu.vol_periods[*channel_ind]
                            }
                    }
                }
            }
        }
    }
}

use crate::emulator::GameBoyEmulator;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::AudioSubsystem;

use std::sync::{Arc, Mutex, RwLock};

use crate::emulator::{ADVANCE_CYCLES, CYCLES_PER_SAMPLE};

const DUTY_CONVERSION: [f32; 4] = [0.125, 0.25, 0.5, 0.75];

const VOLUME_SHIFT_CONVERSION: [u8; 4] = [4, 0, 1, 2];

const NR10_ADDR: usize = 0xFF10;
const NR11_ADDR: usize = 0xFF11;
const NR12_ADDR: usize = 0xFF12;
const NR13_ADDR: usize = 0xFF13;
const NR14_ADDR: usize = 0xFF14;

const NR21_ADDR: usize = 0xFF16;
const NR22_ADDR: usize = 0xFF17;
const NR23_ADDR: usize = 0xFF18;
const NR24_ADDR: usize = 0xFF19;

const NR30_ADDR: usize = 0xFF1A;
const NR31_ADDR: usize = 0xFF1B;
const NR32_ADDR: usize = 0xFF1C;
const NR33_ADDR: usize = 0xFF1D;
const NR34_ADDR: usize = 0xFF1E;

const NR41_ADDR: usize = 0xFF20;
const NR42_ADDR: usize = 0xFF21;
const NR43_ADDR: usize = 0xFF22;
const NR44_ADDR: usize = 0xFF23;

const NR50_ADDR: usize = 0xFF24;
const NR51_ADDR: usize = 0xFF25;
const NR52_ADDR: usize = 0xFF26;

struct Channel1 {
    frequency: Arc<Mutex<u32>>,
    phase: f32,
    enable: Arc<Mutex<bool>>,
    volume: Arc<Mutex<u8>>,
    duty: Arc<Mutex<f32>>,
    so1_enable: Arc<Mutex<u8>>,
    so1_value: Arc<RwLock<f32>>,
    so2_enable: Arc<Mutex<u8>>,
    so2_value: Arc<RwLock<f32>>,
    all_sound_enable: Arc<RwLock<bool>>,
}

impl AudioCallback for Channel1 {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        let frequency = *self.frequency.lock().unwrap();
        let duty = *self.duty.lock().unwrap();
        let mut right = true;
        let so1_mod = *self.so1_enable.lock().unwrap() as f32 * (*self.so1_value.read().unwrap());
        let so2_mod = *self.so2_enable.lock().unwrap() as f32 * (*self.so2_value.read().unwrap());
        let mut so_mod = so1_mod;
        let enable = *self.enable.lock().unwrap() & *self.all_sound_enable.read().unwrap();
        for x in out.iter_mut() {
            let vol = *self.volume.lock().unwrap();
            if !enable {
                *x = 0.0;
            } else if self.phase < duty {
                *x = 0.0;
            } else {
                *x = (vol as f32 * so_mod) / 100.0;
            }
            right = !right;
            if right {
                self.phase =
                    (self.phase + (131072.0 / (2048.0 - frequency as f32)) / 44100.0) % 1.0;
                so_mod = so1_mod;
            } else {
                so_mod = so2_mod;
            }
        }
    }
}

type Channel2 = Channel1;

struct Channel3 {
    output_level: Arc<Mutex<u8>>,
    pointer: Arc<Mutex<usize>>,
    wave_ram: Arc<Mutex<[u8; 16]>>,
    phase: f32,
    enable: Arc<Mutex<bool>>,
    frequency: Arc<Mutex<u32>>,
    so1_enable: Arc<Mutex<u8>>,
    so1_value: Arc<RwLock<f32>>,
    so2_enable: Arc<Mutex<u8>>,
    so2_value: Arc<RwLock<f32>>,
    all_sound_enable: Arc<RwLock<bool>>,
}

impl AudioCallback for Channel3 {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        let output_shift =
            VOLUME_SHIFT_CONVERSION[((*self.output_level.lock().unwrap() >> 5) & 0b11) as usize];
        let mut right = true;
        let so1_mod = *self.so1_enable.lock().unwrap() as f32 * (*self.so1_value.read().unwrap());
        let so2_mod = *self.so2_enable.lock().unwrap() as f32 * (*self.so2_value.read().unwrap());
        let mut so_mod = so1_mod;
        let enable = *self.enable.lock().unwrap() & *self.all_sound_enable.read().unwrap();
        if !enable {
            for x in out.iter_mut() {
                *x = 0.0;
            }
        } else {
            let mut old_phase = self.phase;
            let mut pointer = self.pointer.lock().unwrap();
            let wave_ram = self.wave_ram.lock().unwrap();
            for x in out.iter_mut() {
                if *pointer % 2 == 0 {
                    *x = ((wave_ram[*pointer / 2] >> 4) >> output_shift) as f32 * so_mod / 100.0;
                } else {
                    *x = ((wave_ram[(*pointer - 1) / 2] & 0xF) >> output_shift) as f32 * so_mod
                        / 100.0;
                }
                right = !right;
                if right {
                    self.phase = (self.phase
                        + CYCLES_PER_SAMPLE as f32
                            / (2048.0 - *self.frequency.lock().unwrap() as f32))
                        % 1.0;
                    if self.phase < old_phase {
                        *pointer = (*pointer + 1) % 32;
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
    lsfr: Arc<Mutex<u16>>,
    enable: Arc<Mutex<bool>>,
    frequency: Arc<Mutex<u32>>,
    width: Arc<Mutex<bool>>,
    volume: Arc<Mutex<u8>>,
    out_1_bit: bool,
    so1_enable: Arc<Mutex<u8>>,
    so1_value: Arc<RwLock<f32>>,
    so2_enable: Arc<Mutex<u8>>,
    so2_value: Arc<RwLock<f32>>,
    all_sound_enable: Arc<RwLock<bool>>,
}

impl Channel4 {
    fn noise_lsfr(&mut self) {
        let mut lsfr = self.lsfr.lock().unwrap();
        let new_bit = (*lsfr & 1) ^ ((*lsfr >> 1) & 1);
        *lsfr >>= 1;
        *lsfr += new_bit << 14;
        if *self.width.lock().unwrap() {
            *lsfr &= 0b0111111;
            *lsfr += new_bit << 6;
        }
        self.out_1_bit = (*lsfr & 1) == 0;
    }
}

impl AudioCallback for Channel4 {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        let enable = *self.enable.lock().unwrap() & *self.all_sound_enable.read().unwrap();
        let volume = *self.volume.lock().unwrap();
        let frequency = *self.frequency.lock().unwrap();
        let mut right = true;
        let so1_mod = *self.so1_enable.lock().unwrap() as f32 * (*self.so1_value.read().unwrap());
        let so2_mod = *self.so2_enable.lock().unwrap() as f32 * (*self.so2_value.read().unwrap());
        let mut so_mod = so1_mod;
        for x in out.iter_mut() {
            let old_phase = self.phase;
            if !enable {
                *x = 0.0;
            } else {
                if self.out_1_bit {
                    *x = volume as f32 * so_mod / 100.0;
                } else {
                    *x = 0.0;
                }
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

        //let phase_add = 1.0 / (frequency >> 3);
    }
}

pub struct AudioProcessingUnit {
    _audio_subsystem: AudioSubsystem,
    channel_1_frequency: Arc<Mutex<u32>>,
    channel_2_frequency: Arc<Mutex<u32>>,
    channel_1_sweep_count: u8,
    channel_1_sweep_enable: bool,
    cycle_count_1: u32,
    cycle_count_2: u32,
    cycle_count_3: u32,
    cycle_count_4: u32,
    so1_level: Arc<RwLock<f32>>,
    so2_level: Arc<RwLock<f32>>,
    nr51_data: u8,
    all_sound_enable: Arc<RwLock<bool>>,
    channel_1_enable: Arc<Mutex<bool>>,
    channel_1_volume: Arc<Mutex<u8>>,
    channel_1_volume_count: u8,
    channel_1_duty: Arc<Mutex<f32>>,
    channel_1_so1_enable: Arc<Mutex<u8>>,
    channel_1_so2_enable: Arc<Mutex<u8>>,

    channel_2_enable: Arc<Mutex<bool>>,
    channel_2_volume: Arc<Mutex<u8>>,
    channel_2_volume_count: u8,
    channel_2_duty: Arc<Mutex<f32>>,
    channel_2_so1_enable: Arc<Mutex<u8>>,
    channel_2_so2_enable: Arc<Mutex<u8>>,

    channel_3_pointer: Arc<Mutex<usize>>,
    channel_3_enable: Arc<Mutex<bool>>,
    channel_3_frequency: Arc<Mutex<u32>>,
    channel_3_output_level: Arc<Mutex<u8>>,
    channel_3_so1_enable: Arc<Mutex<u8>>,
    channel_3_so2_enable: Arc<Mutex<u8>>,

    wave_ram: Arc<Mutex<[u8; 16]>>,
    channel_4_volume_count: u8,
    channel_4_lsfr: Arc<Mutex<u16>>,
    channel_4_enable: Arc<Mutex<bool>>,
    channel_4_frequency: Arc<Mutex<u32>>,
    channel_4_width: Arc<Mutex<bool>>,
    channel_4_volume: Arc<Mutex<u8>>,
    channel_4_so1_enable: Arc<Mutex<u8>>,
    channel_4_so2_enable: Arc<Mutex<u8>>,

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
        let all_sound_enable = Arc::new(RwLock::new(true));

        let channel_1_sweep_count = 0;
        let channel_1_enable = Arc::new(Mutex::new(false));
        let channel_1_volume = Arc::new(Mutex::new(0));
        let channel_1_volume_cb = Arc::clone(&channel_1_volume);
        let channel_1_volume_count = 0;
        let channel_1_sweep_enable = false;
        let channel_1_frequency = Arc::new(Mutex::new(0u32));
        let channel_1_frequency_cb = Arc::clone(&channel_1_frequency);
        let channel_1_enable_cb = Arc::clone(&channel_1_enable);
        let channel_1_duty = Arc::new(Mutex::new(0.0));
        let channel_1_duty_cb = Arc::clone(&channel_1_duty);
        let channel_1_so1_enable = Arc::new(Mutex::new(0));
        let channel_1_so1_enable_cb = Arc::clone(&channel_1_so1_enable);
        let channel_1_so2_enable = Arc::new(Mutex::new(0));
        let channel_1_so2_enable_cb = Arc::clone(&channel_1_so2_enable);
        let channel_1_so1_level_cb = Arc::clone(&so1_level);
        let channel_1_so2_level_cb = Arc::clone(&so2_level);
        let channel_1_all_sound_enable_cb = Arc::clone(&all_sound_enable);

        let channel_2_volume = Arc::new(Mutex::new(0));
        let channel_2_volume_cb = Arc::clone(&channel_2_volume);
        let channel_2_volume_count = 0;
        let channel_2_frequency = Arc::new(Mutex::new(0u32));
        let channel_2_frequency_cb = Arc::clone(&channel_2_frequency);
        let channel_2_enable = Arc::new(Mutex::new(false));
        let channel_2_enable_cb = Arc::clone(&channel_2_enable);
        let channel_2_duty = Arc::new(Mutex::new(0.0));
        let channel_2_duty_cb = Arc::clone(&channel_2_duty);
        let channel_2_so1_enable = Arc::new(Mutex::new(0));
        let channel_2_so1_enable_cb = Arc::clone(&channel_2_so1_enable);
        let channel_2_so2_enable = Arc::new(Mutex::new(0));
        let channel_2_so2_enable_cb = Arc::clone(&channel_2_so2_enable);
        let channel_2_so1_level_cb = Arc::clone(&so1_level);
        let channel_2_so2_level_cb = Arc::clone(&so2_level);
        let channel_2_all_sound_enable_cb = Arc::clone(&all_sound_enable);

        let channel_3_pointer = Arc::new(Mutex::new(0));
        let channel_3_pointer_cb = Arc::clone(&channel_3_pointer);
        let wave_ram = Arc::new(Mutex::new([0; 16]));
        let channel_3_output_level = Arc::new(Mutex::new(0));
        let wave_ram_cb = Arc::clone(&wave_ram);
        let channel_3_output_level_cb = Arc::clone(&channel_3_output_level);
        let channel_3_enable = Arc::new(Mutex::new(false));
        let channel_3_enable_cb = Arc::clone(&channel_3_enable);
        let channel_3_frequency = Arc::new(Mutex::new(0));
        let channel_3_frequency_cb = Arc::clone(&channel_3_frequency);
        let channel_3_so1_enable = Arc::new(Mutex::new(0));
        let channel_3_so1_enable_cb = Arc::clone(&channel_3_so1_enable);
        let channel_3_so2_enable = Arc::new(Mutex::new(0));
        let channel_3_so2_enable_cb = Arc::clone(&channel_3_so2_enable);
        let channel_3_so1_level_cb = Arc::clone(&so1_level);
        let channel_3_so2_level_cb = Arc::clone(&so2_level);
        let channel_3_all_sound_enable_cb = Arc::clone(&all_sound_enable);

        let channel_4_volume_count = 0;
        let channel_4_lsfr = Arc::new(Mutex::new(0u16));
        let channel_4_enable = Arc::new(Mutex::new(false));
        let channel_4_frequency = Arc::new(Mutex::new(0));
        let channel_4_width = Arc::new(Mutex::new(false));
        let channel_4_volume = Arc::new(Mutex::new(0));
        let channel_4_lsfr_cb = Arc::clone(&channel_4_lsfr);
        let channel_4_enable_cb = Arc::clone(&channel_4_enable);
        let channel_4_frequency_cb = Arc::clone(&channel_4_frequency);
        let channel_4_width_cb = Arc::clone(&channel_4_width);
        let channel_4_volume_cb = Arc::clone(&channel_4_volume);
        let channel_4_so1_enable = Arc::new(Mutex::new(0));
        let channel_4_so1_enable_cb = Arc::clone(&channel_4_so1_enable);
        let channel_4_so2_enable = Arc::new(Mutex::new(0));
        let channel_4_so2_enable_cb = Arc::clone(&channel_4_so2_enable);
        let channel_4_so1_level_cb = Arc::clone(&so1_level);
        let channel_4_so2_level_cb = Arc::clone(&so2_level);
        let channel_4_all_sound_enable_cb = Arc::clone(&all_sound_enable);

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2), // mono
            samples: Some(32), // default sample size
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
            so1_level,
            so2_level,
            nr51_data: 0,
            all_sound_enable,
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
        let (volume_reg, channel_volume_count, mut channel_volume) = match channel {
            1 => (
                self.mem_unit.get_memory(NR12_ADDR),
                &mut self.apu.channel_1_volume_count,
                self.apu.channel_1_volume.lock().unwrap(),
            ),
            2 => (
                self.mem_unit.get_memory(NR22_ADDR),
                &mut self.apu.channel_2_volume_count,
                self.apu.channel_2_volume.lock().unwrap(),
            ),
            4 => (
                self.mem_unit.get_memory(NR42_ADDR),
                &mut self.apu.channel_4_volume_count,
                self.apu.channel_4_volume.lock().unwrap(),
            ),
            _ => panic!(
                "Wow, how did you get here? You gave a channel for volume envelope that's bad."
            ),
        };
        let volume_time = volume_reg & 0b111;
        let vol_inc = ((volume_reg >> 3) & 1) == 1;
        if volume_time != 0 && *channel_volume_count >= (volume_time - 1) {
            *channel_volume = if vol_inc && *channel_volume < 15 {
                *channel_volume + 1
            } else if !vol_inc && *channel_volume > 0 {
                *channel_volume - 1
            } else {
                *channel_volume
            };
            *channel_volume_count = 0;
        } else {
            *channel_volume_count = std::cmp::min(*channel_volume_count + 1, 254);
        }
    }
    fn sweep_channel_1(&mut self, frequency: &mut u32) {
        let (sweep_shift, sweep_inc, sweep_time) = {
            let sweep_reg = self.mem_unit.get_memory(NR10_ADDR);
            (sweep_reg & 0b11, (sweep_reg >> 3 & 1) == 0, sweep_reg >> 4)
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
            if *frequency >= 2048 {
                *frequency = old_frequency;
                *self.apu.channel_1_enable.lock().unwrap() = false;
            } else {
                self.mem_unit
                    .write_memory(NR13_ADDR, (*frequency & 0xFF) as u8);
                self.mem_unit.write_memory(
                    NR14_ADDR,
                    (self.mem_unit.get_memory(NR14_ADDR) | 0b11111000)
                        & ((*frequency >> 8) & 0b111) as u8,
                );
            }
        } else {
            self.apu.channel_1_sweep_count = std::cmp::min(self.apu.channel_1_sweep_count + 1, 254);
        }
    }
    fn length_unit_8(&mut self, channel: u8, length: &mut u8) {
        let (cc_reg, length_addr, mut enable_reg) = match channel {
            1 => (
                self.mem_unit.get_memory(NR14_ADDR),
                NR11_ADDR,
                self.apu.channel_1_enable.lock().unwrap(),
            ),
            2 => (
                self.mem_unit.get_memory(NR24_ADDR),
                NR21_ADDR,
                self.apu.channel_2_enable.lock().unwrap(),
            ),
            4 => (
                self.mem_unit.get_memory(NR44_ADDR),
                NR41_ADDR,
                self.apu.channel_4_enable.lock().unwrap(),
            ),
            _ => {
                panic!("Wow, how did you get here? You gave a channel for length unit that's bad.")
            }
        };
        let counter_consec = (cc_reg >> 6) & 1;
        if counter_consec == 1 {
            if *length <= 1 {
                *enable_reg = false;
            } else {
                self.mem_unit.write_memory(
                    length_addr,
                    self.mem_unit.get_memory(length_addr) & 0b11000000 | (64 - *length),
                );
                *length -= 1;
            }
        }
    }

    fn length_unit_16(&mut self, length: &mut u16) {
        let cc_reg = self.mem_unit.get_memory(NR34_ADDR);
        let mut enable_reg = self.apu.channel_3_enable.lock().unwrap();
        let counter_consec = cc_reg >> 6 & 1;
        if counter_consec == 1 {
            if *length == 0 {
                *enable_reg = false;
            } else {
                self.mem_unit.write_memory(
                    NR31_ADDR,
                    self.mem_unit.get_memory(NR31_ADDR) & 0b11000000 | (256 - *length) as u8,
                );
                *length -= 1;
            }
        }
    }

    fn channel_1_advance(&mut self) {
        let initialize = (self.mem_unit.get_memory(NR14_ADDR) >> 7) == 1;
        let mut length = 64 - (self.mem_unit.get_memory(NR11_ADDR) & 0b111111);

        *self.apu.channel_1_so1_enable.lock().unwrap() = self.apu.nr51_data & 1;
        *self.apu.channel_1_so2_enable.lock().unwrap() = (self.apu.nr51_data >> 4) & 1;
        if initialize {
            if length == 0 {
                length = 64;
            }
            *self.apu.channel_1_enable.lock().unwrap() = true;
            self.apu.channel_1_sweep_count = 0;
            self.apu.channel_1_volume_count = 0;
            *self.apu.channel_1_volume.lock().unwrap() = self.mem_unit.get_memory(NR12_ADDR) >> 4;
            let (sweep_shift, sweep_time) = {
                let sweep_reg = self.mem_unit.get_memory(NR10_ADDR);
                (sweep_reg & 0b11, sweep_reg >> 4)
            };
            self.apu.channel_1_sweep_enable = if sweep_shift == 0 || sweep_time == 0 {
                false
            } else {
                true
            };
            self.apu.cycle_count_1 = 0;
            self.mem_unit
                .write_memory(NR14_ADDR, self.mem_unit.get_memory(NR14_ADDR) & 0b01111111);
        }
        if *self.apu.channel_1_enable.lock().unwrap() {
            *self.apu.channel_1_duty.lock().unwrap() =
                DUTY_CONVERSION[((self.mem_unit.get_memory(NR11_ADDR) >> 6) & 0b11) as usize];
            let mut channel_1_frequency = (((self.mem_unit.get_memory(NR14_ADDR) & 0b111) as u32)
                << 8)
                + self.mem_unit.get_memory(NR13_ADDR) as u32;

            if self.apu.cycle_count_1 % 32768 == 0 && self.apu.channel_1_sweep_enable {
                self.sweep_channel_1(&mut channel_1_frequency);
            }
            if self.apu.cycle_count_1 == 0 {
                self.volume_envelope(1);
            }

            if self.apu.cycle_count_1 % 16384 == 0 {
                self.length_unit_8(1, &mut length);
            }
            *self.apu.channel_1_frequency.lock().unwrap() = channel_1_frequency;
        }
    }
    fn channel_2_advance(&mut self) {
        let initialize = (self.mem_unit.get_memory(NR24_ADDR) >> 7) == 1;
        let mut length = 64 - (self.mem_unit.get_memory(NR21_ADDR) & 0b111111);
        *self.apu.channel_2_so1_enable.lock().unwrap() = (self.apu.nr51_data >> 1) & 1;
        *self.apu.channel_2_so2_enable.lock().unwrap() = (self.apu.nr51_data >> 5) & 1;
        if initialize {
            if length == 0 {
                length = 64;
            }
            *self.apu.channel_2_enable.lock().unwrap() = true;
            self.apu.channel_2_volume_count = 0;
            *self.apu.channel_2_volume.lock().unwrap() = self.mem_unit.get_memory(NR22_ADDR) >> 4;
            self.mem_unit
                .write_memory(NR24_ADDR, self.mem_unit.get_memory(NR24_ADDR) & 0b01111111);
            self.apu.cycle_count_2 = 0;
        }
        *self.apu.channel_2_duty.lock().unwrap() =
            DUTY_CONVERSION[((self.mem_unit.get_memory(NR21_ADDR) >> 6) & 0b11) as usize];
        *self.apu.channel_2_frequency.lock().unwrap() =
            (((self.mem_unit.get_memory(NR24_ADDR) & 0b111) as u32) << 8)
                + self.mem_unit.get_memory(NR23_ADDR) as u32;

        if self.apu.cycle_count_2 == 0 {
            self.volume_envelope(2);
        }

        if self.apu.cycle_count_2 % 16384 == 0 {
            self.length_unit_8(2, &mut length);
        }
    }

    fn channel_3_advance(&mut self) {
        let initialize = (self.mem_unit.get_memory(NR34_ADDR) >> 7) == 1;
        let mut length = 256 - self.mem_unit.get_memory(NR31_ADDR) as u16 & 0b11111;
        *self.apu.channel_3_so1_enable.lock().unwrap() = (self.apu.nr51_data >> 2) & 1;
        *self.apu.channel_3_so2_enable.lock().unwrap() = (self.apu.nr51_data >> 6) & 1;
        if initialize {
            if length == 0 {
                length = 256;
            }
            self.apu.cycle_count_3 = 0;
            *self.apu.channel_3_pointer.lock().unwrap() = 0;
            self.mem_unit
                .write_memory(NR34_ADDR, self.mem_unit.get_memory(NR34_ADDR) & 0b01111111);
            *self.apu.channel_3_enable.lock().unwrap() = true;
        }
        if self.mem_unit.get_memory(NR30_ADDR) >> 7 == 0 {
            *self.apu.channel_3_enable.lock().unwrap() = false;
        }
        *self.apu.wave_ram.lock().unwrap() = self.mem_unit.get_wave_ram();
        *self.apu.channel_3_output_level.lock().unwrap() = self.mem_unit.get_memory(NR32_ADDR);
        *self.apu.channel_3_frequency.lock().unwrap() =
            (((self.mem_unit.get_memory(NR34_ADDR) & 0b111) as u32) << 8)
                + self.mem_unit.get_memory(NR33_ADDR) as u32;
        if self.apu.cycle_count_3 % 16384 == 0 {
            self.length_unit_16(&mut length);
        }
    }
    fn channel_4_advance(&mut self) {
        let initialize = (self.mem_unit.get_memory(NR44_ADDR) >> 7) == 1;
        let mut length = 64 - (self.mem_unit.get_memory(NR41_ADDR) & 0b11111);
        *self.apu.channel_4_so1_enable.lock().unwrap() = (self.apu.nr51_data >> 3) & 1;
        *self.apu.channel_4_so2_enable.lock().unwrap() = (self.apu.nr51_data >> 7) & 1;
        if initialize {
            if length == 0 {
                length = 64;
            }
            self.mem_unit
                .write_memory(NR44_ADDR, self.mem_unit.get_memory(NR44_ADDR) & 0b01111111);
            self.apu.channel_4_volume_count = 0;
            *self.apu.channel_4_volume.lock().unwrap() = self.mem_unit.get_memory(NR42_ADDR) >> 4;
            *self.apu.channel_4_lsfr.lock().unwrap() = 0x7FFF;
            *self.apu.channel_4_enable.lock().unwrap() = true;
            self.apu.cycle_count_4 = 0;
        }
        let poly_counter_reg = self.mem_unit.get_memory(NR43_ADDR);
        *self.apu.channel_4_width.lock().unwrap() = ((poly_counter_reg >> 3) & 1) == 1;
        *self.apu.channel_4_frequency.lock().unwrap() = {
            let shift_clock_freq = poly_counter_reg >> 4;
            let freq_divider = {
                let possible_ratio = poly_counter_reg & 0b111;
                let s_factor: u32 = 1 << (shift_clock_freq + 4);
                if possible_ratio == 0 {
                    s_factor >> 1
                } else {
                    s_factor * possible_ratio as u32
                }
            };
            freq_divider
        };
        if self.apu.cycle_count_4 == 0 {
            self.volume_envelope(4);
        }

        if self.apu.cycle_count_4 % 16384 == 0 {
            self.length_unit_8(4, &mut length);
        }
    }
    pub fn apu_advance(&mut self) {
        *self.apu.so1_level.write().unwrap() =
            ((self.mem_unit.get_memory(NR50_ADDR) >> 4) & 0b11) as f32 / 7.0;
        *self.apu.so2_level.write().unwrap() =
            (self.mem_unit.get_memory(NR50_ADDR) & 0b11) as f32 / 7.0;
        self.apu.nr51_data = self.mem_unit.get_memory(NR51_ADDR);
        *self.apu.all_sound_enable.write().unwrap() =
            (self.mem_unit.get_memory(NR52_ADDR) >> 7) == 1;
        self.channel_1_advance();
        self.channel_2_advance();
        self.channel_3_advance();
        self.channel_4_advance();
        self.apu.cycle_count_1 = (self.apu.cycle_count_1 + ADVANCE_CYCLES) % 65536;
        self.apu.cycle_count_2 = (self.apu.cycle_count_2 + ADVANCE_CYCLES) % 65536;
        self.apu.cycle_count_3 = (self.apu.cycle_count_3 + ADVANCE_CYCLES) % 65536;
        self.apu.cycle_count_4 = (self.apu.cycle_count_4 + ADVANCE_CYCLES) % 65536;
    }
}

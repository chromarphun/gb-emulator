use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::AudioSubsystem;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use crate::{ADVANCE_CYCLES, CYCLES_PER_SAMPLE};

const CLOCK: u32 = 1_048_576;

const DUTY_CONVERSION: [f32; 4] = [0.125, 0.25, 0.5, 0.75];

const VOLUME_SHIFT_CONVERSION: [u8; 4] = [4, 0, 1, 2];

struct Channel1 {
    frequency: Arc<Mutex<u32>>,
    phase: f32,
    enable: Arc<Mutex<bool>>,
    volume: Arc<Mutex<u8>>,
    duty: Arc<Mutex<f32>>,
}

impl AudioCallback for Channel1 {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        let frequency = *self.frequency.lock().unwrap();
        let duty = *self.duty.lock().unwrap();
        for x in out.iter_mut() {
            let vol = *self.volume.lock().unwrap();
            if !*self.enable.lock().unwrap() {
                *x = 0.0;
            } else if self.phase < duty {
                *x = 0.0;
            } else {
                *x = (vol as f32) / 100.0;
            }
            self.phase = (self.phase + (131072.0 / (2048.0 - frequency as f32)) / 44100.0) % 1.0;
        }
    }
}

struct Channel2 {
    frequency: Arc<Mutex<u32>>,
    phase: f32,
    enable: Arc<Mutex<bool>>,
    volume: Arc<Mutex<u8>>,
    duty: Arc<Mutex<f32>>,
}

impl AudioCallback for Channel2 {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        let frequency = *self.frequency.lock().unwrap();
        let duty = *self.duty.lock().unwrap();
        for x in out.iter_mut() {
            let vol = *self.volume.lock().unwrap();
            if !*self.enable.lock().unwrap() {
                *x = 0.0;
            } else if self.phase < duty {
                *x = 0.0;
            } else {
                *x = (vol as f32) / 100.0;
            }
            self.phase = (self.phase + (131072.0 / (2048.0 - frequency as f32)) / 44100.0) % 1.0;
        }
    }
}

struct Channel3 {
    output_level: Arc<Mutex<u8>>,
    pointer: Arc<Mutex<usize>>,
    wave_ram: Arc<Mutex<[u8; 16]>>,
    phase: f32,
    enable: Arc<Mutex<bool>>,
    frequency: Arc<Mutex<u32>>,
}

impl AudioCallback for Channel3 {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        let output_shift =
            VOLUME_SHIFT_CONVERSION[((*self.output_level.lock().unwrap() >> 5) & 0b11) as usize];
        if !*self.enable.lock().unwrap() {
            for x in out.iter_mut() {
                *x = 0.0;
            }
        } else {
            let mut old_phase = self.phase;
            let mut pointer = self.pointer.lock().unwrap();
            let wave_ram = self.wave_ram.lock().unwrap();
            for x in out.iter_mut() {
                if *pointer % 2 == 0 {
                    *x = ((wave_ram[*pointer / 2] >> 4) >> output_shift) as f32 / 100.0;
                } else {
                    *x = ((wave_ram[(*pointer - 1) / 2] & 0xF) >> output_shift) as f32 / 100.0;
                }
                self.phase = (self.phase
                    + CYCLES_PER_SAMPLE as f32 / (2048.0 - *self.frequency.lock().unwrap() as f32))
                    % 1.0;
                if self.phase < old_phase {
                    *pointer = (*pointer + 1) % 32;
                }
                old_phase = self.phase;
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
        let enable = *self.enable.lock().unwrap();
        let volume = *self.volume.lock().unwrap();

        let frequency = *self.frequency.lock().unwrap();
        for x in out.iter_mut() {
            let old_phase = self.phase;
            if !enable {
                *x = 0.0;
            } else {
                if self.out_1_bit {
                    *x = volume as f32 / 100.0;
                } else {
                    *x = 0.0;
                }
            }
            let phase_add = CYCLES_PER_SAMPLE as f32 / frequency as f32;
            self.phase = (self.phase + phase_add) % 1.0;

            if self.phase < old_phase {
                self.noise_lsfr();
            } else if phase_add >= 1.0 {
                for _ in 0..phase_add as u8 {
                    self.noise_lsfr();
                }
            }
        }

        //let phase_add = 1.0 / (frequency >> 3);
    }
}

pub struct AudioProcessingUnit {
    audio_subsystem: AudioSubsystem,
    nr10: Arc<Mutex<u8>>,
    nr11: Arc<Mutex<u8>>,
    nr12: Arc<Mutex<u8>>,
    nr13: Arc<Mutex<u8>>,
    nr14: Arc<Mutex<u8>>,
    nr21: Arc<Mutex<u8>>,
    nr22: Arc<Mutex<u8>>,
    nr23: Arc<Mutex<u8>>,
    nr24: Arc<Mutex<u8>>,
    nr30: Arc<Mutex<u8>>,
    nr31: Arc<Mutex<u8>>,
    nr32: Arc<Mutex<u8>>,
    nr33: Arc<Mutex<u8>>,
    nr34: Arc<Mutex<u8>>,
    wave_ram: Arc<Mutex<[u8; 16]>>,
    nr41: Arc<Mutex<u8>>,
    nr42: Arc<Mutex<u8>>,
    nr43: Arc<Mutex<u8>>,
    nr44: Arc<Mutex<u8>>,
    channel_1_frequency: Arc<Mutex<u32>>,
    channel_2_frequency: Arc<Mutex<u32>>,
    channel_1_sweep_count: u8,
    channel_1_sweep_enable: bool,
    cycle_count_1: u32,
    cycle_count_2: u32,
    cycle_count_3: u32,
    cycle_count_4: u32,
    channel_1_enable: Arc<Mutex<bool>>,
    channel_1_volume: Arc<Mutex<u8>>,
    channel_1_volume_count: u8,
    channel_1_duty: Arc<Mutex<f32>>,
    channel_2_enable: Arc<Mutex<bool>>,
    channel_2_volume: Arc<Mutex<u8>>,
    channel_2_volume_count: u8,
    channel_2_duty: Arc<Mutex<f32>>,
    channel_3_pointer: Arc<Mutex<usize>>,
    channel_3_enable: Arc<Mutex<bool>>,
    channel_3_frequency: Arc<Mutex<u32>>,
    channel_4_volume_count: u8,
    channel_4_lsfr: Arc<Mutex<u16>>,
    channel_4_enable: Arc<Mutex<bool>>,
    channel_4_frequency: Arc<Mutex<u32>>,
    channel_4_width: Arc<Mutex<bool>>,
    channel_4_volume: Arc<Mutex<u8>>,
    channel_1_device: AudioDevice<Channel1>,
    channel_2_device: AudioDevice<Channel2>,
    channel_3_device: AudioDevice<Channel3>,
    channel_4_device: AudioDevice<Channel4>,
}

impl AudioProcessingUnit {
    pub fn new(
        audio_subsystem: AudioSubsystem,
        nr10: Arc<Mutex<u8>>,
        nr11: Arc<Mutex<u8>>,
        nr12: Arc<Mutex<u8>>,
        nr13: Arc<Mutex<u8>>,
        nr14: Arc<Mutex<u8>>,
        nr21: Arc<Mutex<u8>>,
        nr22: Arc<Mutex<u8>>,
        nr23: Arc<Mutex<u8>>,
        nr24: Arc<Mutex<u8>>,
        nr30: Arc<Mutex<u8>>,
        nr31: Arc<Mutex<u8>>,
        nr32: Arc<Mutex<u8>>,
        nr33: Arc<Mutex<u8>>,
        nr34: Arc<Mutex<u8>>,
        wave_ram: Arc<Mutex<[u8; 16]>>,
        nr41: Arc<Mutex<u8>>,
        nr42: Arc<Mutex<u8>>,
        nr43: Arc<Mutex<u8>>,
        nr44: Arc<Mutex<u8>>,
    ) -> AudioProcessingUnit {
        let cycle_count_1 = 0;
        let cycle_count_2 = 0;
        let cycle_count_3 = 0;
        let cycle_count_4 = 0;
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

        let channel_2_volume = Arc::new(Mutex::new(0));
        let channel_2_volume_cb = Arc::clone(&channel_2_volume);
        let channel_2_volume_count = 0;
        let channel_2_frequency = Arc::new(Mutex::new(0u32));
        let channel_2_frequency_cb = Arc::clone(&channel_2_frequency);
        let channel_2_enable = Arc::new(Mutex::new(false));
        let channel_2_enable_cb = Arc::clone(&channel_2_enable);
        let channel_2_duty = Arc::new(Mutex::new(0.0));
        let channel_2_duty_cb = Arc::clone(&channel_2_duty);

        let channel_3_pointer = Arc::new(Mutex::new(0));
        let channel_3_pointer_cb = Arc::clone(&channel_3_pointer);
        let wave_ram_cb = Arc::clone(&wave_ram);
        let channel_3_output_level = Arc::clone(&nr32);
        let channel_3_enable = Arc::new(Mutex::new(false));
        let channel_3_enable_cb = Arc::clone(&channel_3_enable);

        let channel_3_frequency = Arc::new(Mutex::new(0));
        let channel_3_frequency_cb = Arc::clone(&channel_3_frequency);

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

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: Some(32), // default sample size
        };
        let exp_spec = AudioSpecDesired {
            freq: Some(524288),
            channels: Some(1),  // mono
            samples: Some(256), // default sample size
        };
        let channel_1_device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                // initialize the audio callback
                Channel1 {
                    phase: 0.0,
                    frequency: channel_1_frequency_cb,
                    enable: channel_1_enable_cb,
                    volume: channel_1_volume_cb,
                    duty: channel_1_duty_cb,
                }
            })
            .unwrap();

        let channel_2_device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                // initialize the audio callback
                Channel2 {
                    phase: 0.0,
                    frequency: channel_2_frequency_cb,
                    enable: channel_2_enable_cb,
                    volume: channel_2_volume_cb,
                    duty: channel_2_duty_cb,
                }
            })
            .unwrap();

        let channel_3_device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                // initialize the audio callback
                Channel3 {
                    output_level: channel_3_output_level,
                    pointer: channel_3_pointer_cb,
                    wave_ram: wave_ram_cb,
                    phase: 0.0,
                    enable: channel_3_enable_cb,
                    frequency: channel_3_frequency_cb,
                }
            })
            .unwrap();
        let channel_4_device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                // initialize the audio callback
                Channel4 {
                    phase: 0.0,
                    lsfr: channel_4_lsfr_cb,
                    enable: channel_4_enable_cb,
                    frequency: channel_4_frequency_cb,
                    width: channel_4_width_cb,
                    volume: channel_4_volume_cb,
                    out_1_bit: true,
                }
            })
            .unwrap();
        channel_1_device.resume();
        channel_2_device.resume();
        channel_3_device.resume();
        channel_4_device.resume();
        AudioProcessingUnit {
            audio_subsystem,
            channel_1_frequency,
            channel_2_frequency,
            nr10,
            nr11,
            nr12,
            nr13,
            nr14,
            nr21,
            nr22,
            nr23,
            nr24,
            nr30,
            nr31,
            nr32,
            nr33,
            nr34,
            wave_ram,
            nr41,
            nr42,
            nr43,
            nr44,
            cycle_count_1,
            cycle_count_2,
            cycle_count_3,
            cycle_count_4,
            channel_1_sweep_count,
            channel_1_sweep_enable,
            channel_1_enable,
            channel_1_volume,
            channel_1_volume_count,
            channel_1_duty,
            channel_2_enable,
            channel_2_volume,
            channel_2_volume_count,
            channel_2_duty,
            channel_3_pointer,
            channel_3_enable,
            channel_3_frequency,
            channel_4_volume_count,
            channel_4_lsfr,
            channel_4_enable,
            channel_4_frequency,
            channel_4_width,
            channel_4_volume,
            channel_1_device,
            channel_2_device,
            channel_3_device,
            channel_4_device,
        }
    }

    fn volume_envelope(&mut self, channel: u8) {
        let (volume_reg, channel_volume_count, mut channel_volume) = match channel {
            1 => (
                self.nr12.lock().unwrap(),
                &mut self.channel_1_volume_count,
                self.channel_1_volume.lock().unwrap(),
            ),
            2 => (
                self.nr22.lock().unwrap(),
                &mut self.channel_2_volume_count,
                self.channel_2_volume.lock().unwrap(),
            ),
            4 => (
                self.nr42.lock().unwrap(),
                &mut self.channel_4_volume_count,
                self.channel_4_volume.lock().unwrap(),
            ),
            _ => panic!(
                "Wow, how did you get here? You gave a channel for volume envelope that's bad."
            ),
        };
        let volume_time = *volume_reg & 0b111;
        let vol_inc = ((*volume_reg >> 3) & 1) == 1;
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
            let sweep_reg = *self.nr10.lock().unwrap();
            (sweep_reg & 0b11, (sweep_reg >> 3 & 1) == 0, sweep_reg >> 4)
        };
        if sweep_time != 0 && self.channel_1_sweep_count >= (sweep_time - 1) && sweep_shift != 0 {
            let old_frequency = *frequency;
            *frequency = if sweep_inc {
                *frequency + (*frequency >> sweep_shift)
            } else {
                *frequency - (*frequency >> sweep_shift)
            };
            self.channel_1_sweep_count = 0;
            if *frequency >= 2048 {
                *frequency = old_frequency;
                *self.channel_1_enable.lock().unwrap() = false;
            } else {
                *self.nr13.lock().unwrap() = (*frequency & 0xFF) as u8;
                *self.nr14.lock().unwrap() &= 0b11111000;
                *self.nr14.lock().unwrap() |= ((*frequency >> 8) & 0b111) as u8;
            }
        } else {
            self.channel_1_sweep_count = std::cmp::min(self.channel_1_sweep_count + 1, 254);
        }
    }
    fn length_unit_8(&mut self, channel: u8, length: &mut u8) {
        let (cc_reg, mut length_reg, mut enable_reg) = match channel {
            1 => (
                self.nr14.lock().unwrap(),
                self.nr11.lock().unwrap(),
                self.channel_1_enable.lock().unwrap(),
            ),
            2 => (
                self.nr24.lock().unwrap(),
                self.nr21.lock().unwrap(),
                self.channel_2_enable.lock().unwrap(),
            ),
            4 => (
                self.nr44.lock().unwrap(),
                self.nr41.lock().unwrap(),
                self.channel_4_enable.lock().unwrap(),
            ),
            _ => {
                panic!("Wow, how did you get here? You gave a channel for length unit that's bad.")
            }
        };
        let counter_consec = (*cc_reg >> 6) & 1;
        if counter_consec == 1 {
            if *length <= 1 {
                *enable_reg = false;
            } else {
                *length_reg &= 0b11000000;
                *length -= 1;
                *length_reg |= 64 - *length;
            }
        }
    }

    fn length_unit_16(&mut self, length: &mut u16) {
        let cc_reg = self.nr34.lock().unwrap();
        let mut length_reg = self.nr31.lock().unwrap();
        let mut enable_reg = self.channel_3_enable.lock().unwrap();
        let counter_consec = *cc_reg >> 6 & 1;
        if counter_consec == 1 {
            if *length == 0 {
                *enable_reg = false;
            } else {
                *length_reg &= 0b11000000;
                *length -= 1;
                *length_reg |= (256 - *length) as u8;
            }
        }
    }

    fn channel_1_advance(&mut self) {
        let initialize = (*self.nr14.lock().unwrap() >> 7) == 1;
        let mut length = 64 - (*self.nr11.lock().unwrap() & 0b111111);
        if initialize {
            if length == 0 {
                length = 64;
            }
            *self.channel_1_enable.lock().unwrap() = true;
            self.channel_1_sweep_count = 0;
            self.channel_1_volume_count = 0;
            *self.channel_1_volume.lock().unwrap() = *self.nr12.lock().unwrap() >> 4;
            let (sweep_shift, sweep_time) = {
                let sweep_reg = *self.nr10.lock().unwrap();
                (sweep_reg & 0b11, sweep_reg >> 4)
            };
            self.channel_1_sweep_enable = if sweep_shift == 0 || sweep_time == 0 {
                false
            } else {
                true
            };
            self.cycle_count_1 = 0;
            *self.nr14.lock().unwrap() &= 0b01111111;
        }
        if *self.channel_1_enable.lock().unwrap() {
            *self.channel_1_duty.lock().unwrap() =
                DUTY_CONVERSION[((*self.nr11.lock().unwrap() >> 6) & 0b11) as usize];
            let duty = *self.channel_1_duty.lock().unwrap();
            let mut channel_1_frequency = (((*self.nr14.lock().unwrap() & 0b111) as u32) << 8)
                + *self.nr13.lock().unwrap() as u32;

            if self.cycle_count_1 % 32768 == 0 && self.channel_1_sweep_enable {
                self.sweep_channel_1(&mut channel_1_frequency);
            }
            if self.cycle_count_1 == 0 {
                self.volume_envelope(1);
            }

            if self.cycle_count_1 % 16384 == 0 {
                self.length_unit_8(1, &mut length);
            }
            *self.channel_1_frequency.lock().unwrap() = channel_1_frequency;
        }
    }
    fn channel_2_advance(&mut self) {
        let initialize = (*self.nr24.lock().unwrap() >> 7) == 1;
        let mut length = 64 - (*self.nr21.lock().unwrap() & 0b111111);
        if initialize {
            if length == 0 {
                length = 64;
            }
            *self.channel_2_enable.lock().unwrap() = true;
            self.channel_2_volume_count = 0;
            *self.channel_2_volume.lock().unwrap() = *self.nr22.lock().unwrap() >> 4;
            *self.nr24.lock().unwrap() &= 0b01111111;
            self.cycle_count_2 = 0;
        }
        *self.channel_2_duty.lock().unwrap() =
            DUTY_CONVERSION[((*self.nr21.lock().unwrap() >> 6) & 0b11) as usize];
        *self.channel_2_frequency.lock().unwrap() = (((*self.nr24.lock().unwrap() & 0b111) as u32)
            << 8)
            + *self.nr23.lock().unwrap() as u32;

        if self.cycle_count_2 == 0 {
            self.volume_envelope(2);
        }

        if self.cycle_count_2 % 16384 == 0 {
            self.length_unit_8(2, &mut length);
        }
    }

    fn channel_3_advance(&mut self) {
        let initialize = (*self.nr34.lock().unwrap() >> 7) == 1;
        let mut length = 256 - *self.nr31.lock().unwrap() as u16 & 0b11111;
        if initialize {
            if length == 0 {
                length = 256;
            }
            self.cycle_count_3 = 0;
            *self.channel_3_pointer.lock().unwrap() = 0;
            *self.nr34.lock().unwrap() &= 0b01111111;
            *self.channel_3_enable.lock().unwrap() = true;
        }
        if *self.nr30.lock().unwrap() >> 7 == 0 {
            *self.channel_3_enable.lock().unwrap() = false;
        }
        *self.channel_3_frequency.lock().unwrap() = (((*self.nr34.lock().unwrap() & 0b111) as u32)
            << 8)
            + *self.nr33.lock().unwrap() as u32;
        if self.cycle_count_3 % 16384 == 0 {
            self.length_unit_16(&mut length);
        }
    }
    fn channel_4_advance(&mut self) {
        let initialize = (*self.nr44.lock().unwrap() >> 7) == 1;
        let mut length = 64 - (*self.nr41.lock().unwrap() & 0b11111);
        if initialize {
            if length == 0 {
                length = 64;
            }
            *self.nr44.lock().unwrap() &= 0b01111111;
            self.channel_4_volume_count = 0;
            *self.channel_4_volume.lock().unwrap() = *self.nr42.lock().unwrap() >> 4;
            *self.channel_4_lsfr.lock().unwrap() = 0x7FFF;
            *self.channel_4_enable.lock().unwrap() = true;
            self.cycle_count_4 = 0;
        }
        let poly_counter_reg = *self.nr43.lock().unwrap();
        *self.channel_4_width.lock().unwrap() = ((poly_counter_reg >> 3) & 1) == 1;
        *self.channel_4_frequency.lock().unwrap() = {
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
        if self.cycle_count_4 == 0 {
            self.volume_envelope(4);
        }

        if self.cycle_count_4 % 16384 == 0 {
            self.length_unit_8(4, &mut length);
        }
    }
    pub fn advance(&mut self) {
        self.channel_1_advance();
        self.channel_2_advance();
        self.channel_3_advance();
        self.channel_4_advance();
        self.cycle_count_1 = (self.cycle_count_1 + ADVANCE_CYCLES) % 65536;
        self.cycle_count_2 = (self.cycle_count_2 + ADVANCE_CYCLES) % 65536;
        self.cycle_count_3 = (self.cycle_count_3 + ADVANCE_CYCLES) % 65536;
        self.cycle_count_4 = (self.cycle_count_4 + ADVANCE_CYCLES) % 65536;
    }
}

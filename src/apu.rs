use crate::constants::*;
use crate::emulator::GameBoyEmulator;
use crate::emulator::RequestSource;
use sdl2::audio::AudioQueue;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::AudioSubsystem;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
const SOURCE: RequestSource = RequestSource::APU;

pub struct AudioProcessingUnit {
    _audio_subsystem: AudioSubsystem,
    pub length_counters: [u16; 4],
    length_enables: [bool; 4],
    sequence_counter: u8,
    initial_volumes: [u8; 4],
    vol_inc_flags: [bool; 4],
    vol_periods: [u8; 4],
    volumes: [u8; 4],
    vol_timers: [u8; 4],
    pub apu_power: bool,
    pub buffering: bool,
    channel_1_frequency: u32,
    channel_1_shadow_frequency: u32,
    channel_2_frequency: u32,
    channel_1_sweep_timer: u8,
    channel_1_sweep_enable: bool,
    channel_1_sweep_inc: bool,
    channel_1_sweep_period: u8,
    channel_1_sweep_shift: u8,
    channel_1_neg_after_trigger: bool,
    channel_1_phase: f32,
    pub ch_1_queue: AudioQueue<f32>,
    ch_1_so1_buffer: [f32; AUDIO_BUFFER_SIZE],
    ch_1_so2_buffer: [f32; AUDIO_BUFFER_SIZE],
    ch_1_queue_data: [f32; AUDIO_QUEUE_SIZE],
    ch_1_buffer_pointer: usize,
    cycle_count: u32,

    so1_level: f32,
    so2_level: f32,
    pub all_sound_enable: bool,
    channel_1_enable: bool,

    channel_1_duty: f32,
    channel_1_so1_enable: u8,
    channel_1_so2_enable: u8,

    channel_2_enable: bool,

    channel_2_duty: f32,
    channel_2_so1_enable: u8,
    channel_2_so2_enable: u8,

    channel_3_pointer: usize,
    channel_3_enable: bool,
    channel_3_frequency: u32,
    pub channel_3_output_level: u8,
    channel_3_so1_enable: u8,
    channel_3_so2_enable: u8,

    pub wave_ram: [u8; 16],
    channel_4_lsfr: u16,
    channel_4_enable: bool,
    channel_4_frequency: u32,
    channel_4_width: bool,
    channel_4_so1_enable: u8,
    channel_4_so2_enable: u8,
}

impl AudioProcessingUnit {
    pub fn new(audio_subsystem: AudioSubsystem) -> AudioProcessingUnit {
        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLES_PER_SECOND as i32),
            channels: Some(2),
            samples: Some(256),
        };
        let ch_1_queue = audio_subsystem.open_queue(None, &desired_spec).unwrap();

        AudioProcessingUnit {
            _audio_subsystem: audio_subsystem,
            length_counters: [0; 4],
            length_enables: [false; 4],
            sequence_counter: 0,
            initial_volumes: [0; 4],
            vol_inc_flags: [false; 4],
            vol_periods: [0; 4],
            volumes: [0; 4],
            vol_timers: [1; 4],
            apu_power: false,
            buffering: false,
            channel_1_frequency: 0,
            channel_1_shadow_frequency: 0,
            channel_2_frequency: 0,
            channel_1_sweep_timer: 1,
            channel_1_sweep_enable: false,
            channel_1_sweep_inc: false,
            channel_1_sweep_period: 0,
            channel_1_sweep_shift: 0,
            channel_1_neg_after_trigger: false,
            channel_1_phase: 0.0,
            ch_1_queue,
            ch_1_so1_buffer: [0.0; AUDIO_BUFFER_SIZE],
            ch_1_so2_buffer: [0.0; AUDIO_BUFFER_SIZE],
            ch_1_queue_data: [0.0; AUDIO_QUEUE_SIZE],
            ch_1_buffer_pointer: 0,
            cycle_count: 0,

            so1_level: 0.0,
            so2_level: 0.0,
            all_sound_enable: true,
            channel_1_enable: false,

            channel_1_duty: 0.0,
            channel_1_so1_enable: 0,
            channel_1_so2_enable: 0,

            channel_2_enable: false,

            channel_2_duty: 0.0,
            channel_2_so1_enable: 0,
            channel_2_so2_enable: 0,

            channel_3_pointer: 0,
            channel_3_enable: false,
            channel_3_frequency: 0,
            channel_3_output_level: 0,
            channel_3_so1_enable: 0,
            channel_3_so2_enable: 0,

            wave_ram: [0; 16],
            channel_4_lsfr: 0,
            channel_4_enable: false,
            channel_4_frequency: 0,
            channel_4_width: false,
            channel_4_so1_enable: 0,
            channel_4_so2_enable: 0,
        }
    }
}
impl GameBoyEmulator {
    pub fn disable_channel(&mut self, channel: usize) {
        let (enable_channel, mask) = match channel {
            1 => (&mut self.apu.channel_1_enable, 0b11111110),
            2 => (&mut self.apu.channel_2_enable, 0b11111101),
            3 => (&mut self.apu.channel_3_enable, 0b11111011),
            4 => (&mut self.apu.channel_4_enable, 0b11110111),
            _ => panic!("Wow, how did you get here? You gave a channel for disable that's bad."),
        };
        (*enable_channel) = false;
        self.write_memory(NR52_ADDR, self.get_memory(NR52_ADDR, SOURCE) & mask, SOURCE);
    }
    fn enable_channel(&mut self, channel: usize) {
        let (enable_channel, mask) = match channel {
            1 => (&mut self.apu.channel_1_enable, 0b00000001),
            2 => (&mut self.apu.channel_2_enable, 0b00000010),
            3 => (&mut self.apu.channel_3_enable, 0b00000100),
            4 => (&mut self.apu.channel_4_enable, 0b00001000),
            _ => panic!("Wow, how did you get here? You gave a channel for enable that's bad."),
        };
        (*enable_channel) = true;
        self.write_memory(NR52_ADDR, self.get_memory(NR52_ADDR, SOURCE) | mask, SOURCE);
    }
    pub fn apu_power_up(&mut self) {
        self.apu.cycle_count = 0;
        self.apu.sequence_counter = 0;
        self.apu.all_sound_enable = true;
        self.apu.apu_power = true;
    }

    fn update_frequency_addr(&mut self, channel: usize) {
        let (low_reg_addr, high_reg_addr, frequency_val) = match channel {
            1 => (NR13_ADDR, NR14_ADDR, self.apu.channel_1_frequency),
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
        let mut new_freq = (*freq_atomic);
        new_freq &= 0x700;
        new_freq |= val as u32;
        (*freq_atomic) = new_freq;
        if channel == 1 && !self.apu.channel_1_sweep_enable {
            self.apu.channel_1_shadow_frequency = new_freq;
        }
    }
    fn volume_unit(&mut self, channel: usize) {
        let current_vol = self.apu.volumes[channel - 1];
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
        self.apu.volumes[channel - 1] = new_vol;
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
                self.apu.channel_1_frequency = new_freq;
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
            self.apu.wave_ram[addr - 0xFF30] = val;
            self.write_memory(addr, val, SOURCE);
        }
    }
    pub fn wave_ram_read(&self, addr: usize) -> u8 {
        let addr_send = if self.get_memory(NR30_ADDR, SOURCE) >> 7 == 0 {
            addr
        } else {
            self.apu.channel_3_pointer + 0xFF32
        };
        self.get_memory(addr_send, SOURCE)
    }
    pub fn nrx1_write(&mut self, channel: usize, val: u8) {
        let (mask, max) = match channel {
            1 => {
                self.apu.channel_1_duty = DUTY_CONVERSION[(val >> 6) as usize];
                (0x3F, 64)
            }
            2 => {
                self.apu.channel_2_duty = DUTY_CONVERSION[(val >> 6) as usize];
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
            let mut new_freq = (*freq_atomic);
            new_freq &= 0xFF;
            new_freq |= (val as u32 & 0x7) << 8;
            (*freq_atomic) = new_freq;
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
        self.apu.channel_4_frequency = 524288 / dividing_factor;
        self.apu.channel_4_width = ((val >> 3) & 1) == 1;
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
        self.apu.so1_level = (val & 0x7) as f32 / 7.0;
        self.apu.so2_level = ((val >> 4) & 0x7) as f32 / 7.0;
    }
    pub fn nr51_write(&mut self, val: u8) {
        self.apu.channel_1_so1_enable = val & 1;
        self.apu.channel_2_so1_enable = (val >> 1) & 1;
        self.apu.channel_3_so1_enable = (val >> 2) & 1;
        self.apu.channel_4_so1_enable = (val >> 3) & 1;

        self.apu.channel_1_so2_enable = (val >> 4) & 1;
        self.apu.channel_2_so2_enable = (val >> 5) & 1;
        self.apu.channel_3_so2_enable = (val >> 6) & 1;
        self.apu.channel_4_so2_enable = (val >> 7) & 1;
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
        self.apu.volumes[CH1_IND] = self.apu.initial_volumes[CH1_IND];
        self.apu.channel_1_sweep_enable =
            self.apu.channel_1_sweep_shift != 0 || self.apu.channel_1_sweep_period != 0;
        self.apu.channel_1_shadow_frequency = self.apu.channel_1_frequency;

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
        self.apu.volumes[CH2_IND] = self.apu.initial_volumes[CH2_IND];
        self.enable_channel(2);
        self.dac_check(2);
        self.refill_check(2);
    }
    fn trigger_channel_3(&mut self) {
        self.apu.channel_3_pointer = 0;
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
        self.apu.volumes[CH4_IND] = self.apu.initial_volumes[CH4_IND];
        self.apu.channel_4_lsfr = 0x7FFF;
        self.enable_channel(4);
        self.refill_check(4);
        self.dac_check(4);
    }
    fn channel_1_buffer_add(&mut self) {
        let enable = self.apu.channel_1_enable & self.apu.all_sound_enable;
        if !enable || self.apu.channel_1_phase < self.apu.channel_1_duty {
            self.apu.ch_1_so1_buffer[self.apu.ch_1_buffer_pointer] = 0.0;
            self.apu.ch_1_so2_buffer[self.apu.ch_1_buffer_pointer] = 0.0;
        } else {
            let so1_mod = self.apu.channel_1_so1_enable as f32 * self.apu.so1_level;
            let so2_mod = self.apu.channel_1_so2_enable as f32 * self.apu.so2_level;
            self.apu.ch_1_so1_buffer[self.apu.ch_1_buffer_pointer] =
                (self.apu.volumes[CH1_IND] as f32 * so1_mod) / 100.0;
            self.apu.ch_1_so2_buffer[self.apu.ch_1_buffer_pointer] =
                (self.apu.volumes[CH1_IND] as f32 * so2_mod) / 100.0;
        }
        self.apu.ch_1_buffer_pointer += 1;
        self.apu.channel_1_phase = (self.apu.channel_1_phase
            + (131072.0 / ((MAX_FREQ_VAL - self.apu.channel_1_frequency) as f32))
                / AUDIO_DATA_HZ as f32)
            % 1.0;
    }
    pub fn buffer_check(&mut self) {
        if self.apu.ch_1_queue.size() == 0 && !self.apu.buffering {
            println!("pausing!");
            self.apu.buffering = true;
            self.apu.ch_1_queue.pause()
        } else if self.apu.ch_1_queue.size() > 0 && self.apu.buffering {
            println!("resuming!");
            self.apu.buffering = false;
            self.apu.ch_1_queue.resume();
        }
    }
    pub fn send_to_queue(&mut self) {
        for (i, data) in self.apu.ch_1_queue_data.iter_mut().enumerate() {
            let ind = (i / 2) as f32;
            let buffer_index = (ind as f32 * SAMPLING_SCALE_FACTOR).round() as usize;
            *data = if i % 2 == 0 {
                self.apu.ch_1_so2_buffer[buffer_index]
            } else {
                self.apu.ch_1_so1_buffer[buffer_index]
            };
        }
        self.apu.ch_1_buffer_pointer = 0;
        self.apu.ch_1_queue.queue(&self.apu.ch_1_queue_data);
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
        if self.apu.cycle_count % AUDIO_DATA_CYCLES == 0 {
            self.channel_1_buffer_add();
        }
    }
}

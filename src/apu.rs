use crate::constants::*;
use crate::emulator::GameBoyEmulator;
use crate::emulator::RequestSource;
use sdl2::audio::AudioQueue;
use sdl2::audio::AudioSpecDesired;
use sdl2::AudioSubsystem;
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
    ch_1_frequency: u32,
    ch_1_shadow_frequency: u32,
    ch_2_frequency: u32,
    ch_1_sweep_timer: u8,
    ch_1_sweep_enable: bool,
    ch_1_sweep_inc: bool,
    ch_1_sweep_period: u8,
    ch_1_sweep_shift: u8,
    ch_1_neg_after_trigger: bool,
    ch_1_phase_counter: u32,
    ch_1_queue: AudioQueue<f32>,
    cycle_count: u32,
    sample_cycle_count: f32,

    so1_level: f32,
    so2_level: f32,
    pub all_sound_enable: bool,
    ch_1_enable: bool,

    ch_1_duty_counter: u8,
    ch_1_duty_val: u8,
    ch_1_so1_enable: u8,
    ch_1_so2_enable: u8,
    ch_1_phase: f32,
    ch_2_enable: bool,

    ch_2_duty_counter: u8,
    ch_2_duty_val: u8,
    ch_2_so1_enable: u8,
    ch_2_so2_enable: u8,
    ch_2_phase_counter: u32,
    ch_2_queue: AudioQueue<f32>,

    ch_3_pointer: usize,
    ch_3_enable: bool,
    ch_3_frequency: u32,
    pub ch_3_output_level: u8,
    ch_3_so1_enable: u8,
    ch_3_so2_enable: u8,
    ch_3_phase_counter: u32,
    ch_3_queue: AudioQueue<f32>,

    ch_4_lsfr: u16,
    ch_4_enable: bool,
    ch_4_frequency: u32,
    ch_4_width: bool,
    ch_4_so1_enable: u8,
    ch_4_so2_enable: u8,
    ch_4_phase_counter: u32,
    ch_4_queue: AudioQueue<f32>,
}

impl AudioProcessingUnit {
    pub fn new(audio_subsystem: AudioSubsystem) -> AudioProcessingUnit {
        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLES_PER_SECOND as i32),
            channels: Some(2),
            samples: Some(256),
        };
        let ch_1_queue = audio_subsystem.open_queue(None, &desired_spec).unwrap();
        let ch_2_queue = audio_subsystem.open_queue(None, &desired_spec).unwrap();
        let ch_3_queue = audio_subsystem.open_queue(None, &desired_spec).unwrap();
        let ch_4_queue = audio_subsystem.open_queue(None, &desired_spec).unwrap();
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
            ch_1_frequency: 1,
            ch_1_shadow_frequency: 0,
            ch_2_frequency: 1,
            ch_1_sweep_timer: 1,
            ch_1_sweep_enable: false,
            ch_1_sweep_inc: false,
            ch_1_sweep_period: 0,
            ch_1_sweep_shift: 0,
            ch_1_neg_after_trigger: false,
            ch_1_phase_counter: 1,
            ch_1_queue,
            cycle_count: 0,
            sample_cycle_count: 0.0,

            so1_level: 0.0,
            so2_level: 0.0,
            all_sound_enable: true,
            ch_1_enable: false,

            ch_1_duty_counter: 0,
            ch_1_duty_val: 0,
            ch_1_so1_enable: 0,
            ch_1_so2_enable: 0,
            ch_1_phase: 0.0,
            ch_2_enable: false,

            ch_2_duty_counter: 0,
            ch_2_duty_val: 0,
            ch_2_so1_enable: 0,
            ch_2_so2_enable: 0,
            ch_2_phase_counter: 1,
            ch_2_queue,

            ch_3_pointer: 0,
            ch_3_enable: false,
            ch_3_frequency: 0,
            ch_3_output_level: 0,
            ch_3_so1_enable: 0,
            ch_3_so2_enable: 0,
            ch_3_phase_counter: 1,
            ch_3_queue,

            ch_4_lsfr: 0,
            ch_4_enable: false,
            ch_4_frequency: 1,
            ch_4_width: false,
            ch_4_so1_enable: 0,
            ch_4_so2_enable: 0,
            ch_4_phase_counter: 1,
            ch_4_queue,
        }
    }
}
impl GameBoyEmulator {
    pub fn disable_channel(&mut self, channel: usize) {
        let (enable_channel, mask) = match channel {
            1 => (&mut self.apu.ch_1_enable, 0b11111110),
            2 => (&mut self.apu.ch_2_enable, 0b11111101),
            3 => (&mut self.apu.ch_3_enable, 0b11111011),
            4 => (&mut self.apu.ch_4_enable, 0b11110111),
            _ => panic!("Wow, how did you get here? You gave a channel for disable that's bad."),
        };
        (*enable_channel) = false;
        self.write_memory(NR52_ADDR, self.get_memory(NR52_ADDR, SOURCE) & mask, SOURCE);
    }
    fn enable_channel(&mut self, channel: usize) {
        let (enable_channel, mask) = match channel {
            1 => (&mut self.apu.ch_1_enable, 0b00000001),
            2 => (&mut self.apu.ch_2_enable, 0b00000010),
            3 => (&mut self.apu.ch_3_enable, 0b00000100),
            4 => (&mut self.apu.ch_4_enable, 0b00001000),
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
            1 => (NR13_ADDR, NR14_ADDR, self.apu.ch_1_frequency),
            _ => panic!("bad channel!"),
        };
        self.apu.ch_1_shadow_frequency = frequency_val;
        let low_reg_val = (frequency_val & 0xFF) as u8;
        let high_reg_part_val = ((frequency_val >> 8) & 0x7) as u8;
        self.write_memory(low_reg_addr, low_reg_val, SOURCE);
        let high_reg_val = (self.get_memory(high_reg_addr, SOURCE) & 0xF8) | high_reg_part_val;
        self.write_memory(high_reg_addr, high_reg_val, SOURCE);
    }
    pub fn update_frequency_internal_low(&mut self, channel: usize, val: u8) {
        let freq = match channel {
            1 => &mut self.apu.ch_1_frequency,
            2 => &mut self.apu.ch_2_frequency,
            3 => &mut self.apu.ch_3_frequency,
            _ => panic!("bad channel!"),
        };
        let mut new_freq = *freq;
        new_freq &= 0x700;
        new_freq |= val as u32;
        (*freq) = new_freq;
        if channel == 1 && !self.apu.ch_1_sweep_enable {
            self.apu.ch_1_shadow_frequency = new_freq;
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
        let op_val = self.apu.ch_1_shadow_frequency >> self.apu.ch_1_sweep_shift;
        let mut new_freq = if self.apu.ch_1_sweep_inc {
            self.apu.ch_1_shadow_frequency + op_val
        } else {
            self.apu.ch_1_shadow_frequency - op_val
        };
        if new_freq >= MAX_FREQ_VAL {
            self.disable_channel(1);
        } else {
            if self.apu.ch_1_sweep_shift != 0 && clocked {
                self.apu.ch_1_frequency = new_freq;
                self.update_frequency_addr(1);

                let second_op_val = new_freq >> self.apu.ch_1_sweep_shift;
                new_freq = if self.apu.ch_1_sweep_inc {
                    new_freq + second_op_val
                } else {
                    new_freq - second_op_val
                };
                if new_freq >= MAX_FREQ_VAL {
                    self.disable_channel(1);
                }
            }
        }
        if !self.apu.ch_1_sweep_inc {
            self.apu.ch_1_neg_after_trigger = true;
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
            self.write_memory(addr, val, SOURCE);
        }
    }
    pub fn wave_ram_read(&self, addr: usize) -> u8 {
        let addr_send = if self.get_memory(NR30_ADDR, SOURCE) >> 7 == 0 {
            addr
        } else {
            self.apu.ch_3_pointer + 0xFF32
        };
        self.get_memory(addr_send, SOURCE)
    }
    pub fn nrx1_write(&mut self, channel: usize, val: u8) {
        let (mask, max) = match channel {
            1 => {
                self.apu.ch_1_duty_val = DUTY_CONVERSION[(val >> 6) as usize];

                (0x3F, 64)
            }
            2 => {
                self.apu.ch_2_duty_val = DUTY_CONVERSION[(val >> 6) as usize];
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
            let freq = match channel {
                1 => &mut self.apu.ch_1_frequency,
                2 => &mut self.apu.ch_2_frequency,
                3 => &mut self.apu.ch_3_frequency,
                _ => panic!("bad channel!"),
            };
            let mut new_freq = *freq;
            new_freq &= 0xFF;
            new_freq |= (val as u32 & 0x7) << 8;
            (*freq) = new_freq;
            if (channel == 1) && (!self.apu.ch_1_sweep_enable || trigger) {
                self.apu.ch_1_shadow_frequency = new_freq;
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
        self.apu.ch_4_frequency = dividing_factor << 1;
        self.apu.ch_4_width = ((val >> 3) & 1) == 1;
    }
    pub fn sweep_var_update(&mut self, val: u8) {
        let (sweep_shift, sweep_period, sweep_inc) =
            { (val & 0b111, (val >> 4) & 0b111, ((val >> 3) & 1) == 0) };
        self.apu.ch_1_sweep_shift = sweep_shift;
        self.apu.ch_1_sweep_period = sweep_period;
        self.apu.ch_1_sweep_inc = sweep_inc;
        if sweep_inc && self.apu.ch_1_neg_after_trigger {
            self.disable_channel(1);
        }
    }
    pub fn nr50_write(&mut self, val: u8) {
        self.apu.so1_level = (val & 0x7) as f32 / 7.0;
        self.apu.so2_level = ((val >> 4) & 0x7) as f32 / 7.0;
    }
    pub fn nr51_write(&mut self, val: u8) {
        self.apu.ch_1_so1_enable = val & 1;
        self.apu.ch_2_so1_enable = (val >> 1) & 1;
        self.apu.ch_3_so1_enable = (val >> 2) & 1;
        self.apu.ch_4_so1_enable = (val >> 3) & 1;

        self.apu.ch_1_so2_enable = (val >> 4) & 1;
        self.apu.ch_2_so2_enable = (val >> 5) & 1;
        self.apu.ch_3_so2_enable = (val >> 6) & 1;
        self.apu.ch_4_so2_enable = (val >> 7) & 1;
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
    fn noise_lsfr(&mut self) {
        let new_bit = (self.apu.ch_4_lsfr & 1) ^ ((self.apu.ch_4_lsfr >> 1) & 1);
        self.apu.ch_4_lsfr >>= 1;
        self.apu.ch_4_lsfr += new_bit << 14;
        if self.apu.ch_4_width {
            self.apu.ch_4_lsfr &= 0b0111111;
            self.apu.ch_4_lsfr += new_bit << 6;
        }
    }
    fn trigger_channel_1(&mut self) {
        self.apu.ch_1_neg_after_trigger = false;
        self.enable_channel(1);
        self.dac_check(1);

        self.refill_check(1);

        self.apu.ch_1_sweep_timer = if self.apu.ch_1_sweep_period == 0 {
            8
        } else {
            self.apu.ch_1_sweep_period
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
        self.apu.ch_1_sweep_enable =
            self.apu.ch_1_sweep_shift != 0 || self.apu.ch_1_sweep_period != 0;
        self.apu.ch_1_shadow_frequency = self.apu.ch_1_frequency;
        self.apu.ch_1_phase_counter = MAX_FREQ_VAL - self.apu.ch_1_frequency;
        self.apu.ch_1_duty_counter = 7;
        self.apu.ch_1_phase = 0.0;
        if self.apu.ch_1_sweep_shift != 0 {
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
        self.apu.ch_2_phase_counter = MAX_FREQ_VAL - self.apu.ch_2_frequency;
        self.apu.ch_2_duty_counter = 7;
        self.enable_channel(2);
        self.dac_check(2);
        self.refill_check(2);
    }
    fn trigger_channel_3(&mut self) {
        self.apu.ch_3_pointer = 0;
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
        self.apu.ch_4_lsfr = 0x7FFF;
        self.enable_channel(4);
        self.refill_check(4);
        self.dac_check(4);
    }
    fn channel_1_buffer_add(&mut self) {
        let enable = self.apu.ch_1_enable & self.apu.all_sound_enable;
        if !enable {
            self.apu.ch_1_queue.queue(&[0.0, 0.0]);
        } else {
            let so1_mod = self.apu.ch_1_so1_enable as f32 * self.apu.so1_level;
            let so2_mod = self.apu.ch_1_so2_enable as f32 * self.apu.so2_level;

            let duty_mod = ((self.apu.ch_1_duty_val >> self.apu.ch_1_duty_counter) & 1) as f32;
            self.apu.ch_1_queue.queue(&[
                (self.apu.volumes[CH1_IND] as f32 * so2_mod * duty_mod) / 100.0,
                (self.apu.volumes[CH1_IND] as f32 * so1_mod * duty_mod) / 100.0,
            ]);
        }
    }
    fn channel_2_buffer_add(&mut self) {
        let enable = self.apu.ch_2_enable & self.apu.all_sound_enable;
        if !enable {
            self.apu.ch_2_queue.queue(&[0.0, 0.0]);
        } else {
            let so1_mod = self.apu.ch_2_so1_enable as f32 * self.apu.so1_level;
            let so2_mod = self.apu.ch_2_so2_enable as f32 * self.apu.so2_level;
            let duty_mod = ((self.apu.ch_2_duty_val >> self.apu.ch_2_duty_counter) & 1) as f32;
            self.apu.ch_2_queue.queue(&[
                (self.apu.volumes[CH2_IND] as f32 * so2_mod * duty_mod) / 100.0,
                (self.apu.volumes[CH2_IND] as f32 * so1_mod * duty_mod) / 100.0,
            ]);
        }
    }
    fn channel_3_buffer_add(&mut self) {
        let enable = self.apu.ch_3_enable & self.apu.all_sound_enable;
        let output_shift =
            VOLUME_SHIFT_CONVERSION[self.get_memory(NR32_ADDR, SOURCE) as usize >> 5 & 0x3];
        if !enable {
            self.apu.ch_3_queue.queue(&[0.0, 0.0]);
        } else {
            let so1_mod = self.apu.ch_3_so1_enable as f32 * self.apu.so1_level;
            let so2_mod = self.apu.ch_3_so2_enable as f32 * self.apu.so2_level;
            let wave_val = if self.apu.ch_3_pointer % 2 == 0 {
                self.get_memory(0xFF30 + self.apu.ch_3_pointer / 2, SOURCE) >> 4
            } else {
                self.get_memory(0xFF30 + (self.apu.ch_3_pointer - 1) / 2, SOURCE) & 0xF
            };

            self.apu.ch_3_queue.queue(&[
                ((wave_val >> output_shift) as f32 * so2_mod) / 100.0,
                ((wave_val >> output_shift) as f32 * so1_mod) / 100.0,
            ]);
        }
    }
    fn channel_4_buffer_add(&mut self) {
        let enable = self.apu.ch_4_enable & self.apu.all_sound_enable;
        if !enable {
            self.apu.ch_4_queue.queue(&[0.0, 0.0]);
        } else {
            let so1_mod = self.apu.ch_4_so1_enable as f32 * self.apu.so1_level;
            let so2_mod = self.apu.ch_4_so2_enable as f32 * self.apu.so2_level;
            let reg_mod = (1 - (self.apu.ch_4_lsfr & 1)) as f32;
            self.apu.ch_4_queue.queue(&[
                (self.apu.volumes[CH4_IND] as f32 * so2_mod * reg_mod) / 100.0,
                (self.apu.volumes[CH4_IND] as f32 * so1_mod * reg_mod) / 100.0,
            ]);
        }
    }
    fn buffer_empty(&self) -> bool {
        return self.apu.ch_1_queue.size() == 0
            || self.apu.ch_2_queue.size() == 0
            || self.apu.ch_3_queue.size() == 0
            || self.apu.ch_4_queue.size() == 0;
    }
    pub fn buffer_check(&mut self) {
        if self.buffer_empty() && !self.apu.buffering {
            self.apu.buffering = true;
            self.apu.ch_1_queue.pause();
            self.apu.ch_2_queue.pause();
            self.apu.ch_3_queue.pause();
            self.apu.ch_4_queue.pause();
        } else if !self.buffer_empty() && self.apu.buffering {
            self.apu.buffering = false;
            self.apu.ch_1_queue.resume();
            self.apu.ch_2_queue.resume();
            self.apu.ch_3_queue.resume();
            self.apu.ch_4_queue.resume();
        }
    }
    pub fn apu_advance(&mut self) {
        self.apu.cycle_count = (self.apu.cycle_count + ADVANCE_CYCLES) % CYCLE_COUNT_8HZ;
        self.apu.sample_cycle_count += 4.0;
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
                self.apu.ch_1_sweep_timer -= 1;
                if self.apu.ch_1_sweep_timer == 0 {
                    self.apu.ch_1_sweep_timer = if self.apu.ch_1_sweep_period == 0 {
                        8
                    } else {
                        self.apu.ch_1_sweep_period
                    };
                    if self.apu.ch_1_sweep_enable && self.apu.ch_1_sweep_period != 0 {
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
        self.apu.ch_1_phase_counter -= 1;
        if self.apu.ch_1_phase_counter == 0 {
            self.apu.ch_1_duty_counter = self.apu.ch_1_duty_counter.wrapping_sub(1) % 8;
            self.apu.ch_1_phase_counter = MAX_FREQ_VAL - self.apu.ch_1_frequency;
        }
        self.apu.ch_2_phase_counter -= 1;
        if self.apu.ch_2_phase_counter == 0 {
            self.apu.ch_2_duty_counter = self.apu.ch_2_duty_counter.wrapping_sub(1) % 8;
            self.apu.ch_2_phase_counter = MAX_FREQ_VAL - self.apu.ch_2_frequency;
        }

        for _ in 0..2 {
            self.apu.ch_3_phase_counter -= 1;
            if self.apu.ch_3_phase_counter == 0 {
                self.apu.ch_3_phase_counter = MAX_FREQ_VAL - self.apu.ch_3_frequency;
                self.apu.ch_3_pointer = (self.apu.ch_3_pointer + 1) % 32;
            }
        }

        self.apu.ch_4_phase_counter -= 1;
        if self.apu.ch_4_phase_counter == 0 {
            self.apu.ch_4_phase_counter = self.apu.ch_4_frequency;
            self.noise_lsfr();
        }

        if self.apu.sample_cycle_count >= AUDIO_BUFFER_CLOCK {
            self.apu.sample_cycle_count -= AUDIO_BUFFER_CLOCK;
            self.channel_1_buffer_add();
            self.channel_2_buffer_add();
            self.channel_3_buffer_add();
            self.channel_4_buffer_add();
        }
    }
}

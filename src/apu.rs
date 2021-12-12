use sdl2::audio::{AudioQueue, AudioSpecDesired};
use std::sync::{Arc, Mutex};

use crate::ADVANCE_CYCLES;

const CLOCK: u32 = 1_048_576;

const DUTY_CONVERSION: [f32; 4] = [0.125, 0.25, 0.5, 0.75];

pub struct AudioProcessingUnit {
    queue: AudioQueue<f32>,
    nr10: Arc<Mutex<u8>>,
    nr11: Arc<Mutex<u8>>,
    nr12: Arc<Mutex<u8>>,
    nr13: Arc<Mutex<u8>>,
    nr14: Arc<Mutex<u8>>,
    channel_1_sweep_count: u8,
    channel_1_sweep_enable: bool,
    cycle_count: u32,
    channel_1_enable: bool,
    channel_1_volume: u8,
    channel_1_volume_count: u8,
    channel_1_phase: f32,
    channel_1_queue: Vec<f32>,
}

impl AudioProcessingUnit {
    pub fn new(
        queue: AudioQueue<f32>,
        nr10: Arc<Mutex<u8>>,
        nr11: Arc<Mutex<u8>>,
        nr12: Arc<Mutex<u8>>,
        nr13: Arc<Mutex<u8>>,
        nr14: Arc<Mutex<u8>>,
    ) -> AudioProcessingUnit {
        let cycle_count = 0;
        let channel_1_sweep_count = 0;
        let channel_1_enable = false;
        let channel_1_volume = 0;
        let channel_1_volume_count = 0;
        let channel_1_phase = 0.0;
        let channel_1_sweep_enable = false;
        let channel_1_queue: Vec<f32> = Vec::new();
        queue.resume();
        AudioProcessingUnit {
            queue,
            nr10,
            nr11,
            nr12,
            nr13,
            nr14,
            cycle_count,
            channel_1_sweep_count,
            channel_1_sweep_enable,
            channel_1_enable,
            channel_1_volume,
            channel_1_volume_count,
            channel_1_phase,
            channel_1_queue,
        }
    }

    fn volume_envelope(&mut self, channel: u8) {
        let (volume_reg, channel_volume_count, channel_volume) = match channel {
            1 => (
                self.nr12.lock().unwrap(),
                &mut self.channel_1_volume_count,
                &mut self.channel_1_volume,
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
            *channel_volume_count += 1;
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
                self.channel_1_enable = false;
            } else {
                *self.nr13.lock().unwrap() = (*frequency & 0xFF) as u8;
                *self.nr14.lock().unwrap() &= 0b11111000;
                *self.nr14.lock().unwrap() |= ((*frequency >> 8) & 0b111) as u8;
            }
        } else {
            self.channel_1_sweep_count += 1;
        }
    }
    fn length_unit(&mut self, channel: u8, length: &mut u8) {
        let (cc_reg, mut length_reg) = match channel {
            1 => (self.nr14.lock().unwrap(), self.nr11.lock().unwrap()),
            _ => {
                panic!("Wow, how did you get here? You gave a channel for length unit that's bad.")
            }
        };
        let counter_consec = *cc_reg >> 6 & 1;
        if counter_consec == 1 {
            if *length == 0 {
                self.channel_1_enable = false;
            } else {
                *length -= 1;
                *length_reg &= 0b00000;
                *length_reg |= 64 - *length;
            }
        }
    }

    fn channel_1_advance(&mut self) {
        let initialize = (*self.nr14.lock().unwrap() >> 7) == 1;
        let mut length = 64 - *self.nr11.lock().unwrap() & 0b11111;
        if initialize {
            if length == 0 {
                length = 64;
            }
            self.channel_1_enable = true;
            self.cycle_count = 0;
            self.channel_1_sweep_count = 0;
            self.channel_1_volume_count = 0;
            self.channel_1_volume = *self.nr12.lock().unwrap() >> 4;
            let (sweep_shift, sweep_time) = {
                let sweep_reg = *self.nr10.lock().unwrap();
                (sweep_reg & 0b11, sweep_reg >> 4)
            };
            self.channel_1_sweep_enable = if sweep_shift == 0 || sweep_time == 0 {
                false
            } else {
                true
            };
            *self.nr14.lock().unwrap() &= 0b01111111;
        }
        let mut frequency = (((*self.nr14.lock().unwrap() & 0b111) as u32) << 8)
            + *self.nr13.lock().unwrap() as u32;

        // if self.cycle_count % 32768 == 0 && self.channel_1_sweep_enable {
        //     self.sweep_channel_1(&mut frequency);
        // }
        // if self.cycle_count == 0 {
        //     self.volume_envelope(1);
        // }

        // if self.cycle_count % 16384 == 0 {
        //     self.length_unit(1, &mut length);
        // }

        if self.cycle_count % 64 == 0 && self.channel_1_enable {
            let duty = *self.nr11.lock().unwrap() >> 6;
            if self.channel_1_phase < DUTY_CONVERSION[duty as usize] {
                self.queue.queue(&[0.0]);
            } else {
                //self.queue.queue(&[self.channel_1_volume as f32 / 100.0]);
                self.queue.queue(&[0.15]);
            }
            self.channel_1_phase = (self.channel_1_phase
                + (131072.0 / (2048.0 - frequency as f32)) / 65536 as f32)
                % 1.0;
        }
    }

    pub fn advance(&mut self) {
        self.channel_1_advance();
        self.cycle_count = (self.cycle_count + ADVANCE_CYCLES) % 65536;
    }
}

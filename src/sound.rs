use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::sync::{Arc, Mutex};

const DUTY_CONVERSION: [f32; 4] = [0.125, 0.25, 0.5, 0.75];
const VOLUME_SHIFT_CONVERSION: [u8; 4] = [4, 0, 1, 2];

pub struct Channel1 {
    pub sweep_reg: Arc<Mutex<u8>>,
    pub length_duty_reg: Arc<Mutex<u8>>,
    pub vol_envelope_reg: Arc<Mutex<u8>>,
    pub freq_low_reg: Arc<Mutex<u8>>,
    pub freq_high_reg: Arc<Mutex<u8>>,
    pub clock: u16,
    pub sweep_period: u8,
    pub volume_period: u8,
    pub frequency: u32,
    pub channel_enable: bool,
    pub sweep_enable: bool,
    pub volume: u8,
    pub phase: f32,
}

impl AudioCallback for Channel1 {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let initialize = (*self.freq_high_reg.lock().unwrap() >> 7) == 1;
        let mut length = 64 - *self.length_duty_reg.lock().unwrap() & 0b11111;
        if initialize {
            if length == 0 {
                length = 64;
            }
            self.channel_enable = true;
            self.clock = 0;
            self.sweep_period = 0;
            self.volume_period = 0;
            self.volume = *self.vol_envelope_reg.lock().unwrap() >> 4;
            let (sweep_shift, sweep_time) = {
                let sweep_reg = *self.sweep_reg.lock().unwrap();
                (sweep_reg & 0b11, sweep_reg >> 4)
            };
            self.sweep_enable = if sweep_shift == 0 || sweep_time == 0 {
                false
            } else {
                true
            };
            *self.freq_high_reg.lock().unwrap() &= 0b01111111;
        }
        self.frequency = (((*self.freq_high_reg.lock().unwrap() & 0b111) as u32) << 8)
            + *self.freq_low_reg.lock().unwrap() as u32;
        let counter_consec = *self.freq_high_reg.lock().unwrap() >> 6 & 1;
        for x in out.iter_mut() {
            //SWEEP MODULE
            if self.sweep_enable && self.clock % 250 == 0 {
                let (sweep_shift, sweep_inc, sweep_time) = {
                    let sweep_reg = *self.sweep_reg.lock().unwrap();
                    (sweep_reg & 0b11, (sweep_reg >> 3 & 1) == 0, sweep_reg >> 4)
                };
                if sweep_time != 0 && self.sweep_period >= (sweep_time - 1) && sweep_shift != 0 {
                    let old_frequency = self.frequency;
                    self.frequency = if sweep_inc {
                        self.frequency + (self.frequency >> sweep_shift)
                    } else {
                        self.frequency - (self.frequency >> sweep_shift)
                    };
                    self.sweep_period = 0;
                    if self.frequency >= 2048 {
                        self.frequency = old_frequency;
                        self.channel_enable = false;
                    } else {
                        *self.freq_low_reg.lock().unwrap() = (self.frequency & 0xFF) as u8;
                        *self.freq_high_reg.lock().unwrap() &= 0b11111000;
                        *self.freq_high_reg.lock().unwrap() |=
                            ((self.frequency >> 8) & 0b111) as u8;
                    }
                } else {
                    self.sweep_period += 1;
                }
            }
            //DUTY MODULE
            let duty = DUTY_CONVERSION[(*self.length_duty_reg.lock().unwrap() >> 6) as usize];
            let volume_time = *self.vol_envelope_reg.lock().unwrap() & 0b111;
            let vol_inc = ((*self.vol_envelope_reg.lock().unwrap() >> 3) & 1) == 1;

            //LENGTH MODULE
            if counter_consec == 1 && self.clock % 125 == 0 {
                if length == 0 {
                    self.channel_enable = false;
                } else {
                    length -= 1;
                    *self.length_duty_reg.lock().unwrap() &= 0b00000;
                    *self.length_duty_reg.lock().unwrap() |= 64 - length;
                }
            }

            //VOLUME ENVELOPE MODULE
            if self.clock % 500 == 0 {
                if volume_time != 0 && self.volume_period >= (volume_time - 1) {
                    self.volume = if vol_inc && self.volume < 15 {
                        self.volume + 1
                    } else if self.volume > 0 {
                        self.volume - 1
                    } else {
                        self.volume
                    };
                    self.volume_period = 0;
                } else {
                    self.volume_period += 1;
                }
            }

            *x = if !self.channel_enable {
                0.0
            } else if self.phase <= (1.0 - duty) {
                self.volume as f32 / 100.0
            } else {
                //-(self.volume as f32 / 100.0)
                0.0
            };
            let phase_add = (131072.0 / (2048 - self.frequency) as f32) / 32000.0;
            //let phase_add = 440.0 / 32000.0;
            self.phase = (self.phase + phase_add as f32) % 1.0;
            self.clock = (self.clock + 1) % 32000;
        }
    }
}

pub struct Channel2 {
    pub length_duty_reg: Arc<Mutex<u8>>,
    pub vol_envelope_reg: Arc<Mutex<u8>>,
    pub freq_low_reg: Arc<Mutex<u8>>,
    pub freq_high_reg: Arc<Mutex<u8>>,
    pub clock: u16,
    pub volume_period: u8,
    pub frequency: u32,
    pub channel_enable: bool,
    pub volume: u8,
    pub phase: f32,
}

impl AudioCallback for Channel2 {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let initialize = (*self.freq_high_reg.lock().unwrap() >> 7) == 1;
        let mut length = 64 - *self.length_duty_reg.lock().unwrap() & 0b11111;
        if initialize {
            if length == 0 {
                length = 64;
            }
            self.channel_enable = true;
            self.clock = 0;
            self.volume_period = 0;
            self.volume = *self.vol_envelope_reg.lock().unwrap() >> 4;

            *self.freq_high_reg.lock().unwrap() &= 0b01111111;
        }
        self.frequency = (((*self.freq_high_reg.lock().unwrap() & 0b111) as u32) << 8)
            + *self.freq_low_reg.lock().unwrap() as u32;
        let counter_consec = *self.freq_high_reg.lock().unwrap() >> 6 & 1;
        //SWEEP MODULE
        //DUTY MODULE
        let duty = DUTY_CONVERSION[(*self.length_duty_reg.lock().unwrap() >> 6) as usize];
        let volume_time = *self.vol_envelope_reg.lock().unwrap() & 0b111;
        let vol_inc = ((*self.vol_envelope_reg.lock().unwrap() >> 3) & 1) == 1;
        for x in out.iter_mut() {
            //LENGTH MODULE
            if counter_consec == 1 && self.clock % 125 == 0 {
                if length == 0 {
                    self.channel_enable = false;
                } else {
                    length -= 1;
                    *self.length_duty_reg.lock().unwrap() &= 0b00000;
                    *self.length_duty_reg.lock().unwrap() |= 64 - length;
                }
            }

            //VOLUME ENVELOPE MODULE
            if self.clock % 500 == 0 {
                if volume_time != 0 && self.volume_period >= (volume_time - 1) {
                    self.volume = if vol_inc && self.volume < 15 {
                        self.volume + 1
                    } else if self.volume > 0 {
                        self.volume - 1
                    } else {
                        self.volume
                    };
                    self.volume_period = 0;
                } else {
                    self.volume_period += 1;
                }
            }

            *x = if !self.channel_enable {
                0.0
            } else if self.phase <= (1.0 - duty) {
                self.volume as f32 / 100.0
            } else {
                //-(self.volume as f32 / 100.0)
                0.0
            };
            let phase_add = (131072.0 / (2048 - self.frequency) as f32) / 32000.0;
            //let phase_add = 440.0 / 32000.0;
            self.phase = (self.phase + phase_add as f32) % 1.0;
            self.clock = (self.clock + 1) % 32000;
        }
    }
}

pub struct Channel3 {
    pub on_off_reg: Arc<Mutex<u8>>,
    pub length_reg: Arc<Mutex<u8>>,
    pub output_reg: Arc<Mutex<u8>>,
    pub freq_low_reg: Arc<Mutex<u8>>,
    pub freq_high_reg: Arc<Mutex<u8>>,
    pub frequency: u32,
    pub wave_ram: Arc<Mutex<[u8; 16]>>,
    pub pointer: usize,
    pub channel_enable: bool,
    pub phase: f32,
    pub clock: u16,
    pub upper: bool,
}

impl AudioCallback for Channel3 {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let initialize = (*self.freq_high_reg.lock().unwrap() >> 7) == 1;
        let mut length = 256 - (*self.length_reg.lock().unwrap() & 0b11111) as u16;
        if initialize {
            if length == 0 {
                length = 256;
            }
            self.pointer = 0;
            *self.freq_high_reg.lock().unwrap() &= 0b01111111;
            self.channel_enable = true;
        }

        let counter_consec = *self.freq_high_reg.lock().unwrap() >> 6 & 1;

        //LENGTH MODULE
        for x in out.iter_mut() {
            self.frequency = (((*self.freq_high_reg.lock().unwrap() & 0b111) as u32) << 8)
                + *self.freq_low_reg.lock().unwrap() as u32;
            let volume_shift =
                VOLUME_SHIFT_CONVERSION[((*self.output_reg.lock().unwrap() >> 5) & 0b11) as usize];
            if counter_consec == 1 && self.clock % 125 == 0 {
                if length == 0 {
                    self.channel_enable = false;
                } else {
                    length -= 1;
                    *self.length_reg.lock().unwrap() &= 0b00000;
                    *self.length_reg.lock().unwrap() |= (256 - length) as u8;
                }
            }
            *x = if self.channel_enable && (*self.on_off_reg.lock().unwrap() >> 7) == 1 {
                if self.upper {
                    ((self.wave_ram.lock().unwrap()[self.pointer] >> 4) >> volume_shift) as f32
                        / 100.0
                } else {
                    ((self.wave_ram.lock().unwrap()[self.pointer] & 0xF) >> volume_shift) as f32
                        / 100.0
                }
            } else {
                0.0
            };
            let phase_add = (65536.0 / (2048.0 - self.frequency as f32)) / 32000.0;
            let old_phase = self.phase;
            self.phase = (self.phase as f32 + phase_add as f32) % 1.0;
            if self.phase < old_phase {
                if !self.upper {
                    self.pointer = (self.pointer + 1) % 16;
                }
                self.upper = !self.upper;
            }
            self.clock = (self.clock + 1) % 32000;
        }
    }
}

pub struct Channel4 {
    pub length_duty_reg: Arc<Mutex<u8>>,
    pub vol_envelope_reg: Arc<Mutex<u8>>,
    pub poly_counter_reg: Arc<Mutex<u8>>,
    pub counter_consec_reg: Arc<Mutex<u8>>,
    pub clock: u16,
    pub volume_period: u8,
    pub channel_enable: bool,
    pub volume: u8,
    pub phase: f32,
    pub lsfr_bits: u16,
    pub high: bool,
}

fn noise_lsfr(reg: u16, width: bool) -> (u16, bool) {
    let new_bit = (reg & 1) ^ ((reg >> 1) & 1);
    let out_bit = reg & 1;
    let mut shift_reg = reg >> 1;
    if width {
        shift_reg &= 0b0111111;
        shift_reg += new_bit << 6;
    } else {
        shift_reg += new_bit << 14;
    }
    (shift_reg, out_bit == 1)
}

impl AudioCallback for Channel4 {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let initialize = (*self.counter_consec_reg.lock().unwrap() >> 7) == 1;
        let mut length = 64 - (*self.length_duty_reg.lock().unwrap() & 0b11111);
        if initialize {
            if length == 0 {
                length = 64;
            }
            *self.counter_consec_reg.lock().unwrap() &= 0b01111111;
            self.lsfr_bits = 0x7FFF;
            self.channel_enable = true;
        }
        let (frequency, width) = {
            let poly_counter_reg = *self.poly_counter_reg.lock().unwrap();
            let shift_clock_freq = poly_counter_reg >> 4;
            let width = ((poly_counter_reg >> 3) & 1) == 1;
            let freq_divider = {
                let possible_ratio = poly_counter_reg & 0b111;
                let s_factor = 1 << (shift_clock_freq + 1);
                if possible_ratio == 0 {
                    s_factor >> 1
                } else {
                    s_factor * possible_ratio
                }
            };
            (524288.0 / freq_divider as f32, width)
        };
        let counter_consec = *self.counter_consec_reg.lock().unwrap() >> 6 & 1;
        let volume_time = *self.vol_envelope_reg.lock().unwrap() & 0b111;
        let vol_inc = ((*self.vol_envelope_reg.lock().unwrap() >> 3) & 1) == 1;
        for x in out.iter_mut() {
            //LENGTH MODULE

            if counter_consec == 1 && self.clock % 125 == 0 {
                if length == 0 {
                    self.channel_enable = false;
                } else {
                    length -= 1;
                    *self.length_duty_reg.lock().unwrap() &= 0b00000;
                    *self.length_duty_reg.lock().unwrap() |= 64 - length;
                }
            }

            if self.clock % 500 == 0 {
                if volume_time != 0 && self.volume_period >= (volume_time - 1) {
                    self.volume = if vol_inc && self.volume < 15 {
                        self.volume + 1
                    } else if self.volume > 0 {
                        self.volume - 1
                    } else {
                        self.volume
                    };
                    self.volume_period = 0;
                } else {
                    self.volume_period += 1;
                }
            }

            *x = if !self.channel_enable || frequency >= 32000.0 {
                0.0
            } else if self.high {
                self.volume as f32 / 100.0
            } else {
                -(self.volume as f32) / 100.0
            };
            let old_phase = self.phase;
            self.phase += frequency / 32000.0;
            if old_phase > self.phase {
                let (new_lfsr_bits, new_high) = noise_lsfr(self.lsfr_bits, width);
                self.lsfr_bits = new_lfsr_bits;
                self.high = new_high;
            }
            self.clock = (self.clock + 1) % 32000;
        }
    }
}

use crate::constants::*;
use crate::emulator::{GameBoyEmulator, RequestSource};
use serde::{Deserialize, Serialize};
const TAC_MAPPING: [u32; 4] = [1024, 16, 64, 256];

const SOURCE: RequestSource = RequestSource::Timer;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Timer {
    prev_clock: u32,
    tima_counter: u32,
    div_counter: u32,
}

impl Timer {
    pub fn new() -> Timer {
        let prev_clock = 64;
        let tima_counter = 0;
        let div_counter = 0;
        Timer {
            prev_clock,
            tima_counter,
            div_counter,
        }
    }
}
impl GameBoyEmulator {
    fn set_timer_interrupt(&mut self) {
        self.write_memory(
            INT_FLAG_ADDR,
            self.get_memory(INT_FLAG_ADDR, SOURCE) | 0b100,
            SOURCE,
        );
    }
    pub fn timer_advance(&mut self) {
        let tac = self.get_memory(TAC_ADDR, SOURCE) as usize & 0x7;
        if (tac >> 2) == 1 {
            let clock = TAC_MAPPING[tac & 0b11];
            self.timer.tima_counter = if clock == self.timer.prev_clock {
                self.timer.tima_counter
            } else {
                self.timer.prev_clock = clock;
                0
            };
            self.timer.tima_counter += ADVANCE_CYCLES;
            if self.timer.tima_counter == clock {
                self.timer.tima_counter = 0;
                let mut tima = self.get_memory(TIMA_ADDR, SOURCE);
                tima = (tima).wrapping_add(1);
                if tima == 0 {
                    tima = self.get_memory(TMA_ADDR, SOURCE);
                    self.set_timer_interrupt();
                }
                self.write_memory(TIMA_ADDR, tima, SOURCE);
            }
        }
        self.timer.div_counter += ADVANCE_CYCLES;
        if self.timer.div_counter == CYCLE_COUNT_16384HZ {
            self.write_memory(
                DIV_ADDR,
                self.get_memory(DIV_ADDR, SOURCE).wrapping_add(1),
                SOURCE,
            );
            self.timer.div_counter = 0;
        }
    }
}

use std::sync::{Arc, Mutex};

use crate::ADVANCE_CYCLES;

const DOTS_PER_TIME: i32 = 16;
const TAC_MAPPING: [u32; 4] = [1024, 16, 64, 256];
const LIMIT_8: i32 = 255;

pub struct Timer {
    div: Arc<Mutex<u8>>,
    tima: Arc<Mutex<u8>>,
    tma: Arc<Mutex<u8>>,
    tac: Arc<Mutex<u8>>,
    interrupt_flag: Arc<Mutex<u8>>,
    cycle_count: u32,
    prev_clock: u32,
    tima_counter: u32,
    div_counter: u32,
}

impl Timer {
    pub fn new(
        div: Arc<Mutex<u8>>,
        tima: Arc<Mutex<u8>>,
        tma: Arc<Mutex<u8>>,
        tac: Arc<Mutex<u8>>,
        interrupt_flag: Arc<Mutex<u8>>,
    ) -> Timer {
        let cycle_count = 0;
        let prev_clock = 64;
        let tima_counter = 0;
        let div_counter = 0;
        Timer {
            div,
            tima,
            tma,
            tac,
            interrupt_flag,
            cycle_count,
            prev_clock,
            tima_counter,
            div_counter,
        }
    }
    fn set_timer_interrupt(&mut self) {
        *self.interrupt_flag.lock().unwrap() |= 0b100;
    }
    pub fn advance(&mut self) {
        let tac = *self.tac.lock().unwrap() as usize;
        if (tac >> 2) == 1 {
            let clock = TAC_MAPPING[tac & 0b11];
            self.tima_counter = if clock == self.prev_clock {
                self.tima_counter
            } else {
                self.prev_clock = clock;
                0
            };
            self.tima_counter += ADVANCE_CYCLES;
            if self.tima_counter == clock {
                self.tima_counter = 0;
                let mut tima = self.tima.lock().unwrap();
                *tima = (*tima).wrapping_add(1);
                if *tima == 0 {
                    *tima = *self.tma.lock().unwrap();
                    std::mem::drop(tima);
                    self.set_timer_interrupt();
                }
            }
        }
        self.div_counter += ADVANCE_CYCLES;
        if self.div_counter == 256 {
            let mut div = self.div.lock().unwrap();
            *div = (*div).wrapping_add(1);
            self.div_counter = 0;
        }
    }
}

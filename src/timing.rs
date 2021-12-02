use std::sync::{Arc, Mutex};
use std::time::Instant;

const NANOS_PER_TIME: u128 = 3185;
const TAC_MAPPING: [u8; 4] = [64, 1, 4, 16];

pub struct Timer {
    div: Arc<Mutex<u8>>,
    tima: Arc<Mutex<u8>>,
    tma: Arc<Mutex<u8>>,
    tac: Arc<Mutex<u8>>,
    interrupt_flag: Arc<Mutex<u8>>,
}

impl Timer {
    pub fn new(
        div: Arc<Mutex<u8>>,
        tima: Arc<Mutex<u8>>,
        tma: Arc<Mutex<u8>>,
        tac: Arc<Mutex<u8>>,
        interrupt_flag: Arc<Mutex<u8>>,
    ) -> Timer {
        Timer {
            div,
            tima,
            tma,
            tac,
            interrupt_flag,
        }
    }
    pub fn run(&mut self) {
        let mut div_counter = 0;
        let mut tima_counter = 0;
        let mut prev_clock = 64;
        loop {
            let now = Instant::now();
            let tac = *self.tac.lock().unwrap() as usize;
            if (tac >> 2) == 1 {
                let clock = TAC_MAPPING[tac & 0b11];
                tima_counter = if clock == prev_clock {
                    tima_counter
                } else {
                    prev_clock = clock;
                    0
                };
                tima_counter = (tima_counter + 1) % clock;
                if tima_counter == 0 {
                    let mut tima = self.tima.lock().unwrap();
                    *tima = (*tima).wrapping_add(1);
                    if *tima == 0 {
                        //interrupt
                        *tima = *self.tma.lock().unwrap();
                        *self.interrupt_flag.lock().unwrap() |= 0b100;
                    }
                }
            }

            div_counter = (div_counter + 1) % 16;
            if div_counter == 0 {
                let mut div = self.div.lock().unwrap();
                *div = (*div).wrapping_add(1);
            }

            while (now.elapsed().as_nanos()) < NANOS_PER_TIME {}
        }
    }
}

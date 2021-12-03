use crate::cycle_count_mod;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

const DOTS_PER_TIME: i32 = 16;
const TAC_MAPPING: [u8; 4] = [64, 1, 4, 16];

pub struct Timer {
    div: Arc<Mutex<u8>>,
    tima: Arc<Mutex<u8>>,
    tma: Arc<Mutex<u8>>,
    tac: Arc<Mutex<u8>>,
    interrupt_flag: Arc<Mutex<u8>>,
    cycle_count: Arc<Mutex<i32>>,
    cycle_cond: Arc<Condvar>,
    interrupt_cond: Arc<Condvar>,
}

impl Timer {
    pub fn new(
        div: Arc<Mutex<u8>>,
        tima: Arc<Mutex<u8>>,
        tma: Arc<Mutex<u8>>,
        tac: Arc<Mutex<u8>>,
        interrupt_flag: Arc<Mutex<u8>>,
        cycle_count: Arc<Mutex<i32>>,
        cycle_cond: Arc<Condvar>,
        interrupt_cond: Arc<Condvar>,
    ) -> Timer {
        Timer {
            div,
            tima,
            tma,
            tac,
            interrupt_flag,
            cycle_count,
            cycle_cond,
            interrupt_cond,
        }
    }
    pub fn run(&mut self) {
        let mut div_counter = 0;
        let mut tima_counter = 0;
        let mut prev_clock = 64;
        let mut start_cycle_count = *self.cycle_count.lock().unwrap();
        loop {
            start_cycle_count = *self.cycle_count.lock().unwrap();
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
                        self.interrupt_cond.notify_all();
                    }
                }
            }

            div_counter = (div_counter + 1) % 16;
            if div_counter == 0 {
                let mut div = self.div.lock().unwrap();
                *div = (*div).wrapping_add(1);
            }
            let mut current_cycle_count = self.cycle_count.lock().unwrap();
            while cycle_count_mod(*current_cycle_count - start_cycle_count) <= DOTS_PER_TIME {
                current_cycle_count = self.cycle_cond.wait(current_cycle_count).unwrap();
            }
            println!("{}", *current_cycle_count - start_cycle_count);
            std::mem::drop(current_cycle_count);
        }
    }
}

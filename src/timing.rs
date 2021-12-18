use crate::emulator::GameBoyEmulator;
use crate::ADVANCE_CYCLES;

const TAC_MAPPING: [u32; 4] = [1024, 16, 64, 256];
const DIV_ADDR: usize = 0xFF04;
const TIMA_ADDR: usize = 0xFF05;
const TMA_ADDR: usize = 0xFF06;
const TAC_ADDR: usize = 0xFF07;
const INT_FLAG_ADDR: usize = 0xFF0F;

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
        self.mem_unit.write_memory(
            INT_FLAG_ADDR,
            self.mem_unit.get_memory(INT_FLAG_ADDR) | 0b100,
        );
    }
    pub fn timer_advance(&mut self) {
        let tac = self.mem_unit.get_memory(TAC_ADDR) as usize;
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
                let mut tima = self.mem_unit.get_memory(TIMA_ADDR);
                tima = (tima).wrapping_add(1);
                if tima == 0 {
                    tima = self.mem_unit.get_memory(TMA_ADDR);
                    std::mem::drop(tima);
                    self.set_timer_interrupt();
                }
                self.mem_unit.write_memory(TIMA_ADDR, tima);
            }
        }
        self.timer.div_counter += ADVANCE_CYCLES;
        if self.timer.div_counter == 256 {
            self.mem_unit
                .write_memory(DIV_ADDR, self.mem_unit.get_memory(DIV_ADDR).wrapping_add(1));
            self.timer.div_counter = 0;
        }
    }
}

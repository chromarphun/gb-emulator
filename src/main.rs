use std::env;

mod apu;
mod cpu;
mod emulator;
mod epu;
mod memory;
mod pdu;
mod ppu;
mod timing;

const ADVANCE_CYCLES: u32 = 4;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut em = emulator::GameBoyEmulator::new();
    em.load_rom(&args[1]);
    em.run()
}

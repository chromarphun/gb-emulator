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
    let mut em = emulator::GameBoyEmulator::new();
    let res = rfd::Dialog::pick_file().open();
    if res.len() == 1 {
        em.load_rom(&res[0]);
        em.run()
    }
}

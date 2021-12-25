mod apu;
mod constants;
mod cpu;
mod emulator;
mod epu;
mod memory;
//mod pdu;
mod bootroms;
mod ppu;
mod timing;

fn main() {
    let mut em = emulator::GameBoyEmulator::new();
    let res = rfd::Dialog::pick_file().open();
    if res.len() == 1 {
        em.load_rom(&res[0]);
        em.run()
    }
}

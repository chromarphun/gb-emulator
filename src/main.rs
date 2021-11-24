use std::sync::{Arc, Mutex};
use std::thread;
mod CPU;
mod PPU;

fn main() {
    let lcdc = Arc::new(Mutex::new(0));
    let stat = Arc::new(Mutex::new(0));
    let vram = Arc::new(Mutex::new([0; 6144]));
    let tilemap_1 = Arc::new(Mutex::new([0; 1024]));
    let tilemap_2 = Arc::new(Mutex::new([0; 1024]));
    let scy = Arc::new(Mutex::new(0));
    let scx = Arc::new(Mutex::new(0));
    let ly = Arc::new(Mutex::new(0));
    let lyc = Arc::new(Mutex::new(0));
    let wy = Arc::new(Mutex::new(0));
    let wx = Arc::new(Mutex::new(0));
    let bgp = Arc::new(Mutex::new(0));
    let ime = Arc::new(Mutex::new(0));
    let interrupt_enable = Arc::new(Mutex::new(0));
    let interrupt_flag = Arc::new(Mutex::new(0));

    let lcdc_ppu = Arc::clone(&lcdc);
    let stat_ppu = Arc::clone(&stat);
    let vram_ppu = Arc::clone(&vram);
    let tilemap_1_ppu = Arc::clone(&tilemap_1);
    let tilemap_2_ppu = Arc::clone(&tilemap_2);
    let scy_ppu = Arc::clone(&scy);
    let scx_ppu = Arc::clone(&scx);
    let ly_ppu = Arc::clone(&ly);
    let lyc_ppu = Arc::clone(&lyc);
    let wy_ppu = Arc::clone(&wy);
    let wx_ppu = Arc::clone(&wx);
    let bgp_ppu = Arc::clone(&bgp);
    let ime_ppu = Arc::clone(&ime);
    let interrupt_enable_ppu = Arc::clone(&interrupt_enable);
    let interrupt_flag_ppu = Arc::clone(&interrupt_flag);

    let mut cpu = CPU::CentralProcessingUnit::new(
        lcdc,
        stat,
        vram,
        tilemap_1,
        tilemap_2,
        scy,
        scx,
        ly,
        lyc,
        wy,
        wx,
        bgp,
        ime,
        interrupt_enable,
        interrupt_flag,
    );
    let mut ppu = PPU::PictureProcessingUnit::new(
        lcdc_ppu,
        stat_ppu,
        vram_ppu,
        tilemap_1_ppu,
        tilemap_2_ppu,
        scy_ppu,
        scx_ppu,
        ly_ppu,
        lyc_ppu,
        wy_ppu,
        wx_ppu,
        bgp_ppu,
        interrupt_flag_ppu,
    );
    ppu.run();
    let cpu_handle = thread::spawn(move || loop {
        cpu.run();
    });
    cpu_handle.join().unwrap();
}

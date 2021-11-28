use std::sync::{Arc, Mutex};
use std::thread;
mod cpu;
mod ppu;

fn main() {
    let lcdc = Arc::new(Mutex::new(0));
    let stat = Arc::new(Mutex::new(0));
    let vram = Arc::new(Mutex::new([0; 8192]));
    let oam = Arc::new(Mutex::new([0; 160]));
    let scy = Arc::new(Mutex::new(0));
    let scx = Arc::new(Mutex::new(0));
    let ly = Arc::new(Mutex::new(0));
    let lyc = Arc::new(Mutex::new(0));
    let wy = Arc::new(Mutex::new(0));
    let wx = Arc::new(Mutex::new(0));
    let bgp = Arc::new(Mutex::new(0));
    let obp0 = Arc::new(Mutex::new(0));
    let obp1 = Arc::new(Mutex::new(0));
    let ime = Arc::new(Mutex::new(0));
    let interrupt_enable = Arc::new(Mutex::new(0));
    let interrupt_flag = Arc::new(Mutex::new(0));

    let lcdc_ppu = Arc::clone(&lcdc);
    let stat_ppu = Arc::clone(&stat);
    let vram_ppu = Arc::clone(&vram);
    let oam_ppu = Arc::clone(&oam);
    let scy_ppu = Arc::clone(&scy);
    let scx_ppu = Arc::clone(&scx);
    let ly_ppu = Arc::clone(&ly);
    let lyc_ppu = Arc::clone(&lyc);
    let wy_ppu = Arc::clone(&wy);
    let wx_ppu = Arc::clone(&wx);
    let bgp_ppu = Arc::clone(&bgp);
    let obp0_ppu = Arc::clone(&obp0);
    let obp1_ppu = Arc::clone(&obp1);
    let interrupt_flag_ppu = Arc::clone(&interrupt_flag);

    let mut cpu_instance = cpu::CentralProcessingUnit::new(
        lcdc,
        stat,
        vram,
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
    let mut ppu_instance = ppu::PictureProcessingUnit::new(
        lcdc_ppu,
        stat_ppu,
        vram_ppu,
        oam_ppu,
        scy_ppu,
        scx_ppu,
        ly_ppu,
        lyc_ppu,
        wy_ppu,
        wx_ppu,
        bgp_ppu,
        obp0_ppu,
        obp1_ppu,
        interrupt_flag_ppu,
    );
    let cpu_handle = thread::spawn(move || cpu_instance.run());
    ppu_instance.run();
    cpu_handle.join().unwrap();
}

use std::env;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
mod cpu;
mod dma;
mod lcd;
mod ppu;
use std::time::{Duration, Instant};
mod timing;

const PERIOD_MS: u32 = 5;
const PERIOD_NS: u32 = (PERIOD_MS * 1_000_000) as u32;
const PERIODS_PER_SECOND: u32 = 1000 / PERIOD_MS;
const CYCLES_PER_SECOND: u32 = 4194304;
const CYCLES_PER_PERIOD: u32 = CYCLES_PER_SECOND / PERIODS_PER_SECOND;
const ADVANCE_CYCLES: u32 = 4;
const ADVANCES_PER_PERIOD: u32 = CYCLES_PER_PERIOD / ADVANCE_CYCLES;

fn main() {
    let rom = Arc::new(Mutex::new(Vec::<u8>::new()));
    let external_ram = Arc::new(Mutex::new(vec![0u8; 131072]));
    let internal_ram = Arc::new(Mutex::new([0u8; 8192]));
    let rom_bank = Arc::new(Mutex::new(1usize));
    let ram_bank = Arc::new(Mutex::new(0usize));
    let lcdc = Arc::new(Mutex::new(0u8));
    let stat = Arc::new(Mutex::new(0b0000010u8));
    let vram = Arc::new(Mutex::new([0u8; 8192]));
    let oam = Arc::new(Mutex::new([0u8; 160]));
    let scy = Arc::new(Mutex::new(0u8));
    let scx = Arc::new(Mutex::new(0u8));
    let ly = Arc::new(Mutex::new(0u8));
    let lyc = Arc::new(Mutex::new(0u8));
    let wy = Arc::new(Mutex::new(0u8));
    let wx = Arc::new(Mutex::new(7u8));
    let bgp = Arc::new(Mutex::new(0u8));
    let ime = Arc::new(Mutex::new(0u8));
    let interrupt_enable = Arc::new(Mutex::new(0u8));
    let interrupt_flag = Arc::new(Mutex::new(0u8));
    let p1 = Arc::new(Mutex::new(0xFFu8));
    let div = Arc::new(Mutex::new(0u8));
    let tima = Arc::new(Mutex::new(0u8));
    let tma = Arc::new(Mutex::new(0u8));
    let tac = Arc::new(Mutex::new(0u8));
    let obp0 = Arc::new(Mutex::new(0u8));
    let obp1 = Arc::new(Mutex::new(0u8));
    let dma_transfer = Arc::new(Mutex::new(false));
    let dma_register = Arc::new(Mutex::new(0u8));
    let directional_presses = Arc::new(Mutex::new(0xFu8));
    let action_presses = Arc::new(Mutex::new(0xFu8));

    let (frame_send, frame_recv) = mpsc::channel::<[[u8; 160]; 144]>();

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

    let p1_lcd = Arc::clone(&p1);
    let interrupt_flag_lcd = Arc::clone(&interrupt_flag);
    let directional_presses_lcd = Arc::clone(&directional_presses);
    let action_presses_lcd = Arc::clone(&action_presses);

    let div_timer = Arc::clone(&div);
    let tima_timer = Arc::clone(&tima);
    let tma_timer = Arc::clone(&tma);
    let tac_timer = Arc::clone(&tac);
    let interrupt_flag_timer = Arc::clone(&interrupt_flag);

    let dma_register_dma = Arc::clone(&dma_register);
    let dma_transfer_dma = Arc::clone(&dma_transfer);
    let vram_dma = Arc::clone(&vram);
    let oam_dma = Arc::clone(&oam);
    let rom_dma = Arc::clone(&rom);
    let external_ram_dma = Arc::clone(&external_ram);
    let internal_ram_dma = Arc::clone(&internal_ram);
    let rom_bank_dma = Arc::clone(&rom_bank);
    let ram_bank_dma = Arc::clone(&ram_bank);

    let mut cpu_instance = cpu::CentralProcessingUnit::new(
        rom,
        external_ram,
        internal_ram,
        rom_bank,
        ram_bank,
        lcdc,
        stat,
        vram,
        oam,
        scy,
        scx,
        ly,
        lyc,
        wy,
        wx,
        bgp,
        ime,
        p1,
        div,
        tima,
        tma,
        tac,
        obp0,
        obp1,
        dma_transfer,
        dma_register,
        interrupt_enable,
        interrupt_flag,
        directional_presses,
        action_presses,
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
        frame_send,
    );

    let mut timer_instance = timing::Timer::new(
        div_timer,
        tima_timer,
        tma_timer,
        tac_timer,
        interrupt_flag_timer,
    );

    let mut dma_instance = dma::DirectMemoryAccess::new(
        dma_register_dma,
        dma_transfer_dma,
        vram_dma,
        oam_dma,
        rom_dma,
        external_ram_dma,
        internal_ram_dma,
        rom_bank_dma,
        ram_bank_dma,
    );
    let args: Vec<String> = env::args().collect();
    let mut lcd_instance = lcd::DisplayUnit::new(
        frame_recv,
        interrupt_flag_lcd,
        p1_lcd,
        directional_presses_lcd,
        action_presses_lcd,
    );
    cpu_instance.load_rom(&args[1]);

    thread::spawn(move || loop {
        let now = Instant::now();
        for _ in 0..ADVANCES_PER_PERIOD {
            cpu_instance.advance();
            ppu_instance.advance();
            timer_instance.advance();
            dma_instance.advance();
        }
        spin_sleep::sleep(Duration::new(0, PERIOD_NS).saturating_sub(now.elapsed()));
    });
    lcd_instance.run();
    // thread::spawn(move || cpu_instance.run(&args[1]));
    // thread::spawn(move || timer_instance.run());
    // thread::spawn(move || dma_instance.run());
    // thread::spawn(move || ppu_instance.run());
}

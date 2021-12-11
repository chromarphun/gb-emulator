use std::env;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
mod cpu;
mod dma;
mod lcd;
mod ppu;
mod sound;
use std::time::{Duration, Instant};
mod timing;

const PERIOD_MS: u32 = 5;
const PERIOD_NS: u32 = (PERIOD_MS * 1_000_000) as u32;
const PERIODS_PER_SECOND: u32 = 1000 / PERIOD_MS;
const CYCLES_PER_SECOND: u32 = 4_194_304;
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
    let nr10 = Arc::new(Mutex::new(0u8));
    let nr11 = Arc::new(Mutex::new(0u8));
    let nr12 = Arc::new(Mutex::new(0u8));
    let nr13 = Arc::new(Mutex::new(0u8));
    let nr14 = Arc::new(Mutex::new(0u8));

    let nr21 = Arc::new(Mutex::new(0u8));
    let nr22 = Arc::new(Mutex::new(0u8));
    let nr23 = Arc::new(Mutex::new(0u8));
    let nr24 = Arc::new(Mutex::new(0u8));

    let nr30 = Arc::new(Mutex::new(0u8));
    let nr31 = Arc::new(Mutex::new(0u8));
    let nr32 = Arc::new(Mutex::new(0u8));
    let nr33 = Arc::new(Mutex::new(0u8));
    let nr34 = Arc::new(Mutex::new(0u8));

    let wave_ram = Arc::new(Mutex::new([0; 16]));

    let nr41 = Arc::new(Mutex::new(0u8));
    let nr42 = Arc::new(Mutex::new(0u8));
    let nr43 = Arc::new(Mutex::new(0u8));
    let nr44 = Arc::new(Mutex::new(0u8));

    let nr50 = Arc::new(Mutex::new(0u8));
    let nr52 = Arc::new(Mutex::new(0u8));

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

    let nr10_lcd = Arc::clone(&nr10);
    let nr11_lcd = Arc::clone(&nr11);
    let nr12_lcd = Arc::clone(&nr12);
    let nr13_lcd = Arc::clone(&nr13);
    let nr14_lcd = Arc::clone(&nr14);

    let nr21_lcd = Arc::clone(&nr21);
    let nr22_lcd = Arc::clone(&nr22);
    let nr23_lcd = Arc::clone(&nr23);
    let nr24_lcd = Arc::clone(&nr24);

    let nr30_lcd = Arc::clone(&nr30);
    let nr31_lcd = Arc::clone(&nr31);
    let nr32_lcd = Arc::clone(&nr32);
    let nr33_lcd = Arc::clone(&nr33);
    let nr34_lcd = Arc::clone(&nr34);

    let wave_ram_lcd = Arc::clone(&wave_ram);

    let nr41_lcd = Arc::clone(&nr41);
    let nr42_lcd = Arc::clone(&nr42);
    let nr43_lcd = Arc::clone(&nr43);
    let nr44_lcd = Arc::clone(&nr44);

    let nr50_lcd = Arc::clone(&nr50);
    let nr52_lcd = Arc::clone(&nr52);

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
        nr10,
        nr11,
        nr12,
        nr13,
        nr14,
        nr21,
        nr22,
        nr23,
        nr24,
        nr30,
        nr31,
        nr32,
        nr33,
        nr34,
        wave_ram,
        nr41,
        nr42,
        nr43,
        nr44,
        nr50,
        nr52,
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
    lcd_instance.run(
        nr10_lcd,
        nr11_lcd,
        nr12_lcd,
        nr13_lcd,
        nr14_lcd,
        nr21_lcd,
        nr22_lcd,
        nr23_lcd,
        nr24_lcd,
        nr30_lcd,
        nr31_lcd,
        nr32_lcd,
        nr33_lcd,
        nr34_lcd,
        wave_ram_lcd,
        nr41_lcd,
        nr42_lcd,
        nr43_lcd,
        nr44_lcd,
        nr50_lcd,
        nr52_lcd,
    );
    // thread::spawn(move || cpu_instance.run(&args[1]));
    // thread::spawn(move || timer_instance.run());
    // thread::spawn(move || dma_instance.run());
    // thread::spawn(move || ppu_instance.run());
}

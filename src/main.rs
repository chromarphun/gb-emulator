use sdl2::audio::{AudioQueue, AudioSpecDesired};
use std::env;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
mod apu;
mod cpu;
mod dma;
mod epu;
mod ppu;
mod sound;
use std::time::{Duration, Instant};
mod pdu;
mod timing;

const PERIOD_MS: u32 = 5;
//const PERIOD_NS: u32 = (PERIOD_MS * 1_000_000) as u32;
//const PERIODS_PER_SECOND: u32 = 1000 / PERIOD_MS;
const PERIODS_PER_SECOND: u32 = 64;
const PERIOD_NS: u32 = 1_000_000_000 / PERIODS_PER_SECOND;
const CYCLES_PER_SECOND: u32 = 4_194_304;
const CYCLES_PER_PERIOD: u32 = CYCLES_PER_SECOND / PERIODS_PER_SECOND;
const ADVANCE_CYCLES: u32 = 4;
const ADVANCES_PER_PERIOD: u32 = CYCLES_PER_PERIOD / ADVANCE_CYCLES;
const WINDOW_WIDTH: usize = 160;
const WINDOW_HEIGHT: usize = 144;
const SAMPLES_PER_SECOND: u32 = 44100;
const CYCLES_PER_SAMPLE: u32 = CYCLES_PER_SECOND / SAMPLES_PER_SECOND;

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

    let frame_send = Arc::new(Mutex::new([[0; 160]; 144]));

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

    let p1_epu = Arc::clone(&p1);
    let interrupt_flag_epu = Arc::clone(&interrupt_flag);
    let directional_presses_epu = Arc::clone(&directional_presses);
    let action_presses_epu = Arc::clone(&action_presses);

    let stat_pdu = Arc::clone(&stat);
    let frame_send_pdu = Arc::clone(&frame_send);

    let nr10_apu = Arc::clone(&nr10);
    let nr11_apu = Arc::clone(&nr11);
    let nr12_apu = Arc::clone(&nr12);
    let nr13_apu = Arc::clone(&nr13);
    let nr14_apu = Arc::clone(&nr14);

    let nr21_apu = Arc::clone(&nr21);
    let nr22_apu = Arc::clone(&nr22);
    let nr23_apu = Arc::clone(&nr23);
    let nr24_apu = Arc::clone(&nr24);

    let nr30_apu = Arc::clone(&nr30);
    let nr31_apu = Arc::clone(&nr31);
    let nr32_apu = Arc::clone(&nr32);
    let nr33_apu = Arc::clone(&nr33);
    let nr34_apu = Arc::clone(&nr34);

    let wave_ram_apu = Arc::clone(&wave_ram);

    let nr41_apu = Arc::clone(&nr41);
    let nr42_apu = Arc::clone(&nr42);
    let nr43_apu = Arc::clone(&nr43);
    let nr44_apu = Arc::clone(&nr44);

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

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let window = video_subsystem
        .window(
            "Gameboy Emulator",
            WINDOW_WIDTH as u32,
            WINDOW_HEIGHT as u32,
        )
        .position_centered()
        .build()
        .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut pdu_instance = pdu::PictureDisplayUnit::new(canvas, stat_pdu, frame_send_pdu);

    let mut running = Arc::new(Mutex::new(true));
    let mut running_epu = Arc::clone(&running);

    let mut epu_instance = epu::EventProcessingUnit::new(
        p1_epu,
        directional_presses_epu,
        action_presses_epu,
        interrupt_flag_epu,
        running_epu,
        event_pump,
    );

    let mut apu_instance = apu::AudioProcessingUnit::new(
        audio_subsystem,
        nr10_apu,
        nr11_apu,
        nr12_apu,
        nr13_apu,
        nr14_apu,
        nr21_apu,
        nr22_apu,
        nr23_apu,
        nr24_apu,
        nr30_apu,
        nr31_apu,
        nr32_apu,
        nr33_apu,
        nr34_apu,
        wave_ram_apu,
        nr41_apu,
        nr42_apu,
        nr43_apu,
        nr44_apu,
    );

    cpu_instance.load_rom(&args[1]);
    let period_time = Duration::new(0, PERIOD_NS);
    while *running.lock().unwrap() {
        let now = Instant::now();
        for _ in 0..ADVANCES_PER_PERIOD {
            cpu_instance.advance();
            ppu_instance.advance();
            timer_instance.advance();
            dma_instance.advance();
            pdu_instance.advance();
            apu_instance.advance();
        }
        epu_instance.total_advance();
        //while now.elapsed() < period_time {}
        spin_sleep::sleep(Duration::new(0, PERIOD_NS).saturating_sub(now.elapsed()));
    }

    //lcd_instance.run();
    // thread::spawn(move || cpu_instance.run(&args[1]));
    // thread::spawn(move || timer_instance.run());
    // thread::spawn(move || dma_instance.run());
    // thread::spawn(move || ppu_instance.run());
}

use pixels::Pixels;
use sdl2::video::Window;

use crate::apu::AudioProcessingUnit;
use crate::constants::*;
use crate::cpu::CentralProcessingUnit;
use crate::epu::EventProcessingUnit;
use crate::memory::MemoryUnit;
use crate::ppu::PictureProcessingUnit;
use crate::timing::Timer;
use std::time::{Duration, Instant};

#[derive(PartialEq, Debug)]
pub enum RequestSource {
    APU,
    CPU,
    EPU,
    MAU,
    PPU,
    SPEC,
    Timer,
}
pub struct GameBoyEmulator {
    pub cpu: CentralProcessingUnit,
    pub mem_unit: MemoryUnit,
    pub ppu: PictureProcessingUnit,
    pub epu: EventProcessingUnit,
    pub apu: AudioProcessingUnit,
    pub timer: Timer,
    pub sdl_context: sdl2::Sdl,
    pub double_speed: bool,
    pub cgb: bool,
    pub running: bool,
    _window: Window,
    pub pixels: Pixels,
    pub iteration_count: usize,
}

impl GameBoyEmulator {
    pub fn new() -> GameBoyEmulator {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();
        let mut window = video_subsystem
            .window(
                "Gameboy Emulator",
                WINDOW_WIDTH as u32,
                WINDOW_HEIGHT as u32,
            )
            .position_centered()
            .resizable()
            .allow_highdpi()
            .build()
            .unwrap();
        window
            .set_minimum_size(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
            .unwrap();
        let surface_texture =
            pixels::SurfaceTexture::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, &window);
        let pixels =
            pixels::Pixels::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, surface_texture)
                .unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        GameBoyEmulator {
            cpu: CentralProcessingUnit::new(),
            mem_unit: MemoryUnit::new(),
            ppu: PictureProcessingUnit::new(),
            //pdu: PictureDisplayUnit::new(pixels),
            epu: EventProcessingUnit::new(event_pump),
            timer: Timer::new(),
            sdl_context,
            apu: AudioProcessingUnit::new(audio_subsystem),
            double_speed: false,
            cgb: false,
            running: true,
            _window: window,
            pixels,
            iteration_count: 0,
        }
    }
    pub fn run(&mut self) {
        let work_period = Duration::new(0, PERIOD_NS);

        while self.running {
            let now = Instant::now();
            println!("Buffer start size: {}", self.apu.ch_1_queue.size());
            self.buffer_check();
            for _ in 0..ADVANCES_PER_PERIOD {
                self.cpu_advance();
                self.timer_advance();
                self.dma_tick();
                if self.double_speed {
                    self.cpu_advance();
                    self.timer_advance();
                    self.dma_tick();
                }
                self.apu_advance();
                self.ppu_advance();
                self.iteration_count += 1;
            }
            self.event_check();
            self.send_to_queue();
            println!("Buffer end size: {}", self.apu.ch_1_queue.size());
            // println!(
            //     "elapsed: {}us",
            //     work_period.saturating_sub(now.elapsed()).as_micros()
            // );
            spin_sleep::sleep(work_period.saturating_sub(now.elapsed()));
        }
    }
}

use crate::apu::AudioProcessingUnit;
use crate::cpu::CentralProcessingUnit;
use crate::epu::EventProcessingUnit;
use crate::memory::MemoryUnit;
use crate::pdu::PictureDisplayUnit;
use crate::ppu::PictureProcessingUnit;
use crate::timing::Timer;
use std::fs::File;
use std::io::prelude::*;
use std::io::Write;
use std::time::{Duration, Instant};

const PERIODS_PER_SECOND: u32 = 64;
const PERIOD_NS: u32 = 1_000_000_000 / PERIODS_PER_SECOND;
const CYCLES_PER_SECOND: u32 = 4_194_304;
const CYCLES_PER_PERIOD: u32 = CYCLES_PER_SECOND / PERIODS_PER_SECOND;
pub const ADVANCE_CYCLES: u32 = 4;
const ADVANCES_PER_PERIOD: u32 = CYCLES_PER_PERIOD / ADVANCE_CYCLES;
const WINDOW_WIDTH: usize = 160;
const WINDOW_HEIGHT: usize = 144;
const SAMPLES_PER_SECOND: u32 = 44100;
pub const CYCLES_PER_SAMPLE: u32 = CYCLES_PER_SECOND / SAMPLES_PER_SECOND;

pub struct GameBoyEmulator {
    pub cpu: CentralProcessingUnit,
    pub mem_unit: MemoryUnit,
    pub ppu: PictureProcessingUnit,
    pub pdu: PictureDisplayUnit,
    pub epu: EventProcessingUnit,
    pub apu: AudioProcessingUnit,
    pub timer: Timer,
    pub sdl_context: sdl2::Sdl,
    pub log: File,
    pub frame: [[u8; 160]; 144],
    pub running: bool,
}

impl GameBoyEmulator {
    pub fn new() -> GameBoyEmulator {
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
        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        GameBoyEmulator {
            cpu: CentralProcessingUnit::new(),
            mem_unit: MemoryUnit::new(),
            ppu: PictureProcessingUnit::new(),
            pdu: PictureDisplayUnit::new(canvas),
            epu: EventProcessingUnit::new(event_pump),
            timer: Timer::new(),
            sdl_context: sdl_context,
            apu: AudioProcessingUnit::new(audio_subsystem),
            log: File::create(
                "C://Users//chrom//Documents//Emulators//gb-emulator//src//commands.log",
            )
            .expect("Unable to create file"),
            frame: [[0; 160]; 144],
            running: true,
        }
    }
    pub fn load_rom(&mut self, path: &str) {
        self.mem_unit.load_rom(path);
    }
    pub fn run(&mut self) {
        let work_period = Duration::new(0, PERIOD_NS);

        while self.running {
            let now = Instant::now();
            for _ in 0..ADVANCES_PER_PERIOD {
                self.cpu_advance();
                self.ppu_advance();
                self.pdu_advance();
                self.timer_advance();
                self.apu_advance();
                self.mem_unit.dma_tick();
            }
            self.event_check();
            spin_sleep::sleep(work_period.saturating_sub(now.elapsed()));
        }
    }
}

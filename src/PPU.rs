use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use std::sync::{Arc, Mutex};
use std::time::Instant;

const COLOR_MAP: [Color; 4] = [
    Color::RGB(155, 188, 15),
    Color::RGB(139, 172, 15),
    Color::RGB(48, 98, 48),
    Color::RGB(15, 56, 15),
];

const TILES_PER_ROW: usize = 32;
const BG_MAP_SIZE_PX: usize = 256;
const BG_TILE_WIDTH: usize = 8;
const BG_TILE_HEIGHT: usize = 8;
const BYTES_PER_TILE: usize = 16;
const BYTES_PER_TILE_ROW: usize = 2;
const SCREEN_PX_WIDTH: usize = 160;
const SCREEN_PX_HEIGHT: usize = 144;
const VRAM_BLOCK_SIZE: usize = 128;
const NANOS_PER_DOT: f64 = 238.4185791015625;
const OAM_SCAN_DOTS: u16 = 80;
const DRAWING_DOTS: u16 = 172;
const HBLANK_DOTS: u16 = 204;
const ROW_DOTS: u16 = 456;
const TOTAL_DOTS: u32 = 77520;
const WINDOW_WIDTH: u32 = 160;
const WINDOW_HEIGHT: u32 = 144;

pub struct PictureProcessingUnit {
    lcdc: Arc<Mutex<u8>>,
    stat: Arc<Mutex<u8>>,
    vram: Arc<Mutex<[u8; 8192]>>,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    scy: Arc<Mutex<u8>>,
    scx: Arc<Mutex<u8>>,
    ly: Arc<Mutex<u8>>,
    lyc: Arc<Mutex<u8>>,
    wy: Arc<Mutex<u8>>,
    wx: Arc<Mutex<u8>>,
    bgp: Arc<Mutex<u8>>,
    interrupt_flag: Arc<Mutex<u8>>,
}

impl PictureProcessingUnit {
    pub fn new(
        lcdc: Arc<Mutex<u8>>,
        stat: Arc<Mutex<u8>>,
        vram: Arc<Mutex<[u8; 8192]>>,
        scy: Arc<Mutex<u8>>,
        scx: Arc<Mutex<u8>>,
        ly: Arc<Mutex<u8>>,
        lyc: Arc<Mutex<u8>>,
        wy: Arc<Mutex<u8>>,
        wx: Arc<Mutex<u8>>,
        bgp: Arc<Mutex<u8>>,
        interrupt_flag: Arc<Mutex<u8>>,
    ) -> PictureProcessingUnit {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Gameboy Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        PictureProcessingUnit {
            lcdc,
            stat,
            vram,
            canvas,
            event_pump,
            scy,
            scx,
            ly,
            lyc,
            wy,
            wx,
            bgp,
            interrupt_flag,
        }
    }
    fn get_tile_map_flag(&self) -> u8 {
        (*self.lcdc.lock().unwrap() >> 3) & 1
    }
    fn get_tile_data_flag(&self) -> u8 {
        (*self.lcdc.lock().unwrap() >> 4) & 1
    }
    fn get_lcd_enable_flag(&self) -> u8 {
        (*self.lcdc.lock().unwrap() >> 7) & 1
    }
    fn get_stat_lyc_lc_int_flag(&self) -> u8 {
        (*self.stat.lock().unwrap() >> 6) & 1
    }
    fn get_stat_oam_int_flag(&self) -> u8 {
        (*self.stat.lock().unwrap() >> 5) & 1
    }
    fn get_stat_vblank_int_flag(&self) -> u8 {
        (*self.stat.lock().unwrap() >> 4) & 1
    }
    fn get_stat_hblank_int_flag(&self) -> u8 {
        (*self.stat.lock().unwrap() >> 3) & 1
    }
    pub fn run(&mut self) {
        loop {
            let start = Instant::now();
            let mut now = Instant::now();
            //PIXEL DRAWING
            for row in 0..SCREEN_PX_HEIGHT {
                //OAM SCAN PERIOD
                now = Instant::now();
                if self.get_stat_oam_int_flag() == 1 {
                    *self.interrupt_flag.lock().unwrap() |= 0b00010;
                }

                while (now.elapsed().as_nanos()) < (OAM_SCAN_DOTS as f64 * NANOS_PER_DOT) as u128 {}

                //create context for vram lock to exist
                {
                    now = Instant::now();
                    {
                        *self.ly.lock().unwrap() = row as u8;
                        if *self.lyc.lock().unwrap() == row as u8 {
                            *self.stat.lock().unwrap() |= 0b1000000;
                            if self.get_stat_lyc_lc_int_flag() == 1 {
                                *self.interrupt_flag.lock().unwrap() |= 0b00010;
                            }
                        } else {
                            *self.stat.lock().unwrap() &= 0b0111111;
                        }
                    }
                    let vram = if self.get_lcd_enable_flag() == 1 {
                        *self.vram.lock().unwrap()
                    } else {
                        [0u8; 8192]
                    };
                    let bgp = *self.bgp.lock().unwrap() as usize;
                    let color_indexes: [usize; 4] = [
                        (bgp >> 0) & 0b11,
                        (bgp >> 2) & 0b11,
                        (bgp >> 4) & 0b11,
                        (bgp >> 6) & 0b11,
                    ];
                    let (scx, scy) = {
                        (
                            *self.scx.lock().unwrap() as usize,
                            *self.scy.lock().unwrap() as usize,
                        )
                    };
                    let tile_data_flag = self.get_tile_data_flag();
                    let tilemap_start: usize = if self.get_tile_map_flag() == 0 {
                        6144
                    } else {
                        7168
                    };
                    let total_row: usize = (scy + row) as usize % BG_MAP_SIZE_PX;
                    let mut column: usize = 0;
                    let mut total_column = (scx + column) % BG_MAP_SIZE_PX;
                    let mut px_within_row = column % BG_TILE_WIDTH;
                    while column < SCREEN_PX_WIDTH {
                        let tile_map_index = TILES_PER_ROW * (total_row / BG_TILE_HEIGHT) // getting to the right row for a tile
                            + (total_column / BG_TILE_WIDTH); // getting to the right column for a tile
                        let absolute_tile_index = if tile_data_flag == 1 {
                            vram[tilemap_start + tile_map_index as usize] as usize
                        } else {
                            let initial_index =
                                vram[tilemap_start + tile_map_index as usize] as usize;
                            if initial_index < VRAM_BLOCK_SIZE {
                                initial_index + 2 * VRAM_BLOCK_SIZE
                            } else {
                                initial_index
                            }
                        };
                        let tile_data_index = absolute_tile_index * BYTES_PER_TILE // getting to the starting byte
                            + (total_row % BG_TILE_WIDTH) * BYTES_PER_TILE_ROW; //getting to the row

                        let least_sig_byte = vram[tile_data_index as usize];
                        let most_sig_byte = vram[(tile_data_index + 1) as usize];
                        while px_within_row < BG_TILE_WIDTH && column < SCREEN_PX_WIDTH {
                            total_column = (scx + column) % BG_MAP_SIZE_PX;

                            let bgp_index =
                                ((((most_sig_byte >> (BG_TILE_WIDTH - px_within_row - 1)) & 1)
                                    << 1)
                                    + ((least_sig_byte >> (BG_TILE_WIDTH - px_within_row - 1)) & 1))
                                    as usize;
                            self.canvas
                                .set_draw_color(COLOR_MAP[color_indexes[bgp_index]]);
                            if self.get_lcd_enable_flag() == 1 {
                                self.canvas
                                    .fill_rect(Rect::new(column as i32, row as i32, 1, 1))
                                    .expect("Failure to draw");
                            }
                            //self.canvas.present();
                            column += 1;
                            px_within_row += 1;
                        }
                        px_within_row = 0;
                    }
                    //spin while we're waiting for drawing pixel period to end
                    //vram is still locked!

                    while (now.elapsed().as_nanos()) < (DRAWING_DOTS as f64 * NANOS_PER_DOT) as u128
                    {
                    }
                }
                //HBLANK
                //we've left vram context and now vram is accessible during HBLANK
                now = Instant::now();
                if self.get_stat_hblank_int_flag() == 1 {
                    *self.interrupt_flag.lock().unwrap() |= 0b00010;
                }
                self.canvas.present();
                while (now.elapsed().as_nanos()) < (HBLANK_DOTS as f64 * NANOS_PER_DOT) as u128 {}
            }
            //VBLANK

            now = Instant::now();
            // let cycles = (start.elapsed().as_nanos()) / NANOS_PER_DOT as u128;
            // println!("{}", cycles);
            *self.interrupt_flag.lock().unwrap() |= 0b00001;
            if self.get_stat_vblank_int_flag() == 1 {
                *self.interrupt_flag.lock().unwrap() |= 0b00010;
            }
            while (now.elapsed().as_nanos()) < (ROW_DOTS as f64 * NANOS_PER_DOT) as u128 {}
            *self.ly.lock().unwrap() += 1;

            for _ in 0..9 {
                now = Instant::now();
                while (now.elapsed().as_nanos()) < (ROW_DOTS as f64 * NANOS_PER_DOT) as u128 {}
                *self.ly.lock().unwrap() += 1;
            }
        }
    }
}

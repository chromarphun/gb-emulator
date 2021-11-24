use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use std::sync::{Arc, Mutex};
use std::time::Instant;

const color_map: [Color; 4] = [
    Color::RGB(15, 56, 15),
    Color::RGB(48, 98, 48),
    Color::RGB(139, 172, 15),
    Color::RGB(155, 188, 15),
];

const TILES_PER_ROW: usize = 32;
const TILES_PER_COLUMN: usize = 32;
const BG_MAP_SIZE_PX: usize = 256;
const BG_TILE_WIDTH: usize = 8;
const BG_TILE_HEIGHT: usize = 8;
const BYTES_PER_TILE: usize = 16;
const BYTES_PER_TILE_ROW: usize = 2;
const SCREEN_PX_WIDTH: usize = 160;
const SCREEN_PX_HEIGHT: usize = 144;
const TILES_PER_MAP: usize = 256;
const VRAM_BLOCK_SIZE: usize = 128;
const NANOS_PER_DOT: f64 = 238.4185791015625;
const OAM_SCAN_DOTS: u16 = 80;
const DRAWING_DOTS: u16 = 172;
const HBLANK_DOTS: u16 = 204;
const VBLANK_DOTS: u16 = 4560;
const TOTAL_DOTS: u32 = 77520;
const WINDOW_WIDTH: u32 = 160;
const WINDOW_HEIGHT: u32 = 144;

pub struct PictureProcessingUnit {
    lcdc: Arc<Mutex<u8>>,
    stat: Arc<Mutex<u8>>,
    vram: Arc<Mutex<[u8; 6144]>>,
    tilemap_1: Arc<Mutex<[u8; 1024]>>,
    tilemap_2: Arc<Mutex<[u8; 1024]>>,
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
        vram: Arc<Mutex<[u8; 6144]>>,
        tilemap_1: Arc<Mutex<[u8; 1024]>>,
        tilemap_2: Arc<Mutex<[u8; 1024]>>,
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

        let mut canvas = window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        PictureProcessingUnit {
            lcdc,
            stat,
            vram,
            tilemap_1,
            tilemap_2,
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
    fn set_lyc_eq_lc_flag(&mut self) {
        *self.stat.lock().unwrap() == 1;
    }
    fn reset_lyc_eq_lc_flag(&mut self) {
        *self.stat.lock().unwrap() == 0;
    }
    fn trigger_vblank_int(&mut self) {
        *self.interrupt_flag.lock().unwrap() |= 0b00001;
    }
    fn trigger_lcd_stat_int(&mut self) {
        *self.interrupt_flag.lock().unwrap() |= 0b00010;
    }
    fn trigger_timer_int(&mut self) {
        *self.interrupt_flag.lock().unwrap() |= 0b00100;
    }
    fn trigger_serial_int(&mut self) {
        *self.interrupt_flag.lock().unwrap() |= 0b01000;
    }
    fn trigger_joypad_int(&mut self) {
        *self.interrupt_flag.lock().unwrap() |= 0b10000;
    }
    pub fn run(&mut self) {
        'frame_loop: loop {
            let mut now = Instant::now();
            if self.get_lcd_enable_flag() == 0 {
                while (now.elapsed().as_nanos()) < (TOTAL_DOTS as f64 * NANOS_PER_DOT) as u128 {}
            } else {
                //OAM SCAN PERIOD
                if self.get_stat_oam_int_flag() == 1 {
                    *self.interrupt_flag.lock().unwrap() |= 0b00010;
                }
                while (now.elapsed().as_nanos()) < (OAM_SCAN_DOTS as f64 * NANOS_PER_DOT) as u128 {}
                //PIXEL DRAWING
                for row in 0..160 {
                    //create context for vram lock to exist
                    {
                        now = Instant::now();
                        let vram = self.vram.lock().unwrap();
                        let bgp = *self.bgp.lock().unwrap() as usize;
                        let color_indexes: [usize; 4] = [
                            (bgp >> 6) & 0b11,
                            (bgp >> 4) & 0b11,
                            (bgp >> 2) & 0b11,
                            (bgp >> 0) & 0b11,
                        ];
                        let (scx, scy) = {
                            (
                                *self.scx.lock().unwrap() as usize,
                                *self.scy.lock().unwrap() as usize,
                            )
                        };
                        let tilemap = if self.get_tile_map_flag() == 0 {
                            self.tilemap_1.lock().unwrap()
                        } else {
                            self.tilemap_2.lock().unwrap()
                        };
                        let total_row: usize = (scy + row) as usize % BG_MAP_SIZE_PX;
                        let mut column: usize = 0;
                        let mut total_column = (scx + column) % BG_MAP_SIZE_PX;
                        let mut px_within_row = column % BG_TILE_WIDTH;
                        while column < SCREEN_PX_WIDTH {
                            let tile_map_index = TILES_PER_ROW * (total_row / BG_TILE_HEIGHT) // getting to the right row for a tile
                                * (total_column / BG_TILE_WIDTH); // getting to the right column for a tile

                            let absolute_tile_index = if self.get_tile_data_flag() == 0 {
                                tilemap[tile_map_index as usize] as usize
                            } else {
                                let initial_index = tilemap[tile_map_index as usize] as usize;
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
                                px_within_row = column % BG_TILE_WIDTH;
                                let bgp_index =
                                    ((((most_sig_byte >> (BG_TILE_WIDTH - px_within_row - 1)) & 1)
                                        << 1)
                                        + ((least_sig_byte >> (BG_TILE_WIDTH - px_within_row - 1))
                                            & 1)) as usize;
                                self.canvas
                                    .set_draw_color(color_map[color_indexes[bgp_index]]);
                                self.canvas
                                    .fill_rect(Rect::new(column as i32, row as i32, 1, 1))
                                    .expect("Failure to draw");
                                column += 1;
                                {
                                    *self.ly.lock().unwrap() = column as u8;
                                    if *self.lyc.lock().unwrap() == column as u8 {
                                        *self.stat.lock().unwrap() |= 0b1000000;
                                        if self.get_stat_lyc_lc_int_flag() == 1 {
                                            *self.interrupt_flag.lock().unwrap() |= 0b00010;
                                        }
                                    } else {
                                        *self.stat.lock().unwrap() &= 0b0111111;
                                    }
                                }
                            }
                        }
                        //spin while we're waiting for drawing pixel period to end
                        //vram is still locked!
                        while (now.elapsed().as_nanos())
                            < (DRAWING_DOTS as f64 * NANOS_PER_DOT) as u128
                        {
                        }
                    }
                    //HBLANK
                    //we've left vram context and now vram is accessible during HBLANK
                    now = Instant::now();
                    if self.get_stat_hblank_int_flag() == 1 {
                        *self.interrupt_flag.lock().unwrap() |= 0b00010;
                    }
                    while (now.elapsed().as_nanos()) < (HBLANK_DOTS as f64 * NANOS_PER_DOT) as u128
                    {
                    }
                }
                //VBLANK
                now = Instant::now();
                self.trigger_vblank_int();
                if self.get_stat_vblank_int_flag() == 1 {
                    *self.interrupt_flag.lock().unwrap() |= 0b00010;
                }
                while (now.elapsed().as_nanos()) < (VBLANK_DOTS as f64 * NANOS_PER_DOT) as u128 {}
            }
        }
    }
}

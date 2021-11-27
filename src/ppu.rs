use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
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
        PictureProcessingUnit {
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
            interrupt_flag,
        }
    }
    fn get_bg_tile_map_flag(&self) -> u8 {
        (*self.lcdc.lock().unwrap() >> 3) & 1
    }
    fn get_win_tile_map_flag(&self) -> u8 {
        (*self.lcdc.lock().unwrap() >> 6) & 1
    }
    fn get_tile_data_flag(&self) -> u8 {
        (*self.lcdc.lock().unwrap() >> 4) & 1
    }
    fn get_win_enable_flag(&self) -> u8 {
        (*self.lcdc.lock().unwrap() >> 5) & 1
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
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Gameboy Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        'running: loop {
            let start_time = Instant::now();
            //PIXEL DRAWING

            for row in 0..SCREEN_PX_HEIGHT {
                //OAM SCAN PERIOD
                let mut now = Instant::now();
                if self.get_stat_oam_int_flag() == 1 {
                    *self.interrupt_flag.lock().unwrap() |= 0b00010;
                }

                while (now.elapsed().as_nanos()) < (OAM_SCAN_DOTS as f64 * NANOS_PER_DOT) as u128 {}

                //create context for vram lock to exist
                {
                    now = Instant::now();
                    *self.ly.lock().unwrap() = row as u8;
                    if *self.lyc.lock().unwrap() == row as u8 {
                        *self.stat.lock().unwrap() |= 0b1000000;
                        if self.get_stat_lyc_lc_int_flag() == 1 {
                            *self.interrupt_flag.lock().unwrap() |= 0b00010;
                        }
                    } else {
                        *self.stat.lock().unwrap() &= 0b0111111;
                    }
                    let wx = *self.wx.lock().unwrap() as usize;
                    let wy = *self.wy.lock().unwrap() as usize;
                    let tile_num_begin_window = (wx - 7) / BG_TILE_WIDTH;
                    let window_activated = wy >= row && self.get_win_enable_flag() == 1;

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
                    let bg_tilemap_start: usize = if self.get_bg_tile_map_flag() == 0 {
                        6144
                    } else {
                        7168
                    };
                    let win_tilemap_start: usize = if self.get_win_tile_map_flag() == 0 {
                        6144
                    } else {
                        7168
                    };
                    let total_bg_row: usize = (scy + row) as usize % BG_MAP_SIZE_PX;

                    let total_bg_column = scx % BG_MAP_SIZE_PX;

                    let px_within_row = total_bg_column % BG_TILE_WIDTH;

                    let extra_tile = px_within_row != 0;

                    let extra_end_index = BG_TILE_WIDTH - px_within_row;

                    let mut end_index = 8;

                    let starting_tile_map_index = TILES_PER_ROW * (total_bg_row / BG_TILE_HEIGHT)
                        + (total_bg_column / BG_TILE_WIDTH);

                    let row_within_tile = total_bg_row % BG_TILE_WIDTH;

                    let mut column: i32 = 0;

                    for tile_num in 0..21 {
                        let (tile_map_index, tilemap_start) =
                            if window_activated && tile_num >= tile_num_begin_window {
                                (tile_num, win_tilemap_start)
                            } else {
                                (starting_tile_map_index + tile_num, bg_tilemap_start)
                            };
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
                        + row_within_tile * BYTES_PER_TILE_ROW; //getting to the row
                        let least_sig_byte = vram[tile_data_index as usize];
                        let most_sig_byte = vram[(tile_data_index + 1) as usize];
                        for pixel in px_within_row..end_index {
                            let bgp_index = ((((most_sig_byte >> (BG_TILE_WIDTH - pixel - 1)) & 1)
                                << 1)
                                + ((least_sig_byte >> (BG_TILE_WIDTH - pixel - 1)) & 1))
                                as usize;
                            canvas.set_draw_color(COLOR_MAP[color_indexes[bgp_index]]);
                            canvas
                                .draw_point(Point::new(column, row as i32))
                                .expect("Failed drawing");
                            column += 1;
                        }
                        if tile_num == 19 {
                            if extra_tile {
                                end_index = extra_end_index;
                            } else {
                                break;
                            }
                        }
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
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            scancode: Some(Scancode::Escape),
                            ..
                        } => break 'running,
                        _ => {}
                    }
                }
                while (now.elapsed().as_nanos()) < (HBLANK_DOTS as f64 * NANOS_PER_DOT) as u128 {}
            }
            //VBLANK

            let mut now = Instant::now();

            // let cycles = (start.elapsed().as_nanos()) / NANOS_PER_DOT as u128;
            // println!("{}", cycles);
            *self.interrupt_flag.lock().unwrap() |= 0b00001;
            if self.get_stat_vblank_int_flag() == 1 {
                *self.interrupt_flag.lock().unwrap() |= 0b00010;
            }
            while (now.elapsed().as_nanos()) < (ROW_DOTS as f64 * NANOS_PER_DOT) as u128 {}
            now = Instant::now();
            *self.ly.lock().unwrap() += 1;

            while (now.elapsed().as_nanos()) < (ROW_DOTS as f64 * NANOS_PER_DOT) as u128 {}

            *self.ly.lock().unwrap() += 1;

            canvas.present();

            while (start_time.elapsed().as_nanos()) < (TOTAL_DOTS as f64 * NANOS_PER_DOT) as u128 {
                *self.ly.lock().unwrap() =
                    ((start_time.elapsed().as_nanos() / NANOS_PER_DOT as u128 - 65664) / 10) as u8;
            }
        }
    }
}

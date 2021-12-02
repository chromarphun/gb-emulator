use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::cmp;
use std::convert::TryInto;
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
const TILE_WIDTH: usize = 8;
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
const BYTES_PER_OAM_ENTRY: usize = 4;
const SPIRTES_IN_OAM: usize = 40;
const OAM_Y_INDEX: usize = 0;
const OAM_X_INDEX: usize = 1;
const OAM_TILE_INDEX: usize = 2;
const OAM_ATTRIBUTE_INDEX: usize = 3;

pub struct PictureProcessingUnit {
    lcdc: Arc<Mutex<u8>>,
    stat: Arc<Mutex<u8>>,
    vram: Arc<Mutex<[u8; 8192]>>,
    oam: Arc<Mutex<[u8; 160]>>,
    scy: Arc<Mutex<u8>>,
    scx: Arc<Mutex<u8>>,
    ly: Arc<Mutex<u8>>,
    lyc: Arc<Mutex<u8>>,
    wy: Arc<Mutex<u8>>,
    wx: Arc<Mutex<u8>>,
    bgp: Arc<Mutex<u8>>,
    obp0: Arc<Mutex<u8>>,
    obp1: Arc<Mutex<u8>>,
    p1: Arc<Mutex<u8>>,
    interrupt_flag: Arc<Mutex<u8>>,
}

impl PictureProcessingUnit {
    pub fn new(
        lcdc: Arc<Mutex<u8>>,
        stat: Arc<Mutex<u8>>,
        vram: Arc<Mutex<[u8; 8192]>>,
        oam: Arc<Mutex<[u8; 160]>>,
        scy: Arc<Mutex<u8>>,
        scx: Arc<Mutex<u8>>,
        ly: Arc<Mutex<u8>>,
        lyc: Arc<Mutex<u8>>,
        wy: Arc<Mutex<u8>>,
        wx: Arc<Mutex<u8>>,
        bgp: Arc<Mutex<u8>>,
        obp0: Arc<Mutex<u8>>,
        obp1: Arc<Mutex<u8>>,
        p1: Arc<Mutex<u8>>,
        interrupt_flag: Arc<Mutex<u8>>,
    ) -> PictureProcessingUnit {
        PictureProcessingUnit {
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
            obp0,
            obp1,
            interrupt_flag,
            p1,
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
    fn get_obj_size(&self) -> u8 {
        if ((*self.lcdc.lock().unwrap() >> 2) & 1) == 1 {
            16
        } else {
            8
        }
    }
    fn get_lcd_enable_flag(&self) -> u8 {
        (*self.lcdc.lock().unwrap() >> 7) & 1
    }
    fn get_sprite_enable_flag(&self) -> u8 {
        (*self.lcdc.lock().unwrap() >> 1) & 1
    }
    fn get_bg_window_enable(&self) -> u8 {
        *self.lcdc.lock().unwrap() & 1
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
                let mut now = Instant::now();
                //create context for vram/oam lock to exist
                {
                    //OAM SCAN PERIOD

                    if self.get_stat_oam_int_flag() == 1 {
                        *self.interrupt_flag.lock().unwrap() |= 0b00010;
                    }
                    let mut possible_sprites: Vec<[u8; 4]> = Vec::new();
                    let oam = self.oam.lock().unwrap();
                    let obj_length = self.get_obj_size();
                    let mut sprite_num = 0;
                    'sprite_loop: for i in 0..SPIRTES_IN_OAM {
                        if (oam[i * BYTES_PER_OAM_ENTRY] > (row + 16) as u8)
                            && (oam[i * BYTES_PER_OAM_ENTRY] < (row + 16) as u8 + obj_length)
                        {
                            possible_sprites.push(
                                oam[(i * BYTES_PER_OAM_ENTRY)..((i + 1) * BYTES_PER_OAM_ENTRY)]
                                    .try_into()
                                    .expect("Indexing error"),
                            );
                            sprite_num += 1;
                            if sprite_num == 10 {
                                break 'sprite_loop;
                            }
                        }
                    }
                    while (now.elapsed().as_nanos())
                        < (OAM_SCAN_DOTS as f64 * NANOS_PER_DOT) as u128
                    {}
                    // DRAWOMG PERIOD
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
                    let mut row_colors = [0u8; 160];
                    let vram = if self.get_lcd_enable_flag() == 1 {
                        *self.vram.lock().unwrap()
                    } else {
                        [0u8; 8192]
                    };
                    if self.get_bg_window_enable() == 1 {
                        let wx = *self.wx.lock().unwrap() as usize;
                        let wy = *self.wy.lock().unwrap() as usize;
                        let tile_num_begin_window =
                            (cmp::min(wx as i16 - 7, 0)) as usize / TILE_WIDTH;
                        let window_activated = wy >= row && self.get_win_enable_flag() == 1;

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

                        let mut px_within_row = total_bg_column % TILE_WIDTH;

                        let extra_tile = px_within_row != 0;

                        let extra_end_index = TILE_WIDTH - px_within_row;

                        let mut end_index = 8;

                        let starting_tile_map_index = TILES_PER_ROW
                            * (total_bg_row / BG_TILE_HEIGHT)
                            + (total_bg_column / TILE_WIDTH);

                        let row_within_tile = total_bg_row % TILE_WIDTH;

                        let mut column: i32 = 0;

                        'tile_loop: for tile_num in 0..21 {
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
                                let bgp_index =
                                    ((((most_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1) << 1)
                                        + ((least_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1))
                                        as usize;
                                row_colors[column as usize] = color_indexes[bgp_index] as u8;
                                canvas.set_draw_color(COLOR_MAP[color_indexes[bgp_index]]);
                                canvas
                                    .draw_point(Point::new(column, row as i32))
                                    .expect("Failed drawing");
                                column += 1;
                                px_within_row = 0;
                            }
                            if tile_num == 19 {
                                if extra_tile {
                                    end_index = extra_end_index;
                                } else {
                                    break 'tile_loop;
                                }
                            }
                        }
                    } else {
                        canvas.set_draw_color(Color::RGB(0, 0, 0));
                        canvas.clear();
                    }

                    if self.get_sprite_enable_flag() == 1 {
                        let mut x_precendence = [200u8; 160];
                        for sprite in possible_sprites {
                            let row_within = row - sprite[OAM_Y_INDEX] as usize + 16;
                            let bg_over_obj = (sprite[OAM_ATTRIBUTE_INDEX] >> 7) == 1;
                            let tile_data_index = sprite[OAM_TILE_INDEX] as usize * BYTES_PER_TILE
                                + row_within * BYTES_PER_TILE_ROW;
                            let least_sig_byte = vram[tile_data_index];
                            let most_sig_byte = vram[(tile_data_index + 1)];
                            let palette = if (sprite[OAM_ATTRIBUTE_INDEX] >> 4) & 1 == 0 {
                                *self.obp0.lock().unwrap() as usize
                            } else {
                                *self.obp1.lock().unwrap() as usize
                            };
                            let color_indexes: [usize; 4] = [
                                (palette >> 0) & 0b11,
                                (palette >> 2) & 0b11,
                                (palette >> 4) & 0b11,
                                (palette >> 6) & 0b11,
                            ];
                            let x_end = sprite[OAM_X_INDEX];
                            let x_start = cmp::max(0, x_end - 8);
                            let mut index = TILE_WIDTH - (x_end - x_start) as usize;
                            for x in x_start..x_end {
                                let color_index =
                                    ((((most_sig_byte >> (TILE_WIDTH - index - 1)) & 1) << 1)
                                        + ((least_sig_byte >> (TILE_WIDTH - index - 1)) & 1))
                                        as usize;
                                let draw_color = color_indexes[color_index];
                                if (x_start < x_precendence[x as usize]) //no obj with priority 
                                    & (draw_color != 0)
                                // not transparent
                                {
                                    x_precendence[x as usize] = x_start;
                                    if !bg_over_obj || (row_colors[x as usize] == 0) {
                                        canvas.set_draw_color(COLOR_MAP[draw_color]);
                                        canvas
                                            .draw_point(Point::new(x as i32, row as i32))
                                            .expect("Failed drawing");
                                    }
                                }
                                index += 1;
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
                *self.stat.lock().unwrap() &= 0b1111100;
                let prev_p1 = *self.p1.lock().unwrap();
                let mut directional_keys = 0xF;
                let mut a_b_sel_start_keys = 0xF;
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            scancode: Some(Scancode::Escape),
                            ..
                        } => break 'running,
                        Event::KeyDown {
                            scancode: Some(Scancode::Z),
                            ..
                        } => a_b_sel_start_keys &= 0b0111,
                        Event::KeyDown {
                            scancode: Some(Scancode::X),
                            ..
                        } => a_b_sel_start_keys &= 0b1011,
                        Event::KeyDown {
                            scancode: Some(Scancode::A),
                            ..
                        } => a_b_sel_start_keys &= 0b1101,
                        Event::KeyDown {
                            scancode: Some(Scancode::S),
                            ..
                        } => a_b_sel_start_keys &= 0b1110,
                        Event::KeyDown {
                            scancode: Some(Scancode::Right),
                            ..
                        } => directional_keys &= 0b0111,
                        Event::KeyDown {
                            scancode: Some(Scancode::Left),
                            ..
                        } => directional_keys &= 0b1011,
                        Event::KeyDown {
                            scancode: Some(Scancode::Up),
                            ..
                        } => directional_keys &= 0b1101,
                        Event::KeyDown {
                            scancode: Some(Scancode::Down),
                            ..
                        } => directional_keys &= 0b1110,
                        _ => {}
                    }
                }
                //create context for mutex to drop
                {
                    let mut p1 = self.p1.lock().unwrap();
                    let p14 = (*p1 >> 4) & 1;
                    let p15 = (*p1 >> 5) & 1;
                    *p1 |= 0b110000;
                    if p14 == 1 {
                        *p1 |= directional_keys;
                    }
                    if p15 == 1 {
                        *p1 |= a_b_sel_start_keys;
                    }
                    if ((prev_p1 | *p1) - *p1) & 0xF != 0 {
                        *self.interrupt_flag.lock().unwrap() |= 1 << 4;
                    }
                }
                while (now.elapsed().as_nanos()) < (HBLANK_DOTS as f64 * NANOS_PER_DOT) as u128 {}
            }
            //VBLANK

            let mut now = Instant::now();

            // let cycles = (start.elapsed().as_nanos()) / NANOS_PER_DOT as u128;
            // println!("{}", cycles);
            *self.interrupt_flag.lock().unwrap() |= 0b00001;
            {
                let mut stat = self.stat.lock().unwrap();
                *stat &= 0b1111100;
                *stat |= 1;
            }

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

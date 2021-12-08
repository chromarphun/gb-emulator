use crate::ADVANCE_CYCLES;
use std::cmp;
use std::convert::TryInto;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

const TILES_PER_ROW: usize = 32;
const BG_MAP_SIZE_PX: usize = 256;
const TILE_WIDTH: usize = 8;
const BG_TILE_HEIGHT: usize = 8;
const BYTES_PER_TILE: usize = 16;
const BYTES_PER_TILE_ROW: usize = 2;
const SCREEN_PX_HEIGHT: usize = 144;
const VRAM_BLOCK_SIZE: usize = 128;

const OAM_SCAN_DOTS: u32 = 80;

const DRAWING_DOTS: u32 = 172;

const HBLANK_DOTS: u32 = 204;

const ROW_DOTS: u32 = 456;

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
    interrupt_flag: Arc<Mutex<u8>>,
    frame_send: mpsc::Sender<[[u8; 160]; 144]>,
    cycle_count: u32,
    possible_sprites: Vec<[u8; 4]>,
    starting: bool,
    current_sprite_search: u8,
    mode: u8,
    sprite_num: usize,
    window_row_activated: bool,
    draw_window: bool,
    color_indexes: [usize; 4],
    bg_tilemap_start: usize,
    win_tilemap_start: usize,
    px_within_row: usize,
    bg_tilemap_row_start: usize,
    win_tilemap_row_start: usize,
    bg_tiles_within_row_start: usize,
    bg_row_within_tile: usize,
    win_row_within_tile: usize,
    column: usize,
    tile_num: i8,
    current_window_row: usize,
    frame: [[u8; 160]; 144],
    x_precendence: [u8; 160],
    current_sprite_drawing: usize,
    bg_win_enable: bool,
    obj_enable: bool,
    frame_num: u32,
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
        interrupt_flag: Arc<Mutex<u8>>,
        frame_send: mpsc::Sender<[[u8; 160]; 144]>,
    ) -> PictureProcessingUnit {
        let cycle_count = 0;
        let possible_sprites: Vec<[u8; 4]> = Vec::new();
        let starting = true;
        let current_sprite_search = 0;
        let mode = 0;
        let sprite_num = 0;
        let window_row_activated = false;
        let draw_window = false;

        let color_indexes = [0usize; 4];
        let bg_tilemap_start: usize = 0x1800;
        let win_tilemap_start: usize = 0x1800;

        let px_within_row = 0;

        let bg_tilemap_row_start: usize = 0;
        let win_tilemap_row_start: usize = 0;

        let bg_tiles_within_row_start: usize = 0;

        let bg_row_within_tile: usize = 0;
        let win_row_within_tile: usize = 0;

        let column: usize = 0;
        let tile_num: i8 = 0;
        let current_window_row = 0;
        let frame = [[0; 160]; 144];
        let x_precendence = [200u8; 160];
        let current_sprite_drawing = 0;
        let bg_win_enable = true;
        let obj_enable = true;
        let frame_num = 0;
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
            frame_send,
            cycle_count,
            possible_sprites,
            starting,
            current_sprite_search,
            mode,
            sprite_num,
            window_row_activated,
            draw_window,
            color_indexes,
            bg_tilemap_start,
            win_tilemap_start,
            px_within_row,
            bg_tilemap_row_start,
            win_tilemap_row_start,
            bg_tiles_within_row_start,
            bg_row_within_tile,
            win_row_within_tile,
            column,
            tile_num,
            current_window_row,
            frame,
            x_precendence,
            current_sprite_drawing,
            bg_win_enable,
            obj_enable,
            frame_num,
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

    fn get_sprite_enable_flag(&self) -> u8 {
        (*self.lcdc.lock().unwrap() >> 1) & 1
    }
    fn get_bg_window_enable(&self) -> u8 {
        *self.lcdc.lock().unwrap() & 1
    }
    fn get_ppu_enable(&self) -> u8 {
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
    fn get_mode(&self) -> u8 {
        *self.stat.lock().unwrap() & 0b11
    }
    fn set_mode(&mut self, mode: u8) {
        *self.stat.lock().unwrap() &= 0b1111100;
        *self.stat.lock().unwrap() |= mode;
    }
    fn set_stat_interrupt(&mut self) {
        *self.interrupt_flag.lock().unwrap() |= 0b00010;
    }
    fn set_stat_lyc_lc_flag(&mut self) {
        *self.stat.lock().unwrap() |= 0b0000100;
    }
    fn reset_stat_lyc_lc_flag(&mut self) {
        *self.stat.lock().unwrap() &= 0b1111011;
    }
    fn set_vblank_interrupt(&mut self) {
        *self.interrupt_flag.lock().unwrap() |= 1;
    }

    pub fn advance(&mut self) {
        let mode = self.get_mode();

        match mode {
            0x0 => self.hblank(),
            0x1 => self.vblank(),
            0x2 => self.oam_search(),
            0x3 => self.drawing_tiles(),
            _ => {}
        }
    }

    fn oam_search(&mut self) {
        if self.starting {
            self.set_mode(2);
            if self.get_stat_oam_int_flag() == 1 {
                self.set_stat_interrupt()
            }
            self.possible_sprites = Vec::new();
            self.sprite_num = 0;
            self.current_sprite_search = 0;
            self.cycle_count += ADVANCE_CYCLES;
            self.starting = false;
        } else {
            let row = *self.ly.lock().unwrap() as usize;
            if self.current_sprite_search < 40 {
                let oam = self.oam.lock().unwrap();
                'sprite_loop: for i in self.current_sprite_search..(self.current_sprite_search + 5)
                {
                    if ((row + 16) as u8 >= oam[i as usize * BYTES_PER_OAM_ENTRY])
                        && (((row + 16) as u8)
                            < oam[i as usize * BYTES_PER_OAM_ENTRY] + self.get_obj_size())
                    {
                        self.possible_sprites.push(
                            oam[(i as usize * BYTES_PER_OAM_ENTRY)
                                ..((i as usize + 1) * BYTES_PER_OAM_ENTRY)]
                                .try_into()
                                .expect("Indexing error"),
                        );
                        self.sprite_num += 1;
                        if self.sprite_num == 10 {
                            self.current_sprite_search = 40;
                            break 'sprite_loop;
                        }
                    }
                }
                self.current_sprite_search += 5;
                self.cycle_count += ADVANCE_CYCLES;
            } else {
                self.cycle_count += ADVANCE_CYCLES;
                if self.cycle_count == OAM_SCAN_DOTS {
                    self.set_mode(3);
                    self.cycle_count = 0;
                    self.starting = true;
                }
            }
        }
    }

    fn drawing_tiles(&mut self) {
        let row = *self.ly.lock().unwrap() as usize;
        if self.starting {
            let wx = *self.wx.lock().unwrap() as usize;
            let wy = *self.wy.lock().unwrap() as usize;
            self.window_row_activated =
                wy <= row && self.get_win_enable_flag() == 1 && wx > 0 && wx < 144;
            self.draw_window = self.window_row_activated && wx < 8;
            let bgp = *self.bgp.lock().unwrap() as usize;
            self.color_indexes = [
                (bgp >> 0) & 0b11,
                (bgp >> 2) & 0b11,
                (bgp >> 4) & 0b11,
                (bgp >> 6) & 0b11,
            ];
            let scx = *self.scx.lock().unwrap() as usize;
            let scy = *self.scy.lock().unwrap() as usize;

            self.bg_tilemap_start = if self.get_bg_tile_map_flag() == 0 {
                0x9800 - 0x8000
            } else {
                0x9C00 - 0x8000
            };
            self.win_tilemap_start = if self.get_win_tile_map_flag() == 0 {
                0x9800 - 0x8000
            } else {
                0x9C00 - 0x8000
            };
            let total_bg_row = (scy + row) as usize % BG_MAP_SIZE_PX;

            self.px_within_row = if self.draw_window {
                7 - wx
            } else {
                scx % TILE_WIDTH
            };

            self.bg_tilemap_row_start = TILES_PER_ROW * (total_bg_row / BG_TILE_HEIGHT);
            self.win_tilemap_row_start = TILES_PER_ROW * (self.current_window_row / BG_TILE_HEIGHT);

            self.bg_tiles_within_row_start = scx / TILE_WIDTH;

            self.bg_row_within_tile = total_bg_row % TILE_WIDTH;
            self.win_row_within_tile = self.current_window_row % TILE_WIDTH;

            self.column = 0;
            self.tile_num = 0;
            self.starting = false;
            self.bg_win_enable = self.get_bg_window_enable() == 1;
            self.obj_enable = self.get_sprite_enable_flag() == 1;
            self.current_sprite_drawing = 0;
            self.x_precendence = [200u8; 160];
            self.cycle_count += ADVANCE_CYCLES;
        } else {
            if self.column < 160 && self.bg_win_enable {
                let vram = self.vram.lock().unwrap();
                let tile_data_flag = self.get_tile_data_flag();
                let (tilemap_start, tilemap_row_start, tiles_within_row_start, row_within_tile) =
                    if self.draw_window {
                        (
                            self.win_tilemap_start,
                            self.win_tilemap_row_start,
                            0,
                            self.win_row_within_tile,
                        )
                    } else {
                        (
                            self.bg_tilemap_start,
                            self.bg_tilemap_row_start,
                            self.bg_tiles_within_row_start,
                            self.bg_row_within_tile,
                        )
                    };
                let tile_map_index =
                    tilemap_row_start + (tiles_within_row_start + self.tile_num as usize) % 32;
                let absolute_tile_index = if tile_data_flag == 1 {
                    vram[tilemap_start + tile_map_index as usize] as usize
                } else {
                    let initial_index = vram[tilemap_start + tile_map_index as usize] as usize;
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

                'pixel_loop: for pixel in self.px_within_row..8 {
                    let bgp_index = ((((most_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1) << 1)
                        + ((least_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1))
                        as usize;
                    let pixel_color = self.color_indexes[bgp_index] as u8;
                    self.frame[row][self.column] = pixel_color;
                    self.column += 1;
                    if self.column == 160 {
                        break 'pixel_loop;
                    }
                    if (self.column + 7 == *self.wx.lock().unwrap() as usize)
                        && self.window_row_activated
                    {
                        self.tile_num = -1;
                        self.draw_window = true;
                        break 'pixel_loop;
                    }
                    self.px_within_row = 0;
                }
                self.tile_num += 1;
                self.cycle_count += ADVANCE_CYCLES;
            } else {
                if self.frame_num == 800 {
                    let q = 0;
                }
                if self.current_sprite_drawing < self.sprite_num && self.obj_enable {
                    let obj_length = self.get_obj_size();
                    let vram = self.vram.lock().unwrap();
                    let sprite = self.possible_sprites[self.current_sprite_drawing];
                    let x_flip = ((sprite[OAM_ATTRIBUTE_INDEX] >> 5) & 1) == 1;
                    let y_flip = ((sprite[OAM_ATTRIBUTE_INDEX] >> 6) & 1) == 1;
                    let row_within = if y_flip {
                        (obj_length - 1 + sprite[OAM_Y_INDEX]) as usize - row - 16
                    } else {
                        (row + 16) - sprite[OAM_Y_INDEX] as usize
                    };
                    let bg_over_obj = (sprite[OAM_ATTRIBUTE_INDEX] >> 7) == 1;
                    let tile_map_index = if obj_length == 16 {
                        if row_within >= 8 {
                            sprite[OAM_TILE_INDEX] | 0x01
                        } else {
                            sprite[OAM_TILE_INDEX] & 0xFE
                        }
                    } else {
                        sprite[OAM_TILE_INDEX]
                    };
                    let tile_data_index =
                        tile_map_index as usize * BYTES_PER_TILE + row_within * BYTES_PER_TILE_ROW;
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
                    if x_end > 0 && x_end < 168 {
                        let x_start = cmp::max(0, x_end - 8);
                        let mut index = TILE_WIDTH - (x_end - x_start) as usize;
                        let obj_range = if x_flip {
                            (x_start..x_end).rev().collect::<Vec<u8>>()
                        } else {
                            (x_start..x_end).collect::<Vec<u8>>()
                        };
                        for x in obj_range {
                            let color_index = ((((most_sig_byte >> (TILE_WIDTH - index - 1)) & 1)
                                << 1)
                                + ((least_sig_byte >> (TILE_WIDTH - index - 1)) & 1))
                                as usize;
                            let draw_color = color_indexes[color_index] as u8;
                            if (x_start < self.x_precendence[x as usize]) //no obj with priority 
                                & (draw_color != 0)
                            // not transparent
                            {
                                self.x_precendence[x as usize] = x_start;
                                if !bg_over_obj || (self.frame[row][x as usize] == 0) {
                                    self.frame[row][x as usize] = draw_color;
                                }
                            }
                            index += 1;
                        }
                    }

                    self.current_sprite_drawing += 1;
                    self.cycle_count += ADVANCE_CYCLES;
                } else {
                    self.cycle_count += ADVANCE_CYCLES;
                    if self.cycle_count == DRAWING_DOTS {
                        self.cycle_count = 0;
                        self.starting = true;
                        self.set_mode(0);
                    }
                }
            }
        }
    }

    fn hblank(&mut self) {
        if self.starting {
            *self.ly.lock().unwrap() += 1;
            if *self.lyc.lock().unwrap() == *self.ly.lock().unwrap() {
                self.set_stat_lyc_lc_flag();
                if self.get_stat_lyc_lc_int_flag() == 1 {
                    self.set_stat_interrupt();
                }
            } else {
                self.reset_stat_lyc_lc_flag()
            }
            if self.get_stat_hblank_int_flag() == 1 {
                self.set_stat_interrupt();
            }
            if self.window_row_activated {
                self.current_window_row += 1;
            }
            self.cycle_count += ADVANCE_CYCLES;
            self.starting = false;
        } else {
            self.cycle_count += ADVANCE_CYCLES;
            if self.cycle_count == HBLANK_DOTS {
                self.cycle_count = 0;
                self.starting = true;
                let new_mode = if *self.ly.lock().unwrap() == 144 {
                    1
                } else {
                    2
                };
                self.set_mode(new_mode);
            }
        }
    }

    fn vblank(&mut self) {
        if self.get_ppu_enable() == 0 {
            self.frame = [[0; 160]; 144];
        }
        if self.starting {
            //println!("frame_num: {}", self.frame_num);
            self.frame_num += 1;
            self.frame_send.send(self.frame).unwrap();
            self.starting = false;
            self.set_vblank_interrupt();
            if self.get_stat_vblank_int_flag() == 1 {
                self.set_stat_interrupt();
            }
            self.current_window_row = 0;
        }
        if self.frame_num == 746 && *self.ly.lock().unwrap() == 145 {
            let q = 0;
        }
        self.cycle_count += ADVANCE_CYCLES;
        if self.cycle_count == ROW_DOTS {
            self.cycle_count = 0;
            *self.ly.lock().unwrap() += 1;
            if *self.lyc.lock().unwrap() == *self.ly.lock().unwrap() {
                self.set_stat_lyc_lc_flag();
                if self.get_stat_lyc_lc_int_flag() == 1 {
                    self.set_stat_interrupt();
                }
            } else {
                self.reset_stat_lyc_lc_flag();
            }
            if *self.ly.lock().unwrap() == 154 {
                *self.ly.lock().unwrap() = 0;
                self.starting = true;
                self.set_mode(2);
            }
        }
    }
}

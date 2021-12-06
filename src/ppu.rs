use crate::cycle_count_mod;
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

const OAM_SCAN_DOTS: i32 = 80;

const DRAWING_DOTS: i32 = 172;

const HBLANK_DOTS: i32 = 204;

const ROW_DOTS: i32 = 456;

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
    cycle_count: Arc<Mutex<i32>>,
    cycle_cond: Arc<Condvar>,
    lcdc_cond: Arc<Condvar>,
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
        cycle_count: Arc<Mutex<i32>>,
        cycle_cond: Arc<Condvar>,
        lcdc_cond: Arc<Condvar>,
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
            frame_send,
            cycle_count,
            cycle_cond,
            lcdc_cond,
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
        let mut start_cycle_count = 0;

        *self.ly.lock().unwrap() = 0;
        loop {
            let mut frame: [[u8; 160]; 144] = [[0; 160]; 144];
            let mut current_window_row = 0;
            let mut lcdc = self.lcdc.lock().unwrap();
            while (*lcdc >> 7) & 1 == 0 {
                lcdc = self.lcdc_cond.wait(lcdc).unwrap();
                self.frame_send.send(frame).unwrap();
                *self.ly.lock().unwrap() = 0;
                *self.stat.lock().unwrap() &= 0b11111100;
            }
            std::mem::drop(lcdc);

            for row in 0..SCREEN_PX_HEIGHT {
                //create context for vram/oam lock to exist
                {
                    //OAM SCAN PERIOD
                    start_cycle_count = *self.cycle_count.lock().unwrap();
                    {
                        let mut stat = self.stat.lock().unwrap();
                        *stat &= 0b11111100;
                        *stat |= 0b00000010;
                    }
                    if self.get_stat_oam_int_flag() == 1 {
                        *self.interrupt_flag.lock().unwrap() |= 0b00010;
                    }
                    let mut possible_sprites: Vec<[u8; 4]> = Vec::new();
                    let oam = *self.oam.lock().unwrap();
                    let _oam_lock = self.oam.lock().unwrap();
                    let obj_length = self.get_obj_size();
                    let mut sprite_num = 0;
                    'sprite_loop: for i in 0..SPIRTES_IN_OAM {
                        if ((row + 16) as u8 >= oam[i * BYTES_PER_OAM_ENTRY])
                            && (((row + 16) as u8) < oam[i * BYTES_PER_OAM_ENTRY] + obj_length)
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

                    let mut current_cycle_count = self.cycle_count.lock().unwrap();

                    while cycle_count_mod(*current_cycle_count - start_cycle_count) <= OAM_SCAN_DOTS
                    {
                        current_cycle_count = self.cycle_cond.wait(current_cycle_count).unwrap();
                    }
                    std::mem::drop(current_cycle_count);
                    // DRAWING PERIOD
                    start_cycle_count = *self.cycle_count.lock().unwrap();
                    {
                        let mut stat = self.stat.lock().unwrap();
                        *stat &= 0b11111100;
                        *stat |= 0b00000011;
                    }

                    let mut row_colors = [0u8; 160];
                    let vram = *self.vram.lock().unwrap();
                    let _vram_lock = self.vram.lock().unwrap();
                    if self.get_bg_window_enable() == 1 {
                        let wx = *self.wx.lock().unwrap() as usize;
                        let wy = *self.wy.lock().unwrap() as usize;
                        let window_row_activated =
                            wy <= row && self.get_win_enable_flag() == 1 && wx > 0 && wx < 144;
                        let mut draw_window = window_row_activated && wx < 8;
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
                            0x9800 - 0x8000
                        } else {
                            0x9C00 - 0x8000
                        };
                        let win_tilemap_start: usize = if self.get_win_tile_map_flag() == 0 {
                            0x9800 - 0x8000
                        } else {
                            0x9C00 - 0x8000
                        };
                        let total_bg_row: usize = (scy + row) as usize % BG_MAP_SIZE_PX;

                        let mut px_within_row = if draw_window {
                            7 - wx
                        } else {
                            scx % TILE_WIDTH
                        };

                        let bg_tilemap_row_start = TILES_PER_ROW * (total_bg_row / BG_TILE_HEIGHT);
                        let win_tilemap_row_start =
                            TILES_PER_ROW * (current_window_row / BG_TILE_HEIGHT);

                        let bg_tiles_within_row_start = scx / TILE_WIDTH;

                        let bg_row_within_tile = total_bg_row % TILE_WIDTH;
                        let win_row_within_tile = current_window_row % TILE_WIDTH;

                        let mut column: usize = 0;
                        let mut tile_num: i8 = 0;
                        'tile_loop: loop {
                            let (
                                tilemap_start,
                                tilemap_row_start,
                                tiles_within_row_start,
                                row_within_tile,
                            ) = if draw_window {
                                (
                                    win_tilemap_start,
                                    win_tilemap_row_start,
                                    0,
                                    win_row_within_tile,
                                )
                            } else {
                                (
                                    bg_tilemap_start,
                                    bg_tilemap_row_start,
                                    bg_tiles_within_row_start,
                                    bg_row_within_tile,
                                )
                            };
                            let tile_map_index = tilemap_row_start
                                + (tiles_within_row_start + tile_num as usize) % 32;
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

                            'pixel_loop: for pixel in px_within_row..8 {
                                let bgp_index =
                                    ((((most_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1) << 1)
                                        + ((least_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1))
                                        as usize;
                                let pixel_color = color_indexes[bgp_index] as u8;
                                frame[row][column] = pixel_color;
                                row_colors[column as usize] = pixel_color as u8;
                                column += 1;
                                if column == 160 {
                                    break 'tile_loop;
                                }
                                if (column + 7 == wx) && window_row_activated {
                                    tile_num = -1;
                                    draw_window = true;
                                    break 'pixel_loop;
                                }
                                px_within_row = 0;
                            }
                            tile_num += 1;
                        }
                        if window_row_activated {
                            current_window_row += 1;
                        }
                    }

                    if self.get_sprite_enable_flag() == 1 {
                        let mut x_precendence = [200u8; 160];
                        for sprite in possible_sprites {
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
                            let tile_data_index = tile_map_index as usize * BYTES_PER_TILE
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
                            let obj_range = if x_flip {
                                (x_start..x_end).rev().collect::<Vec<u8>>()
                            } else {
                                (x_start..x_end).collect::<Vec<u8>>()
                            };
                            for x in obj_range {
                                let color_index =
                                    ((((most_sig_byte >> (TILE_WIDTH - index - 1)) & 1) << 1)
                                        + ((least_sig_byte >> (TILE_WIDTH - index - 1)) & 1))
                                        as usize;
                                let draw_color = color_indexes[color_index] as u8;
                                if (x_start < x_precendence[x as usize]) //no obj with priority 
                                    & (draw_color != 0)
                                // not transparent
                                {
                                    x_precendence[x as usize] = x_start;
                                    if !bg_over_obj || (row_colors[x as usize] == 0) {
                                        frame[row][x as usize] = draw_color;
                                    }
                                }
                                index += 1;
                            }
                        }
                    }
                    //spin while we're waiting for drawing pixel period to end
                    //vram is still locked!
                    *self.ly.lock().unwrap() += 1;
                    if *self.lyc.lock().unwrap() == *self.ly.lock().unwrap() {
                        *self.stat.lock().unwrap() |= 0b0000100;
                        if self.get_stat_lyc_lc_int_flag() == 1 {
                            *self.interrupt_flag.lock().unwrap() |= 0b00010;
                        }
                    } else {
                        *self.stat.lock().unwrap() &= 0b1111011;
                    }

                    let mut current_cycle_count = self.cycle_count.lock().unwrap();
                    while cycle_count_mod(*current_cycle_count - start_cycle_count) <= DRAWING_DOTS
                    {
                        current_cycle_count = self.cycle_cond.wait(current_cycle_count).unwrap();
                    }
                    std::mem::drop(current_cycle_count);
                }
                //HBLANK
                //we've left vram context and now vram is accessible during HBLANK
                start_cycle_count = *self.cycle_count.lock().unwrap();
                if self.get_stat_hblank_int_flag() == 1 {
                    *self.interrupt_flag.lock().unwrap() |= 0b00010;
                }
                *self.stat.lock().unwrap() &= 0b1111100;

                let mut current_cycle_count = self.cycle_count.lock().unwrap();
                while cycle_count_mod(*current_cycle_count - start_cycle_count) <= HBLANK_DOTS {
                    current_cycle_count = self.cycle_cond.wait(current_cycle_count).unwrap();
                }
                std::mem::drop(current_cycle_count);
            }
            //VBLANK
            start_cycle_count = *self.cycle_count.lock().unwrap();
            self.frame_send.send(frame).unwrap();
            *self.interrupt_flag.lock().unwrap() |= 0b00001;
            {
                let mut stat = self.stat.lock().unwrap();
                *stat &= 0b1111100;
                *stat |= 0b0000001;
            }

            if self.get_stat_vblank_int_flag() == 1 {
                *self.interrupt_flag.lock().unwrap() |= 0b00010;
            }

            for i in 1..11 {
                let mut current_cycle_count = self.cycle_count.lock().unwrap();
                while cycle_count_mod(*current_cycle_count - start_cycle_count) <= i * ROW_DOTS {
                    current_cycle_count = self.cycle_cond.wait(current_cycle_count).unwrap();
                }
                std::mem::drop(current_cycle_count);
                let new_ly = (*self.ly.lock().unwrap() + 1) % 154;
                *self.ly.lock().unwrap() = new_ly;
                if *self.lyc.lock().unwrap() == new_ly {
                    *self.stat.lock().unwrap() |= 0b0000100;
                    if self.get_stat_lyc_lc_int_flag() == 1 {
                        *self.interrupt_flag.lock().unwrap() |= 0b00010;
                    }
                } else {
                    *self.stat.lock().unwrap() &= 0b1111011;
                }
            }
        }
    }
}

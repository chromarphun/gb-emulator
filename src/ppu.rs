use crate::ADVANCE_CYCLES;
use std::cmp;
use std::convert::TryInto;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

use crate::emulator::GameBoyEmulator;

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

const LCDC_ADDR: usize = 0xFF40;
const STAT_ADDR: usize = 0xFF41;
const SCY_ADDR: usize = 0xFF42;
const SCX_ADDR: usize = 0xFF43;
const LY_ADDR: usize = 0xFF44;
const LYC_ADDR: usize = 0xFF45;
const BGP_ADDR: usize = 0xFF47;
const OBP0_ADDR: usize = 0xFF48;
const OBP1_ADDR: usize = 0xFF49;
const WY_ADDR: usize = 0xFF4A;
const WX_ADDR: usize = 0xFF4B;
const INT_FLAG_ADDR: usize = 0xFF0F;
const OAM_START_ADDR: usize = 0xFE00;
const VRAM_START_ADDR: usize = 0x8000;

pub struct PictureProcessingUnit {
    cycle_count: u32,
    possible_sprites: Vec<[u8; 4]>,
    starting: bool,
    current_sprite_search: u8,
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
    x_precendence: [u8; 160],
    current_sprite_drawing: usize,
    bg_win_enable: bool,
    obj_enable: bool,
    frame_num: u32,
}

impl PictureProcessingUnit {
    pub fn new() -> PictureProcessingUnit {
        PictureProcessingUnit {
            cycle_count: 0,
            possible_sprites: Vec::new(),
            starting: true,
            current_sprite_search: 0,
            sprite_num: 0,
            window_row_activated: false,
            draw_window: false,
            color_indexes: [0; 4],
            bg_tilemap_start: 0,
            win_tilemap_start: 0,
            px_within_row: 0,
            bg_tilemap_row_start: 0,
            win_tilemap_row_start: 0,
            bg_tiles_within_row_start: 0,
            bg_row_within_tile: 0,
            win_row_within_tile: 0,
            column: 0,
            tile_num: 0,
            current_window_row: 0,
            x_precendence: [200; 160],
            current_sprite_drawing: 0,
            bg_win_enable: false,
            obj_enable: false,
            frame_num: 0,
        }
    }
}

impl GameBoyEmulator {
    fn get_bg_tile_map_flag(&self) -> u8 {
        (self.mem_unit.get_memory(LCDC_ADDR) >> 3) & 1
    }
    fn get_win_tile_map_flag(&self) -> u8 {
        (self.mem_unit.get_memory(LCDC_ADDR) >> 6) & 1
    }
    fn get_tile_data_flag(&self) -> u8 {
        (self.mem_unit.get_memory(LCDC_ADDR) >> 4) & 1
    }
    fn get_win_enable_flag(&self) -> u8 {
        (self.mem_unit.get_memory(LCDC_ADDR) >> 5) & 1
    }
    fn get_obj_size(&self) -> u8 {
        if ((self.mem_unit.get_memory(LCDC_ADDR) >> 2) & 1) == 1 {
            16
        } else {
            8
        }
    }

    fn get_sprite_enable_flag(&self) -> u8 {
        (self.mem_unit.get_memory(LCDC_ADDR) >> 1) & 1
    }
    fn get_bg_window_enable(&self) -> u8 {
        self.mem_unit.get_memory(LCDC_ADDR) & 1
    }
    fn get_ppu_enable(&self) -> u8 {
        (self.mem_unit.get_memory(LCDC_ADDR) >> 7) & 1
    }
    fn get_stat_lyc_lc_int_flag(&self) -> u8 {
        (self.mem_unit.get_memory(STAT_ADDR) >> 6) & 1
    }
    fn get_stat_oam_int_flag(&self) -> u8 {
        (self.mem_unit.get_memory(STAT_ADDR) >> 5) & 1
    }
    fn get_stat_vblank_int_flag(&self) -> u8 {
        (self.mem_unit.get_memory(STAT_ADDR) >> 4) & 1
    }
    fn get_stat_hblank_int_flag(&self) -> u8 {
        (self.mem_unit.get_memory(STAT_ADDR) >> 3) & 1
    }
    pub fn get_mode(&self) -> u8 {
        self.mem_unit.get_memory(STAT_ADDR) & 0b11
    }
    fn set_mode(&mut self, mode: u8) {
        self.mem_unit
            .write_memory(STAT_ADDR, self.mem_unit.get_memory(STAT_ADDR) & 0b1111100);
        self.mem_unit
            .write_memory(STAT_ADDR, self.mem_unit.get_memory(STAT_ADDR) | mode);
    }
    fn set_stat_interrupt(&mut self) {
        self.mem_unit.write_memory(
            INT_FLAG_ADDR,
            self.mem_unit.get_memory(INT_FLAG_ADDR) | 0b00010,
        );
    }
    fn set_stat_lyc_lc_flag(&mut self) {
        self.mem_unit
            .write_memory(STAT_ADDR, self.mem_unit.get_memory(STAT_ADDR) | 0b0000100);
    }
    fn reset_stat_lyc_lc_flag(&mut self) {
        self.mem_unit
            .write_memory(STAT_ADDR, self.mem_unit.get_memory(STAT_ADDR) & 0b1111011);
    }
    fn set_vblank_interrupt(&mut self) {
        self.mem_unit
            .write_memory(INT_FLAG_ADDR, self.mem_unit.get_memory(INT_FLAG_ADDR) | 1);
    }
    pub fn ppu_advance(&mut self) {
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
        if self.ppu.starting {
            self.set_mode(2);
            if self.get_stat_oam_int_flag() == 1 {
                self.set_stat_interrupt()
            }
            self.ppu.possible_sprites = Vec::new();
            self.ppu.sprite_num = 0;
            self.ppu.current_sprite_search = 0;
            self.ppu.cycle_count += ADVANCE_CYCLES;
            self.ppu.starting = false;
        } else {
            let row = self.mem_unit.get_memory(LY_ADDR) as usize;
            if self.ppu.current_sprite_search < 40 {
                'sprite_loop: for i in
                    self.ppu.current_sprite_search..(self.ppu.current_sprite_search + 5)
                {
                    let y_pos = self
                        .mem_unit
                        .get_memory(OAM_START_ADDR + i as usize * BYTES_PER_OAM_ENTRY);
                    if ((row + 16) as u8 >= y_pos)
                        && (((row + 16) as u8) < y_pos + self.get_obj_size())
                    {
                        let mut poss_sprite = [0u8; 4];
                        for j in 0..BYTES_PER_OAM_ENTRY {
                            poss_sprite[j] = self
                                .mem_unit
                                .get_memory(OAM_START_ADDR + i as usize * BYTES_PER_OAM_ENTRY + j);
                        }
                        self.ppu.possible_sprites.push(poss_sprite);
                        self.ppu.sprite_num += 1;
                        if self.ppu.sprite_num == 10 {
                            self.ppu.current_sprite_search = 40;
                            break 'sprite_loop;
                        }
                    }
                }
                self.ppu.current_sprite_search += 5;
                self.ppu.cycle_count += ADVANCE_CYCLES;
            } else {
                self.ppu.cycle_count += ADVANCE_CYCLES;
                if self.ppu.cycle_count == OAM_SCAN_DOTS {
                    self.set_mode(3);
                    self.ppu.cycle_count = 0;
                    self.ppu.starting = true;
                }
            }
        }
    }

    fn drawing_tiles(&mut self) {
        let row = self.mem_unit.get_memory(LY_ADDR) as usize;
        if self.ppu.starting {
            let wx = self.mem_unit.get_memory(WX_ADDR) as usize;
            let wy = self.mem_unit.get_memory(WY_ADDR) as usize;
            self.ppu.window_row_activated =
                wy <= row && self.get_win_enable_flag() == 1 && wx > 0 && wx < 144;
            self.ppu.draw_window = self.ppu.window_row_activated && wx < 8;
            let bgp = self.mem_unit.get_memory(BGP_ADDR) as usize;
            self.ppu.color_indexes = [
                (bgp >> 0) & 0b11,
                (bgp >> 2) & 0b11,
                (bgp >> 4) & 0b11,
                (bgp >> 6) & 0b11,
            ];
            let scx = self.mem_unit.get_memory(SCX_ADDR) as usize;
            let scy = self.mem_unit.get_memory(SCY_ADDR) as usize;

            self.ppu.bg_tilemap_start = if self.get_bg_tile_map_flag() == 0 {
                0x9800
            } else {
                0x9C00
            };
            self.ppu.win_tilemap_start = if self.get_win_tile_map_flag() == 0 {
                0x9800
            } else {
                0x9C00
            };
            let total_bg_row = (scy + row) as usize % BG_MAP_SIZE_PX;

            self.ppu.px_within_row = if self.ppu.draw_window {
                7 - wx
            } else {
                scx % TILE_WIDTH
            };

            self.ppu.bg_tilemap_row_start = TILES_PER_ROW * (total_bg_row / BG_TILE_HEIGHT);
            self.ppu.win_tilemap_row_start =
                TILES_PER_ROW * (self.ppu.current_window_row / BG_TILE_HEIGHT);

            self.ppu.bg_tiles_within_row_start = scx / TILE_WIDTH;

            self.ppu.bg_row_within_tile = total_bg_row % TILE_WIDTH;
            self.ppu.win_row_within_tile = self.ppu.current_window_row % TILE_WIDTH;

            self.ppu.column = 0;
            self.ppu.tile_num = 0;
            self.ppu.starting = false;
            self.ppu.bg_win_enable = self.get_bg_window_enable() == 1;
            self.ppu.obj_enable = self.get_sprite_enable_flag() == 1;
            self.ppu.current_sprite_drawing = 0;
            self.ppu.x_precendence = [200u8; 160];
            self.ppu.cycle_count += ADVANCE_CYCLES;
        } else {
            if self.ppu.column < 160 && self.ppu.bg_win_enable {
                let tile_data_flag = self.get_tile_data_flag();
                let (tilemap_start, tilemap_row_start, tiles_within_row_start, row_within_tile) =
                    if self.ppu.draw_window {
                        (
                            self.ppu.win_tilemap_start,
                            self.ppu.win_tilemap_row_start,
                            0,
                            self.ppu.win_row_within_tile,
                        )
                    } else {
                        (
                            self.ppu.bg_tilemap_start,
                            self.ppu.bg_tilemap_row_start,
                            self.ppu.bg_tiles_within_row_start,
                            self.ppu.bg_row_within_tile,
                        )
                    };
                let tile_map_index =
                    tilemap_row_start + (tiles_within_row_start + self.ppu.tile_num as usize) % 32;
                let absolute_tile_index = if tile_data_flag == 1 {
                    self.mem_unit
                        .get_memory(tilemap_start + tile_map_index as usize)
                        as usize
                } else {
                    let initial_index = self
                        .mem_unit
                        .get_memory(tilemap_start + tile_map_index as usize)
                        as usize;
                    if initial_index < VRAM_BLOCK_SIZE {
                        initial_index + 2 * VRAM_BLOCK_SIZE
                    } else {
                        initial_index
                    }
                };
                let tile_data_index = absolute_tile_index * BYTES_PER_TILE // getting to the starting byte
                + row_within_tile * BYTES_PER_TILE_ROW; //getting to the row
                let least_sig_byte = self
                    .mem_unit
                    .get_memory(VRAM_START_ADDR + tile_data_index as usize);
                let most_sig_byte = self
                    .mem_unit
                    .get_memory(VRAM_START_ADDR + (tile_data_index + 1) as usize);

                'pixel_loop: for pixel in self.ppu.px_within_row..8 {
                    let bgp_index = ((((most_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1) << 1)
                        + ((least_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1))
                        as usize;
                    let pixel_color = self.ppu.color_indexes[bgp_index] as u8;
                    self.frame[row][self.ppu.column] = pixel_color;
                    self.ppu.column += 1;
                    if self.ppu.column == 160 {
                        break 'pixel_loop;
                    }
                    if (self.ppu.column + 7 == self.mem_unit.get_memory(WX_ADDR) as usize)
                        && self.ppu.window_row_activated
                    {
                        self.ppu.tile_num = -1;
                        self.ppu.draw_window = true;
                        break 'pixel_loop;
                    }
                    self.ppu.px_within_row = 0;
                }
                self.ppu.tile_num += 1;
                self.ppu.cycle_count += ADVANCE_CYCLES;
            } else {
                if self.ppu.current_sprite_drawing < self.ppu.sprite_num && self.ppu.obj_enable {
                    let obj_length = self.get_obj_size();
                    let sprite = self.ppu.possible_sprites[self.ppu.current_sprite_drawing];
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
                    let least_sig_byte =
                        self.mem_unit.get_memory(VRAM_START_ADDR + tile_data_index);
                    let most_sig_byte = self
                        .mem_unit
                        .get_memory(VRAM_START_ADDR + (tile_data_index + 1));

                    let palette = if (sprite[OAM_ATTRIBUTE_INDEX] >> 4) & 1 == 0 {
                        self.mem_unit.get_memory(OBP0_ADDR) as usize
                    } else {
                        self.mem_unit.get_memory(OBP1_ADDR) as usize
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
                        for x in obj_range.into_iter().filter(|z| *z < 160) {
                            let color_index = ((((most_sig_byte >> (TILE_WIDTH - index - 1)) & 1)
                                << 1)
                                + ((least_sig_byte >> (TILE_WIDTH - index - 1)) & 1))
                                as usize;
                            let draw_color = color_indexes[color_index] as u8;
                            if (x_start < self.ppu.x_precendence[x as usize]) //no obj with priority 
                                & (draw_color != 0)
                            // not transparent
                            {
                                self.ppu.x_precendence[x as usize] = x_start;
                                if !bg_over_obj || (self.frame[row][x as usize] == 0) {
                                    self.frame[row][x as usize] = draw_color;
                                }
                            }
                            index += 1;
                        }
                    }

                    self.ppu.current_sprite_drawing += 1;
                    self.ppu.cycle_count += ADVANCE_CYCLES;
                } else {
                    self.ppu.cycle_count += ADVANCE_CYCLES;
                    if self.ppu.cycle_count == DRAWING_DOTS {
                        self.ppu.cycle_count = 0;
                        self.ppu.starting = true;
                        self.set_mode(0);
                    }
                }
            }
        }
    }

    fn hblank(&mut self) {
        if self.ppu.starting {
            self.mem_unit
                .write_memory(LY_ADDR, self.mem_unit.get_memory(LY_ADDR) + 1);
            if self.mem_unit.get_memory(LYC_ADDR) == self.mem_unit.get_memory(LY_ADDR) {
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
            if self.ppu.window_row_activated {
                self.ppu.current_window_row += 1;
            }
            self.ppu.cycle_count += ADVANCE_CYCLES;
            self.ppu.starting = false;
        } else {
            self.ppu.cycle_count += ADVANCE_CYCLES;
            if self.ppu.cycle_count == HBLANK_DOTS {
                self.ppu.cycle_count = 0;
                self.ppu.starting = true;
                let new_mode = if self.mem_unit.get_memory(LY_ADDR) == 144 {
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
        if self.ppu.starting {
            self.ppu.frame_num += 1;
            self.ppu.starting = false;
            self.set_vblank_interrupt();
            if self.get_stat_vblank_int_flag() == 1 {
                self.set_stat_interrupt();
            }
            self.ppu.current_window_row = 0;
        }
        self.ppu.cycle_count += ADVANCE_CYCLES;
        if self.ppu.cycle_count == ROW_DOTS {
            self.ppu.cycle_count = 0;
            self.mem_unit
                .write_memory(LY_ADDR, self.mem_unit.get_memory(LY_ADDR) + 1);
            if self.mem_unit.get_memory(LYC_ADDR) == self.mem_unit.get_memory(LY_ADDR) {
                self.set_stat_lyc_lc_flag();
                if self.get_stat_lyc_lc_int_flag() == 1 {
                    self.set_stat_interrupt();
                }
            } else {
                self.reset_stat_lyc_lc_flag();
            }
            if self.mem_unit.get_memory(LY_ADDR) == 154 {
                self.mem_unit.write_memory(LY_ADDR, 0);
                self.ppu.starting = true;
                self.set_mode(2);
            }
        }
    }
}

impl PictureProcessingUnit {}

use crate::constants::*;
use crate::emulator::RequestSource;
use pixels::Pixels;
use serde::{Deserialize, Serialize};
use std::cmp;

use crate::emulator::GameBoyEmulator;

const SOURCE: RequestSource = RequestSource::PPU;

#[inline]
fn convert_to_index(row: usize, column: usize) -> usize {
    (160 * row + column) * 4
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PictureProcessingUnit {
    pub cycle_count: u32,
    possible_sprites: Vec<[u8; 4]>,
    starting: bool,
    current_sprite_search: u8,
    sprite_num: usize,
    window_row_activated: bool,
    draw_window: bool,
    color_indexes: [usize; 4],
    bg_tilemap_start_addr: usize,
    win_tilemap_start_addr: usize,
    px_within_row: usize,
    bg_tilemap_row_start_index: usize,
    win_tilemap_row_start_index: usize,
    bg_tiles_within_row_start: usize,
    bg_row_within_tile: usize,
    win_row_within_tile: usize,
    column: usize,
    tile_num: i8,
    current_window_row: usize,
    x_precendence: Vec<u8>,
    current_sprite_drawing: usize,
    bg_win_enable: bool,
    obj_enable: bool,
    frame_num: u32,
    frame_data: Vec<u8>,
    frame_index: usize,
    row_colors: Vec<usize>,
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
            bg_tilemap_start_addr: 0,
            win_tilemap_start_addr: 0,
            px_within_row: 0,
            bg_tilemap_row_start_index: 0,
            win_tilemap_row_start_index: 0,
            bg_tiles_within_row_start: 0,
            bg_row_within_tile: 0,
            win_row_within_tile: 0,
            column: 0,
            tile_num: 0,
            current_window_row: 0,
            x_precendence: vec![200; 160],
            current_sprite_drawing: 0,
            bg_win_enable: false,
            obj_enable: false,
            frame_num: 0,
            frame_data: vec![0; 92160],
            frame_index: 0,
            row_colors: vec![0; 160],
        }
    }
}

impl GameBoyEmulator {
    fn get_bg_tile_map_flag(&self) -> u8 {
        (self.mem_unit.get_memory(LCDC_ADDR, SOURCE) >> 3) & 1
    }
    fn get_win_tile_map_flag(&self) -> u8 {
        (self.mem_unit.get_memory(LCDC_ADDR, SOURCE) >> 6) & 1
    }
    fn get_tile_data_flag(&self) -> u8 {
        (self.mem_unit.get_memory(LCDC_ADDR, SOURCE) >> 4) & 1
    }
    fn get_win_enable_flag(&self) -> u8 {
        (self.mem_unit.get_memory(LCDC_ADDR, SOURCE) >> 5) & 1
    }
    fn get_obj_size(&self) -> u8 {
        if ((self.mem_unit.get_memory(LCDC_ADDR, SOURCE) >> 2) & 1) == 1 {
            16
        } else {
            8
        }
    }

    fn get_sprite_enable_flag(&self) -> u8 {
        (self.mem_unit.get_memory(LCDC_ADDR, SOURCE) >> 1) & 1
    }
    fn get_bg_window_enable(&self) -> u8 {
        self.mem_unit.get_memory(LCDC_ADDR, SOURCE) & 1
    }
    fn get_ppu_enable(&self) -> u8 {
        (self.mem_unit.get_memory(LCDC_ADDR, SOURCE) >> 7) & 1
    }
    fn get_stat_lyc_lc_int_flag(&self) -> u8 {
        (self.mem_unit.get_memory(STAT_ADDR, SOURCE) >> 6) & 1
    }
    fn get_stat_oam_int_flag(&self) -> u8 {
        (self.mem_unit.get_memory(STAT_ADDR, SOURCE) >> 5) & 1
    }
    fn get_stat_vblank_int_flag(&self) -> u8 {
        (self.mem_unit.get_memory(STAT_ADDR, SOURCE) >> 4) & 1
    }
    fn get_stat_hblank_int_flag(&self) -> u8 {
        (self.mem_unit.get_memory(STAT_ADDR, SOURCE) >> 3) & 1
    }
    pub fn get_mode(&self) -> u8 {
        self.mem_unit.get_memory(STAT_ADDR, SOURCE) & 0b11
    }
    fn set_mode(&mut self, mode: u8) {
        self.mem_unit.write_memory(
            STAT_ADDR,
            self.mem_unit.get_memory(STAT_ADDR, SOURCE) & 0b1111100,
            SOURCE,
        );
        self.mem_unit.write_memory(
            STAT_ADDR,
            self.mem_unit.get_memory(STAT_ADDR, SOURCE) | mode,
            SOURCE,
        );
    }
    fn set_stat_interrupt(&mut self) {
        self.mem_unit.write_memory(
            INT_FLAG_ADDR,
            self.mem_unit.get_memory(INT_FLAG_ADDR, SOURCE) | 0b00010,
            SOURCE,
        );
    }
    fn set_stat_lyc_lc_flag(&mut self) {
        self.mem_unit.write_memory(
            STAT_ADDR,
            self.mem_unit.get_memory(STAT_ADDR, SOURCE) | 0b0000100,
            SOURCE,
        );
    }
    fn reset_stat_lyc_lc_flag(&mut self) {
        self.mem_unit.write_memory(
            STAT_ADDR,
            self.mem_unit.get_memory(STAT_ADDR, SOURCE) & 0b1111011,
            SOURCE,
        );
    }
    fn set_vblank_interrupt(&mut self) {
        self.mem_unit.write_memory(
            INT_FLAG_ADDR,
            self.mem_unit.get_memory(INT_FLAG_ADDR, SOURCE) | 1,
            SOURCE,
        );
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
        if self.get_ppu_enable() == 0 {
            self.mem_unit.ppu_mode = 1;
        }
    }

    fn oam_search(&mut self) {
        if self.ppu.starting {
            self.set_mode(2);
            self.mem_unit.ppu_mode = 2;
            if self.get_stat_oam_int_flag() == 1 {
                self.set_stat_interrupt()
            }
            self.ppu.possible_sprites = Vec::new();
            self.ppu.sprite_num = 0;
            self.ppu.current_sprite_search = 0;
            self.ppu.cycle_count += ADVANCE_CYCLES;
            self.ppu.starting = false;
        } else {
            let row = self.mem_unit.get_memory(LY_ADDR, SOURCE) as usize;
            if self.ppu.current_sprite_search < 40 {
                'sprite_loop: for i in
                    self.ppu.current_sprite_search..(self.ppu.current_sprite_search + 5)
                {
                    let y_pos = self
                        .mem_unit
                        .get_memory(OAM_START_ADDR + i as usize * BYTES_PER_OAM_ENTRY, SOURCE);
                    if ((row + 16) as u8 >= y_pos)
                        && (((row + 16) as u8) < y_pos + self.get_obj_size())
                    {
                        let mut poss_sprite = [0u8; 4];
                        for j in 0..BYTES_PER_OAM_ENTRY {
                            poss_sprite[j] = self.mem_unit.get_memory(
                                OAM_START_ADDR + i as usize * BYTES_PER_OAM_ENTRY + j,
                                SOURCE,
                            );
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
                    self.mem_unit.ppu_mode = 3;
                    self.ppu.cycle_count = 0;
                    self.ppu.starting = true;
                }
            }
        }
    }

    fn drawing_tiles(&mut self) {
        let row = self.mem_unit.get_memory(LY_ADDR, SOURCE) as usize;
        if self.ppu.starting {
            let wx = self.mem_unit.get_memory(WX_ADDR, SOURCE) as usize;
            let wy = self.mem_unit.get_memory(WY_ADDR, SOURCE) as usize;
            self.ppu.window_row_activated =
                wy <= row && self.get_win_enable_flag() == 1 && wx > 0 && wx < 144;
            self.ppu.draw_window = self.ppu.window_row_activated && wx < 8;

            let scx = self.mem_unit.get_memory(SCX_ADDR, SOURCE) as usize;
            let scy = self.mem_unit.get_memory(SCY_ADDR, SOURCE) as usize;

            self.ppu.bg_tilemap_start_addr = if self.get_bg_tile_map_flag() == 0 {
                0x9800
            } else {
                0x9C00
            };
            self.ppu.win_tilemap_start_addr = if self.get_win_tile_map_flag() == 0 {
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

            self.ppu.bg_tilemap_row_start_index = TILES_PER_ROW * (total_bg_row / BG_TILE_HEIGHT);
            self.ppu.win_tilemap_row_start_index =
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
            self.ppu.x_precendence = vec![200u8; 160];
            self.ppu.cycle_count += ADVANCE_CYCLES;
        } else if self.ppu.column < 160 && self.ppu.bg_win_enable {
            let mut frame_index = convert_to_index(row, self.ppu.column);
            let tile_data_flag = self.get_tile_data_flag();
            let (
                tilemap_start_addr,
                tilemap_row_start_index,
                tiles_within_row_start,
                row_within_tile,
            ) = if self.ppu.draw_window {
                (
                    self.ppu.win_tilemap_start_addr,
                    self.ppu.win_tilemap_row_start_index,
                    0,
                    self.ppu.win_row_within_tile,
                )
            } else {
                (
                    self.ppu.bg_tilemap_start_addr,
                    self.ppu.bg_tilemap_row_start_index,
                    self.ppu.bg_tiles_within_row_start,
                    self.ppu.bg_row_within_tile,
                )
            };

            let tile_map_index = tilemap_row_start_index
                + (tiles_within_row_start + self.ppu.tile_num as usize) % 32;
            let tile_map_addr = tilemap_start_addr + tile_map_index;
            let (bg_priority, vertical_flip, horizontal_flip, bank_number, palette) = if self.cgb {
                let bg_attribute_data = self.mem_unit.access_vram(tile_map_addr, 1);
                (
                    (bg_attribute_data >> 7) == 1,
                    ((bg_attribute_data >> 6) & 1) == 1,
                    ((bg_attribute_data >> 5) & 1) == 1,
                    ((bg_attribute_data >> 3) & 1),
                    self.mem_unit.get_bg_rbg(bg_attribute_data & 0b111),
                )
            } else {
                (
                    false,
                    false,
                    false,
                    0,
                    [
                        [155, 188, 15, 255],
                        [139, 172, 15, 255],
                        [48, 98, 48, 255],
                        [15, 56, 15, 255],
                    ],
                )
            };
            let absolute_tile_data_index = if tile_data_flag == 1 {
                self.mem_unit.access_vram(tile_map_addr, 0) as usize
            } else {
                let initial_index = self.mem_unit.access_vram(tile_map_addr, 0) as usize;
                if initial_index < VRAM_BLOCK_SIZE {
                    initial_index + 2 * VRAM_BLOCK_SIZE
                } else {
                    initial_index
                }
            };
            let tile_data_addr = VRAM_START_ADDR + absolute_tile_data_index * BYTES_PER_TILE // getting to the starting byte
                + row_within_tile * BYTES_PER_TILE_ROW; //getting to the row
            let least_sig_byte = self.mem_unit.access_vram(tile_data_addr, bank_number);
            let most_sig_byte = self.mem_unit.access_vram(tile_data_addr + 1, bank_number);

            'pixel_loop: for pixel in self.ppu.px_within_row..8 {
                let color_index = ((((most_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1) << 1)
                    + ((least_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1))
                    as usize;
                self.ppu.row_colors[self.ppu.column] = color_index;
                self.ppu.frame_data[frame_index..(frame_index + 4)]
                    .copy_from_slice(&palette[color_index]);
                frame_index += 4;
                self.ppu.column += 1;
                if self.ppu.column == 160 {
                    break 'pixel_loop;
                }
                if (self.ppu.column + 7 == self.mem_unit.get_memory(WX_ADDR, SOURCE) as usize)
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
        } else if self.ppu.current_sprite_drawing < self.ppu.sprite_num && self.ppu.obj_enable {
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
            let least_sig_byte = self
                .mem_unit
                .get_memory(VRAM_START_ADDR + tile_data_index, SOURCE);
            let most_sig_byte = self
                .mem_unit
                .get_memory(VRAM_START_ADDR + (tile_data_index + 1), SOURCE);

            let palette = if (sprite[OAM_ATTRIBUTE_INDEX] >> 4) & 1 == 0 {
                self.mem_unit.get_memory(OBP0_ADDR, SOURCE) as usize
            } else {
                self.mem_unit.get_memory(OBP1_ADDR, SOURCE) as usize
            };
            let color_indexes: [usize; 4] = [
                palette & 0b11,
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
                    let color_index = ((((most_sig_byte >> (TILE_WIDTH - index - 1)) & 1) << 1)
                        + ((least_sig_byte >> (TILE_WIDTH - index - 1)) & 1))
                        as usize;
                    let draw_color = color_indexes[color_index] as u8;
                    if (x_start < self.ppu.x_precendence[x as usize]) //no obj with priority 
                                & (color_index != 0)
                    // not transparent
                    {
                        self.ppu.x_precendence[x as usize] = x_start;
                        if !bg_over_obj || (self.ppu.row_colors[x as usize] == 0) {
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
                self.mem_unit.ppu_mode = 0;
            }
        }
    }

    fn hblank(&mut self) {
        if self.ppu.starting {
            self.mem_unit.write_memory(
                LY_ADDR,
                self.mem_unit.get_memory(LY_ADDR, SOURCE) + 1,
                SOURCE,
            );
            if self.mem_unit.get_memory(LYC_ADDR, SOURCE)
                == self.mem_unit.get_memory(LY_ADDR, SOURCE)
            {
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
                let new_mode = if self.mem_unit.get_memory(LY_ADDR, SOURCE) == 144 {
                    let ppu_enable = self.get_ppu_enable();
                    let frame = self.pixels.get_frame();
                    if ppu_enable != 0 {
                        frame.copy_from_slice(&self.ppu.frame_data);
                    }
                    1
                } else {
                    2
                };
                self.set_mode(new_mode);
                self.mem_unit.ppu_mode = new_mode;
            }
        }
    }

    fn vblank(&mut self) {
        if self.ppu.starting {
            self.ppu.frame_num += 1;
            self.ppu.starting = false;
            self.set_vblank_interrupt();
            if self.get_stat_vblank_int_flag() == 1 {
                self.set_stat_interrupt();
            }
            self.pixels.render().unwrap();
            self.ppu.current_window_row = 0;
            self.ppu.frame_index = 0;
        }
        self.ppu.cycle_count += ADVANCE_CYCLES;
        if self.ppu.cycle_count == ROW_DOTS {
            self.ppu.cycle_count = 0;
            self.mem_unit.write_memory(
                LY_ADDR,
                self.mem_unit.get_memory(LY_ADDR, SOURCE) + 1,
                SOURCE,
            );
            if self.mem_unit.get_memory(LYC_ADDR, SOURCE)
                == self.mem_unit.get_memory(LY_ADDR, SOURCE)
            {
                self.set_stat_lyc_lc_flag();
                if self.get_stat_lyc_lc_int_flag() == 1 {
                    self.set_stat_interrupt();
                }
            } else {
                self.reset_stat_lyc_lc_flag();
            }
            if self.mem_unit.get_memory(LY_ADDR, SOURCE) == 154 {
                self.mem_unit.write_memory(LY_ADDR, 0, SOURCE);
                self.ppu.starting = true;
                self.set_mode(2);
                self.mem_unit.ppu_mode = 2;
            }
        }
    }
}

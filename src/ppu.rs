use crate::constants::*;
use crate::emulator::RequestSource;
use serde::{Deserialize, Serialize};
use std::cmp;

use crate::emulator::GameBoyEmulator;

const SOURCE: RequestSource = RequestSource::PPU;

#[inline]
fn convert_to_index(row: impl Into<usize>, column: impl Into<usize>) -> usize {
    (160 * row.into() as usize + column.into() as usize) * 4
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PictureProcessingUnit {
    pub cycle_count: u32,
    possible_sprites: Vec<[u8; 4]>,
    starting: bool,
    current_sprite_search: usize,
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
    pixel_priority: Vec<u8>,
    current_sprite_drawing: usize,
    bg_win_enable: bool,
    obj_enable: bool,
    frame_num: u32,
    frame_index: usize,
    row_colors: Vec<usize>,
    stat_line: bool,
    lyc_lc_line: bool,
    mode_2_line: bool,
    mode_1_line: bool,
    mode_0_line: bool,
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
            pixel_priority: vec![220; 160],
            current_sprite_drawing: 0,
            bg_win_enable: false,
            obj_enable: false,
            frame_num: 0,
            frame_index: 0,
            row_colors: vec![0; 160],
            stat_line: false,
            lyc_lc_line: false,
            mode_2_line: false,
            mode_1_line: false,
            mode_0_line: false,
        }
    }
}

impl GameBoyEmulator {
    fn update_stat_line(&mut self) {
        let old_line = self.ppu.stat_line;
        self.ppu.stat_line = (self.ppu.lyc_lc_line && self.get_stat_lyc_lc_int_flag())
            || (self.ppu.mode_2_line && self.get_stat_oam_int_flag())
            || (self.ppu.mode_1_line && self.get_stat_vblank_int_flag())
            || (self.ppu.mode_0_line && self.get_stat_hblank_int_flag());
        if !old_line && self.ppu.stat_line {
            self.set_stat_interrupt();
        }
    }
    fn update_ly(&mut self, ly: u8) {
        self.write_memory(LY_ADDR, ly, SOURCE);
        self.check_lyc_flag();
    }
    pub fn check_lyc_flag(&mut self) {
        self.ppu.lyc_lc_line =
            if self.get_memory(LY_ADDR, SOURCE) == self.get_memory(LYC_ADDR, SOURCE) {
                self.set_stat_lyc_lc_flag();
                true
            } else {
                self.reset_stat_lyc_lc_flag();
                false
            };
    }
    fn get_bg_tile_map_flag(&self) -> u8 {
        (self.get_memory(LCDC_ADDR, SOURCE) >> 3) & 1
    }
    fn get_win_tile_map_flag(&self) -> u8 {
        (self.get_memory(LCDC_ADDR, SOURCE) >> 6) & 1
    }
    fn get_tile_data_flag(&self) -> u8 {
        (self.get_memory(LCDC_ADDR, SOURCE) >> 4) & 1
    }
    fn get_win_enable_flag(&self) -> u8 {
        (self.get_memory(LCDC_ADDR, SOURCE) >> 5) & 1
    }
    fn get_obj_size(&self) -> u8 {
        if ((self.get_memory(LCDC_ADDR, SOURCE) >> 2) & 1) == 1 {
            16
        } else {
            8
        }
    }

    fn get_sprite_enable_flag(&self) -> u8 {
        (self.get_memory(LCDC_ADDR, SOURCE) >> 1) & 1
    }
    fn get_bg_window_enable(&self) -> u8 {
        self.get_memory(LCDC_ADDR, SOURCE) & 1
    }
    fn get_ppu_enable(&self) -> u8 {
        (self.get_memory(LCDC_ADDR, SOURCE) >> 7) & 1
    }
    fn get_stat_lyc_lc_int_flag(&self) -> bool {
        ((self.get_memory(STAT_ADDR, SOURCE) >> 6) & 1) == 1
    }
    fn get_stat_oam_int_flag(&self) -> bool {
        ((self.get_memory(STAT_ADDR, SOURCE) >> 5) & 1) == 1
    }
    fn get_stat_vblank_int_flag(&self) -> bool {
        ((self.get_memory(STAT_ADDR, SOURCE) >> 4) & 1) == 1
    }
    fn get_stat_hblank_int_flag(&self) -> bool {
        ((self.get_memory(STAT_ADDR, SOURCE) >> 3) & 1) == 1
    }
    pub fn get_mode(&self) -> u8 {
        self.get_memory(STAT_ADDR, SOURCE) & 0b11
    }
    fn set_mode(&mut self, mode: u8) {
        match mode {
            0 => {
                self.ppu.mode_0_line = true;
                self.ppu.mode_1_line = false;
                self.ppu.mode_2_line = false;
            }
            1 => {
                self.set_vblank_interrupt();
                self.ppu.mode_0_line = false;
                self.ppu.mode_1_line = true;
                self.ppu.mode_2_line = false;
            }
            2 => {
                self.ppu.mode_0_line = false;
                self.ppu.mode_1_line = false;
                self.ppu.mode_2_line = true;
            }
            3 => {
                self.ppu.mode_0_line = false;
                self.ppu.mode_1_line = false;
                self.ppu.mode_2_line = false;
            }
            _ => panic!("bad mode!"),
        }
        let write_val = (self.get_memory(STAT_ADDR, SOURCE) & 0b1111100) | mode;
        self.write_memory(STAT_ADDR, write_val, SOURCE);
        self.mem_unit.ppu_mode = mode;
    }
    fn set_stat_interrupt(&mut self) {
        self.write_memory(
            INT_FLAG_ADDR,
            self.get_memory(INT_FLAG_ADDR, SOURCE) | 0b10,
            SOURCE,
        );
    }
    fn set_stat_lyc_lc_flag(&mut self) {
        self.write_memory(
            STAT_ADDR,
            self.get_memory(STAT_ADDR, SOURCE) | 0b100,
            SOURCE,
        );
    }
    fn reset_stat_lyc_lc_flag(&mut self) {
        self.write_memory(
            STAT_ADDR,
            self.get_memory(STAT_ADDR, SOURCE) & 0b1111011,
            SOURCE,
        );
    }
    fn set_vblank_interrupt(&mut self) {
        self.write_memory(
            INT_FLAG_ADDR,
            self.get_memory(INT_FLAG_ADDR, SOURCE) | 1,
            SOURCE,
        );
    }
    pub fn ppu_advance(&mut self) {
        if self.get_ppu_enable() == 0 {
            self.set_mode(0);
            self.update_ly(0);
            self.ppu.cycle_count = 0;
            self.ppu.starting = true;
        } else {
            let mode = self.get_mode();
            match mode {
                0x0 => self.hblank(),
                0x1 => self.vblank(),
                0x2 => self.oam_search(),
                0x3 => self.drawing_tiles(),
                _ => {}
            }
        }
        self.update_stat_line();
    }

    fn oam_search(&mut self) {
        if self.ppu.starting {
            self.ppu.possible_sprites = Vec::new();
            self.ppu.sprite_num = 0;
            self.ppu.current_sprite_search = 0;
            self.ppu.cycle_count += ADVANCE_CYCLES;
            self.ppu.starting = false;
        } else {
            let row = self.get_memory(LY_ADDR, SOURCE) as usize;
            if self.ppu.current_sprite_search < OAM_SPRITE_NUM {
                'sprite_loop: for i in
                    self.ppu.current_sprite_search..(self.ppu.current_sprite_search + 5)
                {
                    let y_pos =
                        self.get_memory(OAM_START_ADDR + i * BYTES_PER_OAM_ENTRY, SOURCE) as usize;
                    let lower_y_bound = (row + 16) >= y_pos;
                    let upper_y_bound = (row + 16) < y_pos + self.get_obj_size() as usize;
                    if lower_y_bound && upper_y_bound {
                        let mut poss_sprite = [255u8; 4];
                        for j in 0..BYTES_PER_OAM_ENTRY {
                            poss_sprite[j] = self
                                .get_memory(OAM_START_ADDR + i * BYTES_PER_OAM_ENTRY + j, SOURCE);
                        }
                        self.ppu.possible_sprites.push(poss_sprite);
                        self.ppu.sprite_num += 1;
                        if self.ppu.sprite_num == MAX_SPRITES_PER_ROW {
                            self.ppu.current_sprite_search = OAM_SPRITE_NUM;
                            break 'sprite_loop;
                        }
                    }
                }
                self.ppu.current_sprite_search += 5;
                self.ppu.cycle_count += ADVANCE_CYCLES;
            } else {
                self.ppu.cycle_count += ADVANCE_CYCLES;
                if self.ppu.cycle_count == OAM_SCAN_DOTS {
                    self.set_mode(DRAWING_MODE);
                    self.ppu.cycle_count = 0;
                    self.ppu.starting = true;
                }
            }
        }
    }

    fn drawing_tiles(&mut self) {
        if self.ppu.starting {
            self.drawing_initialize();
        } else if self.ppu.column < WINDOW_WIDTH && (self.ppu.bg_win_enable || self.cgb) {
            self.bg_win_draw();
        } else if self.ppu.current_sprite_drawing < self.ppu.sprite_num && self.ppu.obj_enable {
            self.obj_draw();
        } else {
            self.ppu.cycle_count += ADVANCE_CYCLES;
            if self.ppu.cycle_count == DRAWING_DOTS {
                self.ppu.cycle_count = 0;
                self.ppu.starting = true;
                self.set_mode(HBLANK_MODE);
            }
        }
    }
    fn drawing_initialize(&mut self) {
        let row = self.get_memory(LY_ADDR, SOURCE) as usize;
        let wx = self.get_memory(WX_ADDR, SOURCE) as usize;
        let wy = self.get_memory(WY_ADDR, SOURCE) as usize;
        self.ppu.window_row_activated =
            wy <= row && self.get_win_enable_flag() == 1 && wx > 0 && wx < WINDOW_HEIGHT;
        self.ppu.draw_window = self.ppu.window_row_activated && wx < 8;

        let scx = self.get_memory(SCX_ADDR, SOURCE) as usize;
        let scy = self.get_memory(SCY_ADDR, SOURCE) as usize;

        self.ppu.bg_tilemap_start_addr = if self.get_bg_tile_map_flag() == 0 {
            TILE_MAP_1_START_ADDR
        } else {
            TILE_MAP_2_START_ADDR
        };
        self.ppu.win_tilemap_start_addr = if self.get_win_tile_map_flag() == 0 {
            TILE_MAP_1_START_ADDR
        } else {
            TILE_MAP_2_START_ADDR
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
        self.ppu.pixel_priority = vec![255u8; 160];
        self.ppu.cycle_count += ADVANCE_CYCLES;
    }
    fn bg_win_draw(&mut self) {
        let row = self.get_memory(LY_ADDR, SOURCE) as usize;
        let mut frame_index = convert_to_index(row, self.ppu.column);
        let tile_data_flag = self.get_tile_data_flag();
        let (
            tilemap_start_addr,
            tilemap_row_start_index,
            tiles_within_row_start,
            mut row_within_tile,
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
        let tile_map_index =
            tilemap_row_start_index + (tiles_within_row_start + self.ppu.tile_num as usize) % 32;
        let tile_map_addr = tilemap_start_addr + tile_map_index;
        let (bg_priority, vertical_flip, horizontal_flip, bank_number, palette) = if self.cgb {
            let bg_attribute_data = self.access_vram(tile_map_addr, 1);
            (
                (bg_attribute_data >> 7) == 1,
                ((bg_attribute_data >> 6) & 1) == 1,
                ((bg_attribute_data >> 5) & 1) == 1,
                ((bg_attribute_data >> 3) & 1),
                self.get_bg_rbg(bg_attribute_data & 0b111),
            )
        } else {
            let color_data = self.get_memory(BGP_ADDR, SOURCE) as usize;

            let palette: [[u8; 4]; 4] = [
                DMG_COLOR_MAP[color_data & 0b11],
                DMG_COLOR_MAP[(color_data >> 2) & 0b11],
                DMG_COLOR_MAP[(color_data >> 4) & 0b11],
                DMG_COLOR_MAP[(color_data >> 6) & 0b11],
            ];
            (false, false, false, 0, palette)
        };
        if vertical_flip {
            row_within_tile = 7 - row_within_tile;
        }
        let absolute_tile_data_index = if tile_data_flag == 1 {
            self.access_vram(tile_map_addr, 0) as usize
        } else {
            let initial_index = self.access_vram(tile_map_addr, 0) as usize;
            if initial_index < VRAM_BLOCK_SIZE {
                initial_index + 2 * VRAM_BLOCK_SIZE
            } else {
                initial_index
            }
        };
        let tile_data_addr = VRAM_START_ADDR + absolute_tile_data_index * BYTES_PER_TILE // getting to the starting byte
            + row_within_tile * BYTES_PER_TILE_ROW; //getting to the row
        let least_sig_byte = self.access_vram(tile_data_addr, bank_number);
        let most_sig_byte = self.access_vram(tile_data_addr + 1, bank_number);

        let bg_range = if horizontal_flip {
            (self.ppu.px_within_row..=7).rev().collect::<Vec<usize>>()
        } else {
            (self.ppu.px_within_row..=7).collect::<Vec<usize>>()
        };

        let wx = self.get_memory(WX_ADDR, SOURCE) as usize;
        let frame = self.pixels.get_frame();
        'pixel_loop: for pixel in bg_range.into_iter() {
            let color_index = ((((most_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1) << 1)
                + ((least_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1))
                as usize;
            let priority_level = if !self.ppu.bg_win_enable {
                BG_LCDC_LOW_PRIORITY
            } else if color_index == 0 {
                BG_COLOR_0_PRIORITY
            } else if bg_priority {
                BG_HIGH_PRIORITY
            } else {
                BG_COLOR_1_3_PRIORITY
            };
            self.ppu.pixel_priority[self.ppu.column] = priority_level;
            self.ppu.row_colors[self.ppu.column] = color_index;
            frame[frame_index..(frame_index + PIXEL_LENGTH)].copy_from_slice(&palette[color_index]);
            frame_index += PIXEL_LENGTH;
            self.ppu.column += 1;
            if self.ppu.column == WINDOW_WIDTH {
                break 'pixel_loop;
            }
            if (self.ppu.column + 7 == wx) && self.ppu.window_row_activated {
                self.ppu.tile_num = -1;
                self.ppu.draw_window = true;
                break 'pixel_loop;
            }
            self.ppu.px_within_row = 0;
        }
        self.ppu.tile_num += 1;
        self.ppu.cycle_count += ADVANCE_CYCLES;
    }

    fn obj_draw(&mut self) {
        let row = self.get_memory(LY_ADDR, SOURCE) as usize;
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
        let tile_data_index = tile_map_index as usize * BYTES_PER_TILE
            + (row_within % TILE_WIDTH) * BYTES_PER_TILE_ROW;

        let (palette, bank_number) = if self.cgb {
            (
                self.get_obj_rbg(sprite[OAM_ATTRIBUTE_INDEX] & 0b111),
                (sprite[OAM_ATTRIBUTE_INDEX] >> 3) & 1,
            )
        } else {
            let color_data = if (sprite[OAM_ATTRIBUTE_INDEX] >> 4) & 1 == 0 {
                self.get_memory(OBP0_ADDR, SOURCE) as usize
            } else {
                self.get_memory(OBP1_ADDR, SOURCE) as usize
            };
            let palette: [[u8; 4]; 4] = [
                DMG_COLOR_MAP[color_data & 0b11],
                DMG_COLOR_MAP[(color_data >> 2) & 0b11],
                DMG_COLOR_MAP[(color_data >> 4) & 0b11],
                DMG_COLOR_MAP[(color_data >> 6) & 0b11],
            ];
            (palette, 0)
        };
        let least_sig_byte = self.access_vram(VRAM_START_ADDR + tile_data_index, bank_number);
        let most_sig_byte = self.access_vram(VRAM_START_ADDR + (tile_data_index + 1), bank_number);
        let x_end = sprite[OAM_X_INDEX];
        let priority = if bg_over_obj {
            OAM_LOW_PRIORITY
        } else if self.cgb {
            self.ppu.current_sprite_drawing as u8
        } else {
            x_end
        };
        if x_end > 0 && x_end < (WINDOW_WIDTH + TILE_WIDTH) as u8 {
            let x_start = cmp::max(0, x_end as isize - 8) as u8;
            let x_end = cmp::min(160, x_end);
            let mut x = x_start as usize;
            let starting_pixel = if x_end < 7 { 7 - x_end as usize } else { 0 };
            let ending_pixel = if x_start > (WINDOW_WIDTH - TILE_WIDTH) as u8 {
                WINDOW_WIDTH - 1 - x_start as usize
            } else {
                TILE_WIDTH - 1
            };
            let obj_range = if x_flip {
                (starting_pixel..=ending_pixel)
                    .rev()
                    .collect::<Vec<usize>>()
            } else {
                (starting_pixel..=ending_pixel).collect::<Vec<usize>>()
            };
            let mut frame_index = convert_to_index(row, x_start);
            let frame = self.pixels.get_frame();
            for pixel in obj_range.into_iter() {
                let color_index = ((((most_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1) << 1)
                    + ((least_sig_byte >> (TILE_WIDTH - pixel - 1)) & 1))
                    as usize;
                let color_pixels = palette[color_index];
                if (priority < self.ppu.pixel_priority[x]) && (color_index != 0) {
                    self.ppu.pixel_priority[x] = priority;
                    frame[frame_index..(frame_index + 4)].copy_from_slice(&color_pixels);
                }
                frame_index += 4;
                x += 1;
            }
        }

        self.ppu.current_sprite_drawing += 1;
        self.ppu.cycle_count += ADVANCE_CYCLES;
    }

    fn hblank(&mut self) {
        if self.ppu.starting {
            if self.ppu.window_row_activated {
                self.ppu.current_window_row += 1;
            }
            self.ppu.cycle_count += ADVANCE_CYCLES;
            self.ppu.starting = false;
        } else {
            self.ppu.cycle_count += ADVANCE_CYCLES;
            if self.ppu.cycle_count == HBLANK_DOTS {
                let ly = self.get_memory(LY_ADDR, SOURCE) + 1;
                self.update_ly(ly);
                self.ppu.cycle_count = 0;
                self.ppu.starting = true;
                let new_mode = if ly == 144 {
                    VBLANK_MODE
                } else {
                    OAM_SEARCH_MODE
                };
                self.set_mode(new_mode);
            }
        }
    }

    fn vblank(&mut self) {
        if self.ppu.starting {
            self.ppu.frame_num += 1;
            self.ppu.starting = false;
            self.pixels.render().unwrap();
            self.ppu.current_window_row = 0;
            self.ppu.frame_index = 0;
        }
        self.ppu.cycle_count += ADVANCE_CYCLES;
        if self.ppu.cycle_count == ROW_DOTS {
            self.ppu.cycle_count = 0;
            let ly = (self.get_memory(LY_ADDR, SOURCE) + 1) % 154;
            self.update_ly(ly);
            if ly == 0 {
                self.ppu.starting = true;
                self.set_mode(OAM_SEARCH_MODE);
            }
        }
    }
}

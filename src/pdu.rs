use crate::emulator::GameBoyEmulator;
use crate::emulator::RequestSource;
use crate::emulator::ADVANCE_CYCLES;
use pixels::Pixels;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

// const COLOR_MAP: [Color; 5] = [
//     Color::RGB(155, 188, 15),
//     Color::RGB(139, 172, 15),
//     Color::RGB(48, 98, 48),
//     Color::RGB(15, 56, 15),
//     Color::RGB(255, 255, 255),
// ];

const COLOR_MAP: [[u8; 4]; 4] = [
    [155, 188, 15, 255],
    [139, 172, 15, 255],
    [48, 98, 48, 255],
    [15, 56, 15, 255],
];

const WINDOW_WIDTH: usize = 160;
const WINDOW_HEIGHT: usize = 144;
const TOTAL_PIXELS: usize = WINDOW_HEIGHT * WINDOW_WIDTH;
const LY_ADDR: usize = 0xFF44;
const SOURCE: RequestSource = RequestSource::PDU;
const PIXEL_LENGTH: usize = 4;

enum DisplayMode {
    DataInput(usize, usize),
    Cloning,
    Drawing,
    Waiting,
}
pub struct PictureDisplayUnit {
    pub pixels: Pixels,
    current_mode: DisplayMode,
    frame_data: Vec<u8>,
}

impl PictureDisplayUnit {
    pub fn new(pixels: Pixels) -> PictureDisplayUnit {
        PictureDisplayUnit {
            pixels,
            current_mode: DisplayMode::DataInput(0, 0),
            frame_data: vec![0; TOTAL_PIXELS * PIXEL_LENGTH],
        }
    }
}

impl GameBoyEmulator {
    pub fn pdu_advance(&mut self) {
        match self.pdu.current_mode {
            DisplayMode::DataInput(mut row, mut column) => {
                if self.mem_unit.get_memory(LY_ADDR, SOURCE) as usize > row {
                    let mut data_index = (row * WINDOW_WIDTH + column) * PIXEL_LENGTH;
                    self.pdu.frame_data[data_index..(data_index + PIXEL_LENGTH)]
                        .copy_from_slice(&COLOR_MAP[self.frame[row][column] as usize]);
                    column += 1;
                    data_index += PIXEL_LENGTH;

                    self.pdu.frame_data[data_index..(data_index + PIXEL_LENGTH)]
                        .copy_from_slice(&COLOR_MAP[self.frame[row][column] as usize]);
                    column += 1;

                    if column == WINDOW_WIDTH {
                        row += 1;
                        self.pdu.current_mode = if row == WINDOW_HEIGHT {
                            DisplayMode::Cloning
                        } else {
                            DisplayMode::DataInput(row, 0)
                        }
                    } else {
                        self.pdu.current_mode = DisplayMode::DataInput(row, column);
                    }
                }
            }
            DisplayMode::Cloning => {
                let frame = self.pdu.pixels.get_frame();
                frame.copy_from_slice(&self.pdu.frame_data[..]);
                self.pdu.current_mode = DisplayMode::Drawing
            }
            DisplayMode::Drawing => {
                self.pdu.pixels.render().unwrap();
                self.pdu.current_mode = DisplayMode::Waiting
            }
            DisplayMode::Waiting => {
                if self.mem_unit.get_memory(LY_ADDR, SOURCE) == 0 {
                    self.pdu.current_mode = DisplayMode::DataInput(0, 0);
                }
            }
        }
    }
}

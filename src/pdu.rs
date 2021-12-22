use crate::emulator::GameBoyEmulator;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

const COLOR_MAP: [Color; 5] = [
    Color::RGB(155, 188, 15),
    Color::RGB(139, 172, 15),
    Color::RGB(48, 98, 48),
    Color::RGB(15, 56, 15),
    Color::RGB(255, 255, 255),
];

const WINDOW_WIDTH: usize = 160;
const WINDOW_HEIGHT: usize = 144;
const VBLANK_DOTS: usize = 4560;

pub struct PictureDisplayUnit {
    pub canvas: Canvas<Window>,
    pub height: usize,
    pub width: usize,
    point_vecs: [Vec<Point>; 4],
    row: usize,
    ready: bool,
    draw_color: usize,
    pub sample_map: Vec<(usize, usize)>,
}

impl PictureDisplayUnit {
    pub fn new(canvas: Canvas<Window>) -> PictureDisplayUnit {
        let point_vecs = [
            Vec::<Point>::new(),
            Vec::<Point>::new(),
            Vec::<Point>::new(),
            Vec::<Point>::new(),
        ];
        let row = 0;
        let ready = true;
        let draw_color = 0;
        let mut sample_map = Vec::new();
        for i in 0..WINDOW_HEIGHT {
            for j in 0..WINDOW_WIDTH {
                sample_map.push((i, j));
            }
        }
        PictureDisplayUnit {
            canvas,
            height: WINDOW_HEIGHT as usize,
            width: WINDOW_WIDTH as usize,
            point_vecs,
            row,
            ready,
            draw_color,
            sample_map,
        }
    }
}

impl GameBoyEmulator {
    pub fn pdu_advance(&mut self) {
        if self.get_mode() == 1 {
            if self.pdu.ready {
                if self.pdu.row < self.pdu.height {
                    for column in 0..self.pdu.width {
                        let (frame_row, frame_column) =
                            self.pdu.sample_map[self.pdu.row * self.pdu.width + column];
                        let color_choice = self.frame[frame_row][frame_column];
                        self.pdu.point_vecs[color_choice as usize]
                            .push(Point::new(column as i32, self.pdu.row as i32));
                    }
                    self.pdu.row += 1;
                } else {
                    if self.pdu.draw_color < 4 {
                        self.pdu
                            .canvas
                            .set_draw_color(COLOR_MAP[self.pdu.draw_color]);
                        self.pdu
                            .canvas
                            .draw_points(&self.pdu.point_vecs[self.pdu.draw_color][..])
                            .expect("Draw failure");
                        self.pdu.draw_color += 1;
                    } else {
                        self.pdu.canvas.present();
                        self.pdu.ready = false;
                        self.pdu.row = 0;
                        self.pdu.draw_color = 0;
                        self.pdu.point_vecs = [
                            Vec::<Point>::new(),
                            Vec::<Point>::new(),
                            Vec::<Point>::new(),
                            Vec::<Point>::new(),
                        ];
                    }
                }
            }
        } else {
            self.pdu.ready = true
        }
    }
}

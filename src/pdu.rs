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

pub struct PictureDisplayUnit {
    pub canvas: Canvas<Window>,
    pub height: usize,
    pub width: usize,
    point_vecs: [Vec<Point>; 4],
    row: usize,
    ready: bool,
    draw_color: usize,
    pub height_scale_factor: f32,
    pub width_scale_factor: f32,
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

        PictureDisplayUnit {
            canvas,
            height: WINDOW_HEIGHT,
            width: WINDOW_WIDTH,
            point_vecs,
            row,
            ready,
            draw_color,
            height_scale_factor: 1.0,
            width_scale_factor: 1.0,
        }
    }
}

impl GameBoyEmulator {
    pub fn pdu_advance(&mut self) {
        if self.get_mode() == 1 {
            if self.pdu.ready {
                if self.pdu.row < self.pdu.height {
                    let row_samp =
                        (self.pdu.row as f32 * self.pdu.height_scale_factor).round() as usize;
                    for column in 0..self.pdu.width {
                        let column_samp =
                            (column as f32 * self.pdu.width_scale_factor).round() as usize;
                        let color_choice = self.frame[row_samp][column_samp];
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

use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::sync::{Arc, Mutex};

const COLOR_MAP: [Color; 5] = [
    Color::RGB(155, 188, 15),
    Color::RGB(139, 172, 15),
    Color::RGB(48, 98, 48),
    Color::RGB(15, 56, 15),
    Color::RGB(255, 255, 255),
];
pub struct PictureDisplayUnit {
    canvas: Canvas<Window>,
    point_vecs: [Vec<Point>; 4],
    stat: Arc<Mutex<u8>>,
    row: usize,
    ready: bool,
    frame: Arc<Mutex<[[u8; 160]; 144]>>,
    draw_color: usize,
}

impl PictureDisplayUnit {
    pub fn new(
        canvas: Canvas<Window>,
        stat: Arc<Mutex<u8>>,
        frame: Arc<Mutex<[[u8; 160]; 144]>>,
    ) -> PictureDisplayUnit {
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
            point_vecs,
            stat,
            row,
            ready,
            frame,
            draw_color,
        }
    }
    fn get_mode(&self) -> u8 {
        *self.stat.lock().unwrap() & 0b11
    }
    pub fn advance(&mut self) {
        if self.get_mode() == 1 {
            if self.ready {
                if self.row < 144 {
                    let frame = self.frame.lock().unwrap();
                    for column in 0..160 {
                        self.point_vecs[frame[self.row][column] as usize]
                            .push(Point::new(column as i32, self.row as i32));
                    }
                    self.row += 1;
                } else {
                    if self.draw_color < 4 {
                        self.canvas.set_draw_color(COLOR_MAP[self.draw_color]);
                        self.canvas
                            .draw_points(&self.point_vecs[self.draw_color][..])
                            .expect("Draw failure");
                        self.draw_color += 1;
                    } else {
                        self.canvas.present();
                        self.ready = false;
                        self.row = 0;
                        self.draw_color = 0;
                        self.point_vecs = [
                            Vec::<Point>::new(),
                            Vec::<Point>::new(),
                            Vec::<Point>::new(),
                            Vec::<Point>::new(),
                        ];
                    }
                }
            }
        } else {
            self.ready = true
        }
    }
}

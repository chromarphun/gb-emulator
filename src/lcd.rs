use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

const WINDOW_WIDTH: usize = 160;
const WINDOW_HEIGHT: usize = 144;

const COLOR_MAP: [Color; 4] = [
    Color::RGB(155, 188, 15),
    Color::RGB(139, 172, 15),
    Color::RGB(48, 98, 48),
    Color::RGB(15, 56, 15),
];

pub struct DisplayUnit {
    reciever: mpsc::Receiver<[[u8; 160]; 144]>,
    interrupt_flag: Arc<Mutex<u8>>,
    p1: Arc<Mutex<u8>>,
}

impl DisplayUnit {
    pub fn new(
        reciever: mpsc::Receiver<[[u8; 160]; 144]>,
        interrupt_flag: Arc<Mutex<u8>>,
        p1: Arc<Mutex<u8>>,
    ) -> DisplayUnit {
        DisplayUnit {
            reciever,
            interrupt_flag,
            p1,
        }
    }
    pub fn run(&mut self) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(
                "Gameboy Emulator",
                WINDOW_WIDTH as u32,
                WINDOW_HEIGHT as u32,
            )
            .position_centered()
            .build()
            .unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        'running: loop {
            let frame_option = self.reciever.try_recv();
            match frame_option {
                Ok(frame) => {
                    for row in 0..WINDOW_HEIGHT {
                        for column in 0..WINDOW_WIDTH {
                            let pixel_color = COLOR_MAP[frame[row][column] as usize];
                            canvas.set_draw_color(pixel_color);
                            canvas
                                .draw_point(Point::new(column as i32, row as i32))
                                .expect("Failed drawing");
                        }
                    }
                    canvas.present();
                }
                _ => {}
            }
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
        }
    }
}

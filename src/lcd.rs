use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const WINDOW_WIDTH: usize = 160;
const WINDOW_HEIGHT: usize = 144;

const COLOR_MAP: [Color; 5] = [
    Color::RGB(155, 188, 15),
    Color::RGB(139, 172, 15),
    Color::RGB(48, 98, 48),
    Color::RGB(15, 56, 15),
    Color::RGB(255, 255, 255),
];

enum RomType {}
pub struct DisplayUnit {
    reciever: mpsc::Receiver<[[u8; 160]; 144]>,
    interrupt_flag: Arc<Mutex<u8>>,
    p1: Arc<Mutex<u8>>,
    debug_var: u8,
    directional_presses: Arc<Mutex<u8>>,
    action_presses: Arc<Mutex<u8>>,
}

impl DisplayUnit {
    pub fn new(
        reciever: mpsc::Receiver<[[u8; 160]; 144]>,
        interrupt_flag: Arc<Mutex<u8>>,
        p1: Arc<Mutex<u8>>,
        directional_presses: Arc<Mutex<u8>>,
        action_presses: Arc<Mutex<u8>>,
    ) -> DisplayUnit {
        let debug_var = 0;
        DisplayUnit {
            reciever,
            interrupt_flag,
            p1,
            debug_var,
            directional_presses,
            action_presses,
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
        let mut now = Instant::now();
        let mut frame_num = 0;
        'running: loop {
            let frame_option = self.reciever.try_recv();
            let mut point_vec_0: Vec<Point> = Vec::new();
            let mut point_vec_1: Vec<Point> = Vec::new();
            let mut point_vec_2: Vec<Point> = Vec::new();
            let mut point_vec_3: Vec<Point> = Vec::new();
            match frame_option {
                Ok(frame) => {
                    for row in 0..WINDOW_HEIGHT {
                        for column in 0..WINDOW_WIDTH {
                            // let pixel_color = COLOR_MAP[frame[row][column] as usize];
                            // canvas.set_draw_color(pixel_color);
                            // canvas
                            //     .draw_point(Point::new(column as i32, row as i32))
                            //     .expect("Failed drawing");
                            match frame[row][column] {
                                0 => point_vec_0.push(Point::new(column as i32, row as i32)),
                                1 => point_vec_1.push(Point::new(column as i32, row as i32)),
                                2 => point_vec_2.push(Point::new(column as i32, row as i32)),
                                3 => point_vec_3.push(Point::new(column as i32, row as i32)),
                                _ => {}
                            }
                        }
                    }
                    canvas.set_draw_color(COLOR_MAP[0]);
                    canvas.draw_points(&point_vec_0[..]);
                    canvas.set_draw_color(COLOR_MAP[1]);
                    canvas.draw_points(&point_vec_1[..]);
                    canvas.set_draw_color(COLOR_MAP[2]);
                    canvas.draw_points(&point_vec_2[..]);
                    canvas.set_draw_color(COLOR_MAP[3]);
                    canvas.draw_points(&point_vec_3[..]);
                    canvas.present();
                }
                _ => {}
            }
            //spin_sleep::sleep(Duration::new(0, 10_000_000));
            let prev_p1 = *self.p1.lock().unwrap();
            let mut new_directional_presses = 0xF;
            let mut new_action_presses = 0xF;
            let state = event_pump.keyboard_state();
            for code in state.pressed_scancodes() {
                match code {
                    Scancode::Z => new_action_presses &= 0b1110,
                    Scancode::X => new_action_presses &= 0b1101,
                    Scancode::S => new_action_presses &= 0b1011,
                    Scancode::A => new_action_presses &= 0b0111,
                    Scancode::Right => new_directional_presses &= 0b1110,
                    Scancode::Left => new_directional_presses &= 0b1101,
                    Scancode::Up => new_directional_presses &= 0b1011,
                    Scancode::Down => new_directional_presses &= 0b0111,
                    _ => {}
                }
            }
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        scancode: Some(Scancode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }
            //create context for mutex to drop
            {
                let mut p1 = self.p1.lock().unwrap();
                *self.directional_presses.lock().unwrap() = new_directional_presses;
                *self.action_presses.lock().unwrap() = new_action_presses;
                let p14 = (*p1 >> 4) & 1;
                let p15 = (*p1 >> 5) & 1;
                let mut new_bits = 0xF;
                *p1 &= 0b110000;

                if p14 == 0 {
                    new_bits &= new_directional_presses;
                }
                if p15 == 0 {
                    new_bits &= new_action_presses;
                }
                *p1 += new_bits;
                if ((prev_p1 | *p1) - *p1) & 0xF != 0 {
                    *self.interrupt_flag.lock().unwrap() |= 1 << 4;
                }
            }
            // frame_num += 1;
            // if frame_num == 60 {
            //     println!(
            //         "FPS: {}",
            //         (frame_num as f64 / (now.elapsed().as_nanos() as f64)) * 1000000000.0
            //     );
            //     frame_num = 0;
            //     now = Instant::now();
            // }
        }
    }
}

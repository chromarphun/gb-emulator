use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::EventPump;
use std::sync::{Arc, Mutex};

enum Stage {
    GetState,
    SetIntermediateRegs,
    SetMainRegs,
    CheckQuit,
}

pub struct EventProcessingUnit {
    stage: Stage,
    p1: Arc<Mutex<u8>>,
    directional_presses: Arc<Mutex<u8>>,
    action_presses: Arc<Mutex<u8>>,
    interrupt_flag: Arc<Mutex<u8>>,
    running: Arc<Mutex<bool>>,
    new_directional_presses: u8,
    new_action_presses: u8,
    event_pump: EventPump,
}

impl EventProcessingUnit {
    pub fn new(
        p1: Arc<Mutex<u8>>,
        directional_presses: Arc<Mutex<u8>>,
        action_presses: Arc<Mutex<u8>>,
        interrupt_flag: Arc<Mutex<u8>>,
        running: Arc<Mutex<bool>>,
        event_pump: EventPump,
    ) -> EventProcessingUnit {
        let stage = Stage::GetState;
        let new_directional_presses = 0xF;
        let new_action_presses = 0xF;
        EventProcessingUnit {
            stage,
            p1,
            directional_presses,
            action_presses,
            interrupt_flag,
            running,
            new_directional_presses,
            new_action_presses,
            event_pump,
        }
    }
    pub fn advance(&mut self) {
        match self.stage {
            Stage::GetState => {
                self.new_directional_presses = 0xF;
                self.new_action_presses = 0xF;
                let state = self.event_pump.keyboard_state();
                for code in state.pressed_scancodes() {
                    match code {
                        Scancode::Z => self.new_action_presses &= 0b1110,
                        Scancode::X => self.new_action_presses &= 0b1101,
                        Scancode::S => self.new_action_presses &= 0b1011,
                        Scancode::A => self.new_action_presses &= 0b0111,
                        Scancode::Right => self.new_directional_presses &= 0b1110,
                        Scancode::Left => self.new_directional_presses &= 0b1101,
                        Scancode::Up => self.new_directional_presses &= 0b1011,
                        Scancode::Down => self.new_directional_presses &= 0b0111,
                        _ => {}
                    }
                }
                self.stage = Stage::SetIntermediateRegs;
            }
            Stage::SetIntermediateRegs => {
                *self.directional_presses.lock().unwrap() = self.new_directional_presses;
                *self.action_presses.lock().unwrap() = self.new_action_presses;
                self.stage = Stage::SetMainRegs;
            }
            Stage::SetMainRegs => {
                let mut p1 = self.p1.lock().unwrap();
                let prev_p1 = *p1;
                *self.directional_presses.lock().unwrap() = self.new_directional_presses;
                *self.action_presses.lock().unwrap() = self.new_action_presses;
                let p14 = (*p1 >> 4) & 1;
                let p15 = (*p1 >> 5) & 1;
                let mut new_bits = 0xF;
                *p1 &= 0b110000;

                if p14 == 0 {
                    new_bits &= self.new_directional_presses;
                }
                if p15 == 0 {
                    new_bits &= self.new_action_presses;
                }
                *p1 += new_bits;
                if ((prev_p1 | *p1) - *p1) & 0xF != 0 {
                    *self.interrupt_flag.lock().unwrap() |= 1 << 4;
                }
                self.stage = Stage::CheckQuit;
            }
            Stage::CheckQuit => {
                for event in self.event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            scancode: Some(Scancode::Escape),
                            ..
                        } => *self.running.lock().unwrap() = false,
                        _ => {}
                    }
                }
                self.stage = Stage::GetState;
            }
        }
    }
    pub fn total_advance(&mut self) {
        self.new_directional_presses = 0xF;
        self.new_action_presses = 0xF;
        let state = self.event_pump.keyboard_state();
        for code in state.pressed_scancodes() {
            match code {
                Scancode::Z => self.new_action_presses &= 0b1110,
                Scancode::X => self.new_action_presses &= 0b1101,
                Scancode::S => self.new_action_presses &= 0b1011,
                Scancode::A => self.new_action_presses &= 0b0111,
                Scancode::Right => self.new_directional_presses &= 0b1110,
                Scancode::Left => self.new_directional_presses &= 0b1101,
                Scancode::Up => self.new_directional_presses &= 0b1011,
                Scancode::Down => self.new_directional_presses &= 0b0111,
                _ => {}
            }
        }
        *self.directional_presses.lock().unwrap() = self.new_directional_presses;
        *self.action_presses.lock().unwrap() = self.new_action_presses;
        let mut p1 = self.p1.lock().unwrap();
        let prev_p1 = *p1;
        *self.directional_presses.lock().unwrap() = self.new_directional_presses;
        *self.action_presses.lock().unwrap() = self.new_action_presses;
        let p14 = (*p1 >> 4) & 1;
        let p15 = (*p1 >> 5) & 1;
        let mut new_bits = 0xF;
        *p1 &= 0b110000;

        if p14 == 0 {
            new_bits &= self.new_directional_presses;
        }
        if p15 == 0 {
            new_bits &= self.new_action_presses;
        }
        *p1 += new_bits;
        if ((prev_p1 | *p1) - *p1) & 0xF != 0 {
            *self.interrupt_flag.lock().unwrap() |= 1 << 4;
        }
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    scancode: Some(Scancode::Escape),
                    ..
                } => *self.running.lock().unwrap() = false,
                _ => {}
            }
        }
    }
}

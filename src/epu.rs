use crate::emulator::GameBoyEmulator;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::EventPump;

const P1_ADDR: usize = 0xFF00;
const INT_FLAG_ADDR: usize = 0xFF0F;

pub struct EventProcessingUnit {
    new_directional_presses: u8,
    new_action_presses: u8,
    event_pump: EventPump,
}

impl EventProcessingUnit {
    pub fn new(event_pump: EventPump) -> EventProcessingUnit {
        let new_directional_presses = 0xF;
        let new_action_presses = 0xF;
        EventProcessingUnit {
            new_directional_presses,
            new_action_presses,
            event_pump,
        }
    }
}
impl GameBoyEmulator {
    pub fn event_check(&mut self) {
        self.epu.new_directional_presses = 0xF;
        self.epu.new_action_presses = 0xF;
        let state = self.epu.event_pump.keyboard_state();
        for code in state.pressed_scancodes() {
            match code {
                Scancode::Z => self.epu.new_action_presses &= 0b1110,
                Scancode::X => self.epu.new_action_presses &= 0b1101,
                Scancode::S => self.epu.new_action_presses &= 0b1011,
                Scancode::A => self.epu.new_action_presses &= 0b0111,
                Scancode::Right => self.epu.new_directional_presses &= 0b1110,
                Scancode::Left => self.epu.new_directional_presses &= 0b1101,
                Scancode::Up => self.epu.new_directional_presses &= 0b1011,
                Scancode::Down => self.epu.new_directional_presses &= 0b0111,
                _ => {}
            }
        }
        self.mem_unit.directional_presses = self.epu.new_directional_presses;
        self.mem_unit.action_presses = self.epu.new_action_presses;
        let mut p1 = self.mem_unit.get_memory(P1_ADDR);
        let prev_p1 = p1;
        self.mem_unit.directional_presses = self.epu.new_directional_presses;
        self.mem_unit.action_presses = self.epu.new_action_presses;
        let p14 = (p1 >> 4) & 1;
        let p15 = (p1 >> 5) & 1;
        let mut new_bits = 0xF;

        p1 &= 0b110000;

        if p14 == 0 {
            new_bits &= self.epu.new_directional_presses;
        }
        if p15 == 0 {
            new_bits &= self.epu.new_action_presses;
        }
        p1 += new_bits;
        if ((prev_p1 | p1) - p1) & 0xF != 0 {
            self.mem_unit.write_memory(
                INT_FLAG_ADDR,
                self.mem_unit.get_memory(INT_FLAG_ADDR) | (1 << 4),
            );
        }
        self.mem_unit.write_memory(P1_ADDR, p1);

        for event in self.epu.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    scancode: Some(Scancode::Escape),
                    ..
                } => self.running = false,
                _ => {}
            }
        }
    }
}

use crate::constants::*;
use crate::emulator::GameBoyEmulator;
use crate::emulator::RequestSource;
use serde::{Deserialize, Serialize};

const SOURCE: RequestSource = RequestSource::CPU;

const FUNCTION_MAP: [fn(&mut GameBoyEmulator, u8); 256] = [
    //0x00
    GameBoyEmulator::nop,
    GameBoyEmulator::ld_reg_16,
    GameBoyEmulator::ld_addr_a,
    GameBoyEmulator::inc_reg_16,
    GameBoyEmulator::inc_reg_8,
    GameBoyEmulator::dec_reg_8,
    GameBoyEmulator::ld_reg_8,
    GameBoyEmulator::rlca,
    GameBoyEmulator::ld_addr_sp,
    GameBoyEmulator::add_hl_reg_16,
    GameBoyEmulator::ld_a_addr,
    GameBoyEmulator::dec_reg_16,
    GameBoyEmulator::inc_reg_8,
    GameBoyEmulator::dec_reg_8,
    GameBoyEmulator::ld_reg_8,
    GameBoyEmulator::rrca,
    //0x10
    GameBoyEmulator::stop,
    GameBoyEmulator::ld_reg_16,
    GameBoyEmulator::ld_addr_a,
    GameBoyEmulator::inc_reg_16,
    GameBoyEmulator::inc_reg_8,
    GameBoyEmulator::dec_reg_8,
    GameBoyEmulator::ld_reg_8,
    GameBoyEmulator::rla,
    GameBoyEmulator::jr,
    GameBoyEmulator::add_hl_reg_16,
    GameBoyEmulator::ld_a_addr,
    GameBoyEmulator::dec_reg_16,
    GameBoyEmulator::inc_reg_8,
    GameBoyEmulator::dec_reg_8,
    GameBoyEmulator::ld_reg_8,
    GameBoyEmulator::rra,
    //0x20
    GameBoyEmulator::jr,
    GameBoyEmulator::ld_reg_16,
    GameBoyEmulator::ld_addr_a,
    GameBoyEmulator::inc_reg_16,
    GameBoyEmulator::inc_reg_8,
    GameBoyEmulator::dec_reg_8,
    GameBoyEmulator::ld_reg_8,
    GameBoyEmulator::daa,
    GameBoyEmulator::jr,
    GameBoyEmulator::add_hl_reg_16,
    GameBoyEmulator::ld_a_addr,
    GameBoyEmulator::dec_reg_16,
    GameBoyEmulator::inc_reg_8,
    GameBoyEmulator::dec_reg_8,
    GameBoyEmulator::ld_reg_8,
    GameBoyEmulator::cpl,
    //0x30
    GameBoyEmulator::jr,
    GameBoyEmulator::ld_reg_16,
    GameBoyEmulator::ld_addr_a,
    GameBoyEmulator::inc_reg_16,
    GameBoyEmulator::inc_reg_8,
    GameBoyEmulator::dec_reg_8,
    GameBoyEmulator::ld_reg_8,
    GameBoyEmulator::scf,
    GameBoyEmulator::jr,
    GameBoyEmulator::add_hl_reg_16,
    GameBoyEmulator::ld_a_addr,
    GameBoyEmulator::dec_reg_16,
    GameBoyEmulator::inc_reg_8,
    GameBoyEmulator::dec_reg_8,
    GameBoyEmulator::ld_reg_8,
    GameBoyEmulator::ccf,
    //0x40
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_hl_addr,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_hl_addr,
    GameBoyEmulator::ld_reg_reg,
    //0x50
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_hl_addr,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_hl_addr,
    GameBoyEmulator::ld_reg_reg,
    //0x60
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_hl_addr,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_hl_addr,
    GameBoyEmulator::ld_reg_reg,
    //0x70
    GameBoyEmulator::ld_hl_addr_reg,
    GameBoyEmulator::ld_hl_addr_reg,
    GameBoyEmulator::ld_hl_addr_reg,
    GameBoyEmulator::ld_hl_addr_reg,
    GameBoyEmulator::ld_hl_addr_reg,
    GameBoyEmulator::ld_hl_addr_reg,
    GameBoyEmulator::halt,
    GameBoyEmulator::ld_hl_addr_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_reg,
    GameBoyEmulator::ld_reg_hl_addr,
    GameBoyEmulator::ld_reg_reg,
    //0x80
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    //0x90
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    //0xA0
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    //0xB0
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::arthimetic_a,
    //0xC0
    GameBoyEmulator::ret,
    GameBoyEmulator::pop,
    GameBoyEmulator::jp,
    GameBoyEmulator::jp,
    GameBoyEmulator::call,
    GameBoyEmulator::push,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::rst,
    GameBoyEmulator::ret,
    GameBoyEmulator::ret,
    GameBoyEmulator::jp,
    GameBoyEmulator::cb,
    GameBoyEmulator::call,
    GameBoyEmulator::call,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::rst,
    //0xD0
    GameBoyEmulator::ret,
    GameBoyEmulator::pop,
    GameBoyEmulator::jp,
    GameBoyEmulator::fail,
    GameBoyEmulator::call,
    GameBoyEmulator::push,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::rst,
    GameBoyEmulator::ret,
    GameBoyEmulator::ret,
    GameBoyEmulator::jp,
    GameBoyEmulator::fail,
    GameBoyEmulator::call,
    GameBoyEmulator::fail,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::rst,
    //0xE0
    GameBoyEmulator::ld_addr_a,
    GameBoyEmulator::pop,
    GameBoyEmulator::ld_addr_a,
    GameBoyEmulator::fail,
    GameBoyEmulator::fail,
    GameBoyEmulator::push,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::rst,
    GameBoyEmulator::add_sp_i8,
    GameBoyEmulator::jp_hl,
    GameBoyEmulator::ld_addr_a,
    GameBoyEmulator::fail,
    GameBoyEmulator::fail,
    GameBoyEmulator::fail,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::rst,
    //0xF0
    GameBoyEmulator::ld_a_addr,
    GameBoyEmulator::pop,
    GameBoyEmulator::ld_a_addr,
    GameBoyEmulator::di,
    GameBoyEmulator::fail,
    GameBoyEmulator::push,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::rst,
    GameBoyEmulator::ld_hl_sp_i8,
    GameBoyEmulator::ld_sp_hl,
    GameBoyEmulator::ld_a_addr,
    GameBoyEmulator::ei,
    GameBoyEmulator::fail,
    GameBoyEmulator::fail,
    GameBoyEmulator::arthimetic_a,
    GameBoyEmulator::rst,
];

const CYCLES_MAP: [u32; 256] = [
    4, 12, 8, 8, 4, 4, 8, 4, 20, 8, 8, 8, 4, 4, 8, 4, // 0x0
    4, 12, 8, 8, 4, 4, 8, 4, 12, 8, 8, 8, 4, 4, 8, 4, // 0x1
    8, 12, 8, 8, 4, 4, 8, 4, 8, 8, 8, 8, 4, 4, 8, 4, // 0x2
    8, 12, 8, 8, 12, 12, 12, 4, 8, 8, 8, 8, 4, 4, 8, 4, // 0x3
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0x4
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0x5
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0x6
    8, 8, 8, 8, 8, 8, 4, 8, 4, 4, 4, 4, 4, 4, 8, 4, // 0x7
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0x8
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0x9
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0xA
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0xB
    8, 12, 12, 16, 12, 16, 8, 16, 8, 16, 12, 8, 12, 24, 8, 16, // 0xC
    8, 12, 12, 0, 12, 16, 8, 16, 8, 16, 12, 0, 12, 0, 8, 16, // 0xD
    12, 12, 8, 0, 0, 16, 8, 16, 16, 4, 16, 0, 0, 0, 8, 16, // 0xE
    12, 12, 8, 4, 0, 16, 8, 16, 12, 8, 16, 4, 0, 0, 8, 16, // 0xF
];

#[inline]
fn combine_bytes(high_byte: u8, low_byte: u8) -> u16 {
    ((high_byte as u16) << 8) + low_byte as u16
}

#[inline]
fn split_u16(val: u16) -> (u8, u8) {
    ((val >> 8) as u8, (val & 0xFF) as u8)
}

#[inline]
fn split_byte(val: u8) -> (u8, u8) {
    (val >> 4, val & 0xF)
}
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct CentralProcessingUnit {
    ime: bool,
    regs: [u8; NUM_REG],
    pc: u16,
    sp: u16,
    cycle_modification: u32,
    z_flag: u8,
    n_flag: u8,
    h_flag: u8,
    c_flag: u8,
    reenable_interrupts: bool,
    change_ime_true: bool,
    halting: bool,
    cycle_count: u32,
    repeat: bool,
    old_pc: u16,
    pub waiting: bool,
    pub cycle_goal: u32,
    call_counter: i32,
    debug_var: i32,
    pub debug_action: bool,
    command: usize,
}

impl CentralProcessingUnit {
    pub fn new() -> CentralProcessingUnit {
        CentralProcessingUnit {
            ime: false,
            regs: [0; NUM_REG],
            pc: 0,
            sp: 0,
            cycle_modification: 0,
            z_flag: 0,
            n_flag: 0,
            h_flag: 0,
            c_flag: 0,
            reenable_interrupts: false,
            change_ime_true: false,
            halting: false,
            cycle_count: 0,
            repeat: false,
            old_pc: 0,
            waiting: false,
            cycle_goal: 0,
            call_counter: 0,
            debug_var: 0,
            debug_action: false,
            command: 0,
        }
    }

    fn cgb_initialize_after_boot(&mut self) {
        self.regs[REG_A] = 0x11;
        self.z_flag = 1;
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = 0;
        self.regs[REG_B] = 0;
        self.regs[REG_C] = 0;
        self.regs[REG_D] = 0xFF;
        self.regs[REG_E] = 0x56;
        self.regs[REG_H] = 0;
        self.regs[REG_L] = 0xD;
        self.sp = 0xFFFE;
    }
    fn dmg_initialize_after_boot(&mut self) {
        self.regs[REG_A] = 1;
        self.z_flag = 0;
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = 0;
        self.regs[REG_B] = 0xFF;
        self.regs[REG_C] = 0x13;
        self.regs[REG_D] = 0;
        self.regs[REG_E] = 0xC1;
        self.regs[REG_H] = 0x84;
        self.regs[REG_L] = 0x3;
        self.sp = 0xFFFE;
    }
    fn add_set_flags_16(&mut self, val1: &u32, val2: &u32, z: bool, h: bool, c: bool) {
        if z {
            self.z_flag = if (val1 + val2) == 0 { 1 } else { 0 };
        }
        if h {
            self.h_flag = if (((val1 & 0xFFF) + (val2 & 0xFFF)) & (0x1000)) == 0x1000 {
                1
            } else {
                0
            };
        }
        if c {
            self.c_flag = if (val1 + val2) > CARRY_LIMIT_16 { 1 } else { 0 };
        }
    }
    #[inline]
    fn get_f(&self) -> u8 {
        (self.z_flag << 7) + (self.n_flag << 6) + (self.h_flag << 5) + (self.c_flag << 4)
    }
    #[inline]
    fn write_f(&mut self, val: u8) {
        self.z_flag = (val >> 7) & 1;
        self.n_flag = (val >> 6) & 1;
        self.h_flag = (val >> 5) & 1;
        self.c_flag = (val >> 4) & 1;
    }
}

impl GameBoyEmulator {
    pub fn cpu_initialize_after_boot(&mut self) {
        if self.cgb {
            self.cpu.cgb_initialize_after_boot();
        } else {
            self.cpu.dmg_initialize_after_boot();
        }
    }
    pub fn cpu_advance(&mut self) {
        if self.cpu.waiting {
            self.cpu.cycle_count += ADVANCE_CYCLES;
            if self.cpu.cycle_count == self.cpu.cycle_goal {
                self.cpu.waiting = false;
                self.cpu.cycle_count = 0;
            }
        } else {
            if self.cpu.change_ime_true {
                self.cpu.change_ime_true = false;
                self.cpu.ime = true;
            }
            if self.cpu.reenable_interrupts {
                self.cpu.reenable_interrupts = false;
                self.cpu.change_ime_true = true;
            }

            let viable_interrupts =
                self.get_memory(INT_FLAG_ADDR, SOURCE) & self.get_memory(INT_ENABLE_ADDR, SOURCE);

            if self.cpu.ime && viable_interrupts != 0 && !self.cpu.halting {
                let (mask, addr) = match viable_interrupts.trailing_zeros() {
                    0 => (0b11110, 0x40),
                    1 => (0b11101, 0x48),
                    2 => (0b11011, 0x50),
                    3 => (0b10111, 0x58),
                    4 => (0b01111, 0x60),
                    _ => {
                        panic!("Wow, how did you get here? This is the interrupt area where they are no interrupts.")
                    }
                };
                self.write_memory(
                    INT_FLAG_ADDR,
                    self.get_memory(INT_FLAG_ADDR, SOURCE) & mask,
                    SOURCE,
                );
                self.cpu.ime = false;
                let (high_pc, low_pc) = split_u16(self.cpu.pc);
                self.push_stack(high_pc, low_pc);
                self.cpu.pc = addr;
                self.cpu.waiting = true;
                self.cpu.cycle_count += ADVANCE_CYCLES;
                self.cpu.cycle_goal = INTERRUPT_DOTS;
            } else {
                self.cpu.command = self.get_memory(self.cpu.pc, SOURCE) as usize;

                let repeat_operation = if self.cpu.repeat {
                    self.cpu.repeat = false;
                    true
                } else {
                    false
                };
                FUNCTION_MAP[self.cpu.command](self, self.cpu.command as u8);
                self.cpu.cycle_goal = if self.cpu.cycle_modification != 0 {
                    let val = self.cpu.cycle_modification;
                    self.cpu.cycle_modification = 0;
                    val
                } else {
                    CYCLES_MAP[self.cpu.command]
                };
                if repeat_operation {
                    self.cpu.pc = self.cpu.old_pc;
                }
                self.cpu.cycle_count += ADVANCE_CYCLES;
                if self.cpu.cycle_count < self.cpu.cycle_goal {
                    self.cpu.waiting = true;
                } else {
                    self.cpu.cycle_count = 0;
                }
            }
        }
    }
    fn push_stack(&mut self, high_val: u8, low_val: u8) {
        self.cpu.sp = self.cpu.sp.wrapping_sub(1);
        self.write_memory(self.cpu.sp, high_val, SOURCE);
        self.cpu.sp = self.cpu.sp.wrapping_sub(1);
        self.write_memory(self.cpu.sp, low_val, SOURCE);
    }
    fn pop_stack(&mut self) -> [u8; 2] {
        let val1 = self.get_memory((self.cpu.sp) as usize, SOURCE);
        self.cpu.sp = self.cpu.sp.wrapping_add(1);
        let val2 = self.get_memory((self.cpu.sp) as usize, SOURCE);
        self.cpu.sp = self.cpu.sp.wrapping_add(1);
        [val1, val2]
    }
    fn fail(&mut self, command: u8) {
        panic!(
            "{}",
            format!("Unrecognized command {:X} not in function table!!", command)
        );
    }
    fn nop(&mut self, _command: u8) {
        self.cpu.pc += 1;
    }
    fn stop(&mut self, _command: u8) {
        let key1 = self.get_memory(KEY1_ADDR, SOURCE);
        let prepare = (key1 & 1) == 1;
        if prepare && self.cgb {
            self.double_speed = !self.double_speed;
            self.cpu.cycle_modification = 8200;
            let speed_bit = if self.double_speed { 1 } else { 0 };
            self.write_memory(KEY1_ADDR, speed_bit << 7, RequestSource::SPEC);
            self.cpu.pc += 1;
        }
    }
    fn ld_reg_16(&mut self, command: u8) {
        let low_byte = self.get_memory(self.cpu.pc + 1, SOURCE);
        let high_byte = self.get_memory(self.cpu.pc + 2, SOURCE);
        match command {
            0x01 => {
                //LD BC
                self.cpu.regs[REG_B] = high_byte;
                self.cpu.regs[REG_C] = low_byte;
            }
            0x11 => {
                //LD DE
                self.cpu.regs[REG_D] = high_byte;
                self.cpu.regs[REG_E] = low_byte;
            }
            0x21 => {
                //LD HL
                self.cpu.regs[REG_H] = high_byte;
                self.cpu.regs[REG_L] = low_byte;
            }
            0x31 => {
                //LD SP XX
                self.cpu.sp = combine_bytes(high_byte, low_byte);
            }
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at ld_reg_16!", command)
            ),
        };
        self.cpu.pc += 3;
    }
    fn ld_addr_a(&mut self, command: u8) {
        let adding_1 = self.get_memory(self.cpu.pc + 1, SOURCE);
        let adding_2 = self.get_memory(self.cpu.pc + 2, SOURCE);
        let addr = match command {
            0x02 => combine_bytes(self.cpu.regs[REG_B], self.cpu.regs[REG_C]), //LD (BC) A

            0x12 => combine_bytes(self.cpu.regs[REG_D], self.cpu.regs[REG_E]), //LD (DE) A

            0x22 => {
                //LD (HL+) A
                let mut hl: u16 = combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]);
                let addr = hl;
                hl = hl.wrapping_add(1);
                let (new_h, new_l) = split_u16(hl);
                self.cpu.regs[REG_L] = new_l;
                self.cpu.regs[REG_H] = new_h;
                addr
            }
            0x32 => {
                //LD (HL-) A
                let mut hl: u16 = combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]);
                let addr = hl;
                hl = hl.wrapping_sub(1);
                let (new_h, new_l) = split_u16(hl);
                self.cpu.regs[REG_L] = new_l;
                self.cpu.regs[REG_H] = new_h;
                addr
            }
            0xE0 => {
                //LD (FF00 + XX) A
                let adding = adding_1 as u16;
                self.cpu.pc += 1;
                0xFF00 + adding
            }
            0xE2 => {
                // LD (FF00 + C) A
                0xFF00 + self.cpu.regs[REG_C] as u16
            }
            0xEA => {
                //LD (XX) A
                self.cpu.pc += 2;
                combine_bytes(adding_2, adding_1)
            }
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at ld_reg_addr_a!", command)
            ),
        };
        self.write_memory(addr, self.cpu.regs[REG_A], SOURCE);
        self.cpu.pc += 1;
    }
    fn inc_reg_16(&mut self, command: u8) {
        if command == 0x33 {
            //INC SP
            self.cpu.sp = self.cpu.sp.wrapping_add(1);
        } else {
            let (r_low, r_high) = match command {
                0x03 => (REG_C, REG_B), //INC BC
                0x13 => (REG_E, REG_D), //INC DE
                0x23 => (REG_L, REG_H), //INC HL
                _ => panic!(
                    "{}",
                    format!("Unrecognized command {:X} at inc_reg_16!", command)
                ),
            };
            let mut comb = combine_bytes(self.cpu.regs[r_high], self.cpu.regs[r_low]);
            comb = comb.wrapping_add(1);
            let (comb_high, comb_low) = split_u16(comb);
            self.cpu.regs[r_high] = comb_high;
            self.cpu.regs[r_low] = comb_low;
        }
        self.cpu.pc += 1;
    }
    fn inc_reg_8(&mut self, command: u8) {
        let val = match command {
            0x04 => {
                //INC B
                self.cpu.regs[REG_B] = self.cpu.regs[REG_B].wrapping_add(1);
                self.cpu.regs[REG_B]
            }
            0x14 => {
                //INC D
                self.cpu.regs[REG_D] = self.cpu.regs[REG_D].wrapping_add(1);
                self.cpu.regs[REG_D]
            }
            0x24 => {
                //INC H
                self.cpu.regs[REG_H] = self.cpu.regs[REG_H].wrapping_add(1);
                self.cpu.regs[REG_H]
            }
            0x34 => {
                //INC (HL)
                let addr = combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]);
                let mut val = self.get_memory(addr, SOURCE);
                val = val.wrapping_add(1);
                self.write_memory(addr, val, SOURCE);
                val
            }
            0x0C => {
                //INC C
                self.cpu.regs[REG_C] = self.cpu.regs[REG_C].wrapping_add(1);
                self.cpu.regs[REG_C]
            }
            0x1C => {
                //INC E
                self.cpu.regs[REG_E] = self.cpu.regs[REG_E].wrapping_add(1);
                self.cpu.regs[REG_E]
            }
            0x2C => {
                //INC L
                self.cpu.regs[REG_L] = self.cpu.regs[REG_L].wrapping_add(1);
                self.cpu.regs[REG_L]
            }
            0x3C => {
                //INC A
                self.cpu.regs[REG_A] = self.cpu.regs[REG_A].wrapping_add(1);
                self.cpu.regs[REG_A]
            }
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at inc_reg_8!", command)
            ),
        };
        self.cpu.z_flag = if val == 0 { 1 } else { 0 };
        self.cpu.h_flag = if (val & 0xF) == 0 { 1 } else { 0 };
        self.cpu.n_flag = 0;
        self.cpu.pc += 1;
    }
    fn dec_reg_8(&mut self, command: u8) {
        let val = match command {
            0x05 => {
                //DEC B
                self.cpu.regs[REG_B] = self.cpu.regs[REG_B].wrapping_sub(1);
                self.cpu.regs[REG_B]
            }
            0x15 => {
                //DEC D
                self.cpu.regs[REG_D] = self.cpu.regs[REG_D].wrapping_sub(1);
                self.cpu.regs[REG_D]
            }
            0x25 => {
                //DEC H
                self.cpu.regs[REG_H] = self.cpu.regs[REG_H].wrapping_sub(1);
                self.cpu.regs[REG_H]
            }
            0x35 => {
                //DEC (HL)
                let addr: usize =
                    combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]) as usize;
                let mut val = self.get_memory(addr, SOURCE);
                val = val.wrapping_sub(1);
                self.write_memory(addr, val, SOURCE);
                val
            }
            0x0D => {
                //DEC C
                self.cpu.regs[REG_C] = self.cpu.regs[REG_C].wrapping_sub(1);
                self.cpu.regs[REG_C]
            }
            0x1D => {
                //DEC E
                self.cpu.regs[REG_E] = self.cpu.regs[REG_E].wrapping_sub(1);
                self.cpu.regs[REG_E]
            }
            0x2D => {
                //DEC L
                self.cpu.regs[REG_L] = self.cpu.regs[REG_L].wrapping_sub(1);
                self.cpu.regs[REG_L]
            }
            0x3D => {
                //DEC A
                self.cpu.regs[REG_A] = self.cpu.regs[REG_A].wrapping_sub(1);
                self.cpu.regs[REG_A]
            }
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at dec_reg_8!", command)
            ),
        };
        self.cpu.z_flag = if val == 0 { 1 } else { 0 };
        self.cpu.h_flag = if (val & 0xF) == 0xF { 1 } else { 0 };
        self.cpu.n_flag = 1;
        self.cpu.pc += 1;
    }
    fn ld_reg_8(&mut self, command: u8) {
        let to_load = self.get_memory((self.cpu.pc + 1) as usize, SOURCE);

        match command {
            0x06 => {
                //LD B XX
                self.cpu.regs[REG_B] = to_load;
            }
            0x16 => {
                //LD D XX
                self.cpu.regs[REG_D] = to_load;
            }
            0x26 => {
                //LD H XX
                self.cpu.regs[REG_H] = to_load;
            }
            0x36 => {
                //LD (HL) XX
                let addr = combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]);
                self.write_memory(addr, to_load, SOURCE);
            }
            0x0E => {
                //LD C
                self.cpu.regs[REG_C] = to_load;
            }
            0x1E => {
                //LD E
                self.cpu.regs[REG_E] = to_load;
            }
            0x2E => {
                //LD L
                self.cpu.regs[REG_L] = to_load;
            }
            0x3E => {
                //LD A
                self.cpu.regs[REG_A] = to_load;
            }
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at ld_reg_8!", command)
            ),
        };
        self.cpu.pc += 2;
    }
    fn rlca(&mut self, _command: u8) {
        let bit = self.cpu.regs[REG_A] >> 7;
        self.cpu.regs[REG_A] <<= 1;
        self.cpu.regs[REG_A] += bit;
        self.cpu.z_flag = 0;
        self.cpu.n_flag = 0;
        self.cpu.h_flag = 0;
        self.cpu.c_flag = bit;
        self.cpu.pc += 1;
    }
    fn rla(&mut self, _command: u8) {
        let last_bit = self.cpu.regs[REG_A] >> 7;
        self.cpu.regs[REG_A] <<= 1;
        self.cpu.regs[REG_A] += self.cpu.c_flag;
        self.cpu.z_flag = 0;
        self.cpu.n_flag = 0;
        self.cpu.h_flag = 0;
        self.cpu.c_flag = last_bit;
        self.cpu.pc += 1;
    }
    fn daa(&mut self, _command: u8) {
        let mut correction = 0;
        if self.cpu.h_flag == 1 || ((self.cpu.n_flag == 0) && ((self.cpu.regs[REG_A] & 0xF) > 0x9))
        {
            correction |= 0x6;
        }
        if self.cpu.c_flag == 1 || ((self.cpu.n_flag == 0) && (self.cpu.regs[REG_A] > 0x99)) {
            correction |= 0x60;
            self.cpu.c_flag = 1;
        }
        self.cpu.regs[REG_A] = if self.cpu.n_flag == 0 {
            self.cpu.regs[REG_A].wrapping_add(correction)
        } else {
            self.cpu.regs[REG_A].wrapping_sub(correction)
        };
        self.cpu.z_flag = if self.cpu.regs[REG_A] == 0 { 1 } else { 0 };
        self.cpu.h_flag = 0;
        self.cpu.pc += 1;
    }
    fn scf(&mut self, _command: u8) {
        self.cpu.n_flag = 0;
        self.cpu.h_flag = 0;
        self.cpu.c_flag = 1;
        self.cpu.pc += 1;
    }
    fn ld_addr_sp(&mut self, _command: u8) {
        let addr_low = self.get_memory(self.cpu.pc + 1, SOURCE);
        let addr_high = self.get_memory(self.cpu.pc + 2, SOURCE);
        let addr = combine_bytes(addr_high, addr_low);
        let (high_sp, low_sp) = split_u16(self.cpu.sp);
        self.write_memory(addr, low_sp, SOURCE);
        self.write_memory(addr + 1, high_sp, SOURCE);
        self.cpu.pc += 3;
    }
    fn jr(&mut self, command: u8) {
        let add = self.get_memory(self.cpu.pc + 1, SOURCE);
        self.cpu.pc += 2;
        let condition = match command {
            0x18 => true,                 //JR XX
            0x20 => self.cpu.z_flag == 0, //JR NZ XX
            0x28 => self.cpu.z_flag == 1, //JR Z XX
            0x30 => self.cpu.c_flag == 0, //JR NC XX
            0x38 => self.cpu.c_flag == 1, //JR C XX
            _ => panic!("{}", format!("Unrecognized command {:X} at jr!", command)),
        };
        if condition {
            self.cpu.cycle_modification = 12;
            self.cpu.pc = (self.cpu.pc as i32 + (add as i8) as i32) as u16;
            //self.cpu.pc = self.cpu.pc.wrapping_add((add as i8) as u16);
        }
    }
    fn add_hl_reg_16(&mut self, command: u8) {
        let mut hl = combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]);
        let reg16 = match command {
            0x09 => combine_bytes(self.cpu.regs[REG_B], self.cpu.regs[REG_C]), //ADD HL, BC

            0x19 => combine_bytes(self.cpu.regs[REG_D], self.cpu.regs[REG_E]), //ADD HL, DE

            0x29 => combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]), //ADD HL, HL

            0x39 => self.cpu.sp, //ADD HL, SP
            _ => panic!("{}", format!("Unrecognized command {:X} at jr!", command)),
        };
        self.cpu
            .add_set_flags_16(&(hl as u32), &(reg16 as u32), false, true, true);
        hl = hl.wrapping_add(reg16);
        let (h_new, l_new) = split_u16(hl);
        self.cpu.regs[REG_H] = h_new;
        self.cpu.regs[REG_L] = l_new;
        self.cpu.n_flag = 0;
        self.cpu.pc += 1;
    }
    fn ld_a_addr(&mut self, command: u8) {
        let addr_low = self.get_memory(self.cpu.pc + 1, SOURCE);
        let addr_high = self.get_memory(self.cpu.pc + 2, SOURCE);
        let addr = match command {
            0x0A => combine_bytes(self.cpu.regs[REG_B], self.cpu.regs[REG_C]), //LD A (BC)

            0x1A => combine_bytes(self.cpu.regs[REG_D], self.cpu.regs[REG_E]), //LD A (DE)

            0x2A => {
                //LD A (HL +)
                let hl_old: u16 = combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]);
                let hl_new = hl_old.wrapping_add(1);
                let (h_new, l_new) = split_u16(hl_new);
                self.cpu.regs[REG_L] = l_new;
                self.cpu.regs[REG_H] = h_new;
                hl_old
            }
            0x3A => {
                //LD A (HL -)
                let hl_old: u16 = combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]);
                let hl_new = hl_old.wrapping_sub(1);
                let (h_new, l_new) = split_u16(hl_new);
                self.cpu.regs[REG_L] = l_new;
                self.cpu.regs[REG_H] = h_new;
                hl_old
            }
            0xF0 => {
                //LD A (FF00 + XX)
                self.cpu.pc += 1;
                0xFF00 + (addr_low as u16)
            }
            0xF2 => 0xFF00 + (self.cpu.regs[REG_C] as u16), //LD A (FF00 + C)

            0xFA => {
                //LD A (XX)
                let addr = combine_bytes(addr_high, addr_low);
                self.cpu.pc += 2;
                addr
            }
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at ld_a_reg_addr!", command)
            ),
        };
        let new_val = self.get_memory(addr, SOURCE);
        self.cpu.regs[REG_A] = new_val;
        self.cpu.pc += 1;
    }
    fn dec_reg_16(&mut self, command: u8) {
        if command == 0x3B {
            //DEC SP
            self.cpu.sp = self.cpu.sp.wrapping_sub(1);
        } else {
            let (r_low, r_high) = match command {
                0x0B => (REG_C, REG_B), //DEC BC
                0x1B => (REG_E, REG_D), //DEC DE
                0x2B => (REG_L, REG_H), //DEC HL
                _ => panic!(
                    "{}",
                    format!("Unrecognized command {:X} at dec_reg_16!", command)
                ),
            };
            let mut comb = combine_bytes(self.cpu.regs[r_high], self.cpu.regs[r_low]);
            comb = comb.wrapping_sub(1);
            let (comb_high, comb_low) = split_u16(comb);
            self.cpu.regs[r_high] = comb_high;
            self.cpu.regs[r_low] = comb_low;
        }
        self.cpu.pc += 1;
    }
    fn rrca(&mut self, _command: u8) {
        let bit = self.cpu.regs[REG_A] & 1;
        self.cpu.regs[REG_A] >>= 1;
        self.cpu.regs[REG_A] += bit << 7;

        self.cpu.z_flag = 0;
        self.cpu.n_flag = 0;
        self.cpu.h_flag = 0;
        self.cpu.c_flag = bit;
        self.cpu.pc += 1;
    }
    fn rra(&mut self, _command: u8) {
        let bit = self.cpu.regs[REG_A] & 1;
        self.cpu.regs[REG_A] >>= 1;
        self.cpu.regs[REG_A] += self.cpu.c_flag << 7;
        self.cpu.z_flag = 0;
        self.cpu.n_flag = 0;
        self.cpu.h_flag = 0;
        self.cpu.c_flag = bit;
        self.cpu.pc += 1;
    }
    fn cpl(&mut self, _command: u8) {
        self.cpu.n_flag = 1;
        self.cpu.h_flag = 1;
        self.cpu.regs[REG_A] = !self.cpu.regs[REG_A];
        self.cpu.pc += 1;
    }
    fn ccf(&mut self, _command: u8) {
        self.cpu.c_flag = if self.cpu.c_flag == 1 { 0 } else { 1 };
        self.cpu.n_flag = 0;
        self.cpu.h_flag = 0;
        self.cpu.pc += 1;
    }
    fn ld_reg_reg(&mut self, command: u8) {
        let (command_high, command_low) = split_byte(command);
        let reg_1 = match command_high {
            0x4 => {
                if command_low <= 0x7 {
                    REG_B
                } else {
                    REG_C
                }
            }
            0x5 => {
                if command_low <= 0x7 {
                    REG_D
                } else {
                    REG_E
                }
            }
            0x6 => {
                if command_low <= 0x7 {
                    REG_H
                } else {
                    REG_L
                }
            }
            0x7 => REG_A,
            _ => panic!(
                "{}",
                format!("Unrecognized subcommand {:X} at ld_reg_reg!", command_high)
            ),
        };
        let reg_2 = match command_low % 8 {
            0x0 => REG_B,
            0x1 => REG_C,
            0x2 => REG_D,
            0x3 => REG_E,
            0x4 => REG_H,
            0x5 => REG_L,
            0x7 => REG_A,
            _ => panic!(
                "{}",
                format!("Unrecognized subcommand {:X} at ld_reg_reg!", command_low)
            ),
        };
        self.cpu.regs[reg_1] = self.cpu.regs[reg_2];
        self.cpu.pc += 1;
    }
    fn ld_reg_hl_addr(&mut self, command: u8) {
        let (command_high, command_low) = split_byte(command);
        let addr = combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]);
        let reg = if command_low == 0x6 {
            match command_high {
                0x4 => REG_B,
                0x5 => REG_D,
                0x6 => REG_H,
                _ => panic!(
                    "{}",
                    format!(
                        "Unrecognized subcommand {:X} at ld_reg_hl_addr!",
                        command_high
                    )
                ),
            }
        } else {
            match command_high {
                0x4 => REG_C,
                0x5 => REG_E,
                0x6 => REG_L,
                0x7 => REG_A,
                _ => panic!(
                    "{}",
                    format!(
                        "Unrecognized subcommand {:X} at ld_reg_hl_addr!",
                        command_high
                    )
                ),
            }
        };
        let new_val = self.get_memory(addr, SOURCE);
        self.cpu.regs[reg] = new_val;
        self.cpu.pc += 1;
    }
    fn ld_hl_addr_reg(&mut self, command: u8) {
        let addr = combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]);
        let reg = match command {
            0x70 => REG_B,
            0x71 => REG_C,
            0x72 => REG_D,
            0x73 => REG_E,
            0x74 => REG_H,
            0x75 => REG_L,
            0x77 => REG_A,
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at ld_hl_addr_reg!", command)
            ),
        };
        self.cpu.pc += 1;
        self.write_memory(addr, self.cpu.regs[reg], SOURCE);
    }
    fn halt(&mut self, _command: u8) {
        self.cpu.halting = true;
        if self.get_memory(INT_FLAG_ADDR, SOURCE) & self.get_memory(INT_ENABLE_ADDR, SOURCE) != 0 {
            self.cpu.pc += 1;
            self.cpu.halting = false;
            if !self.cpu.ime {
                self.cpu.repeat = true;
            }
        }
    }
    fn arthimetic_a(&mut self, command: u8) {
        let additional_val = self.get_memory((self.cpu.pc + 1) as usize, SOURCE);
        let (command_high, command_low) = split_byte(command);
        let op_val = if command_high <= 0xB {
            match command_low % 8 {
                0x0 => self.cpu.regs[REG_B],
                0x1 => self.cpu.regs[REG_C],
                0x2 => self.cpu.regs[REG_D],
                0x3 => self.cpu.regs[REG_E],
                0x4 => self.cpu.regs[REG_H],
                0x5 => self.cpu.regs[REG_L],
                0x6 => {
                    let addr = combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]) as usize;
                    self.get_memory(addr, SOURCE)
                }
                0x7 => self.cpu.regs[REG_A],
                _ => panic!(
                    "{}",
                    format!(
                        "Unrecognized subcommand {:X} at arthimetic!",
                        command_low % 8
                    )
                ),
            }
        } else {
            self.cpu.pc += 1;
            additional_val
        };
        let command_high_mod = (command_high - 0x8) % 4;
        let (additional, second_half) = if command_low >= 0x8 {
            if command_high_mod != 0x3 {
                (self.cpu.c_flag, true)
            } else {
                (0, true)
            }
        } else {
            (0, false)
        };
        let zero_val = if !second_half && command_high_mod == 0x3 {
            // OR A
            self.cpu.regs[REG_A] |= op_val;
            self.cpu.n_flag = 0;
            self.cpu.h_flag = 0;
            self.cpu.c_flag = 0;
            self.cpu.regs[REG_A]
        } else {
            match command_high_mod {
                0x0 => {
                    //ADD/ADC A
                    self.cpu.h_flag =
                        if ((self.cpu.regs[REG_A] & 0xF) + (op_val & 0xF) + additional) & 0x10
                            == 0x10
                        {
                            1
                        } else {
                            0
                        };
                    self.cpu.c_flag =
                        if self.cpu.regs[REG_A] as u16 + op_val as u16 + additional as u16
                            > CARRY_LIMIT_8
                        {
                            1
                        } else {
                            0
                        };
                    self.cpu.regs[REG_A] = self.cpu.regs[REG_A]
                        .wrapping_add(op_val)
                        .wrapping_add(additional);
                    self.cpu.n_flag = 0;
                    self.cpu.regs[REG_A]
                }
                0x1 | 0x3 => {
                    //SUB/SBC/CP
                    self.cpu.h_flag = if ((self.cpu.regs[REG_A] & 0xF)
                        .wrapping_sub(op_val & 0xF)
                        .wrapping_sub(additional))
                        & 0x10
                        == 0x10
                    {
                        1
                    } else {
                        0
                    };
                    self.cpu.c_flag =
                        if (op_val as u16 + additional as u16) > self.cpu.regs[REG_A] as u16 {
                            1
                        } else {
                            0
                        };

                    let new_a = self.cpu.regs[REG_A]
                        .wrapping_sub(op_val)
                        .wrapping_sub(additional);
                    if command_high_mod == 0x1 {
                        self.cpu.regs[REG_A] = new_a;
                    }
                    self.cpu.n_flag = 1;
                    new_a
                }
                0x2 => {
                    if second_half {
                        //XOR A
                        self.cpu.regs[REG_A] ^= op_val;
                        self.cpu.h_flag = 0;
                    } else {
                        //AND A
                        self.cpu.regs[REG_A] &= op_val;
                        self.cpu.h_flag = 1;
                    }
                    self.cpu.n_flag = 0;
                    self.cpu.c_flag = 0;
                    self.cpu.regs[REG_A]
                }
                _ => panic!(
                    "{}",
                    format!(
                        "Unrecognized subcommand {:X} at arthimetic!",
                        command_high_mod
                    )
                ),
            }
        };
        self.cpu.z_flag = if zero_val == 0 { 1 } else { 0 };
        self.cpu.pc += 1;
    }
    fn ret(&mut self, command: u8) {
        self.cpu.pc += 1;
        let condition = match command {
            0xC0 => {
                //RET NZ
                if self.cpu.z_flag == 0 {
                    self.cpu.cycle_modification = 20;
                    true
                } else {
                    false
                }
            }
            0xD0 => {
                //RET NC
                if self.cpu.c_flag == 0 {
                    self.cpu.cycle_modification = 20;
                    true
                } else {
                    false
                }
            }
            0xC8 => {
                //RET Z
                if self.cpu.z_flag == 1 {
                    self.cpu.cycle_modification = 20;
                    true
                } else {
                    false
                }
            }
            0xD8 => {
                //RET C
                if self.cpu.c_flag == 1 {
                    self.cpu.cycle_modification = 20;
                    true
                } else {
                    false
                }
            }
            0xC9 => {
                //RET
                true
            }
            0xD9 => {
                //RETI

                self.cpu.ime = true;
                true
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at ret!", command)),
        };
        if condition {
            let [addr_low, addr_high] = self.pop_stack();
            let addr = combine_bytes(addr_high, addr_low);
            self.cpu.pc = addr;
            self.cpu.call_counter -= 1;
        }
    }
    fn pop(&mut self, command: u8) {
        let [first_val, second_val] = self.pop_stack();
        self.cpu.pc += 1;
        match command {
            0xC1 => {
                self.cpu.regs[REG_C] = first_val;
                self.cpu.regs[REG_B] = second_val;
            }
            0xD1 => {
                self.cpu.regs[REG_E] = first_val;
                self.cpu.regs[REG_D] = second_val;
            }
            0xE1 => {
                self.cpu.regs[REG_L] = first_val;
                self.cpu.regs[REG_H] = second_val;
            }
            0xF1 => {
                self.cpu.write_f(first_val);
                self.cpu.regs[REG_A] = second_val;
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at pop!", command)),
        }
    }
    fn push(&mut self, command: u8) {
        self.cpu.pc += 1;
        let (high_val, low_val) = match command {
            0xC5 => (self.cpu.regs[REG_B], self.cpu.regs[REG_C]),
            0xD5 => (self.cpu.regs[REG_D], self.cpu.regs[REG_E]),
            0xE5 => (self.cpu.regs[REG_H], self.cpu.regs[REG_L]),
            0xF5 => (self.cpu.regs[REG_A], self.cpu.get_f()),
            _ => panic!("{}", format!("Unrecognized command {:X} at push!", command)),
        };
        self.push_stack(high_val, low_val);
    }
    fn call(&mut self, command: u8) {
        let addr_low = self.get_memory(self.cpu.pc + 1, SOURCE);
        let addr_high = self.get_memory(self.cpu.pc + 2, SOURCE);
        let addr = combine_bytes(addr_high, addr_low);
        self.cpu.pc += 3;
        let (pc_high, pc_low) = split_u16(self.cpu.pc);
        match command {
            0xC4 => {
                //CALL NZ
                if self.cpu.z_flag == 0 {
                    self.push_stack(pc_high, pc_low);
                    self.cpu.pc = addr;
                    self.cpu.cycle_modification = 24;
                    self.cpu.call_counter += 1;
                }
            }
            0xD4 => {
                //CALL NC
                if self.cpu.c_flag == 0 {
                    self.push_stack(pc_high, pc_low);
                    self.cpu.pc = addr;
                    self.cpu.cycle_modification = 24;
                    self.cpu.call_counter += 1;
                }
            }
            0xCC => {
                //CALL Z
                if self.cpu.z_flag == 1 {
                    self.push_stack(pc_high, pc_low);
                    self.cpu.pc = addr;
                    self.cpu.cycle_modification = 24;
                    self.cpu.call_counter += 1;
                }
            }
            0xDC => {
                //CALL C
                if self.cpu.c_flag == 1 {
                    self.push_stack(pc_high, pc_low);
                    self.cpu.pc = addr;
                    self.cpu.cycle_modification = 24;
                    self.cpu.call_counter += 1;
                }
            }
            0xCD => {
                //CALL
                self.push_stack(pc_high, pc_low);
                self.cpu.pc = addr;
                self.cpu.cycle_modification = 24;
                self.cpu.call_counter += 1;
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at call!", command)),
        }
    }
    fn rst(&mut self, command: u8) {
        let (high_command, low_command) = split_byte(command);
        self.cpu.pc += 1;
        let (high_pc, low_pc) = split_u16(self.cpu.pc);
        self.push_stack(high_pc, low_pc);
        self.cpu.pc = if low_command == 0xF {
            16 * (high_command as u16 - 0xC) + 8
        } else {
            16 * (high_command as u16 - 0xC)
        };
        self.cpu.call_counter += 1;
    }
    fn jp(&mut self, command: u8) {
        let low_byte = self.get_memory(self.cpu.pc + 1, SOURCE);
        let high_byte = self.get_memory(self.cpu.pc + 2, SOURCE);
        let addr = combine_bytes(high_byte, low_byte);
        let condition = match command {
            0xC2 => self.cpu.z_flag == 0, //JP NZ
            0xD2 => self.cpu.c_flag == 0, //JP NC
            0xC3 => true,                 //JP
            0xCA => self.cpu.z_flag == 1, //JP Z
            0xDA => self.cpu.c_flag == 1, //JP C
            _ => panic!("{}", format!("Unrecognized command {:X} at jp!", command)),
        };
        self.cpu.pc = if condition {
            self.cpu.cycle_modification = 16;
            addr
        } else {
            self.cpu.pc + 3
        };
    }
    fn jp_hl(&mut self, _command: u8) {
        self.cpu.pc = combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]);
    }
    fn add_sp_i8(&mut self, _command: u8) {
        let val = self.get_memory(self.cpu.pc + 1, SOURCE) as i8;
        let new_sp = self.cpu.sp.wrapping_add(val as u16);
        self.cpu.h_flag = if val >= 0 {
            if (self.cpu.sp & 0xF) as i8 + (val & 0xF) > 0xF {
                1
            } else {
                0
            }
        } else if new_sp & 0xF <= self.cpu.sp & 0xF {
            1
        } else {
            0
        };

        self.cpu.c_flag = if val >= 0 {
            if (self.cpu.sp & 0xFF) as i16 + val as i16 > 0xFF {
                1
            } else {
                0
            }
        } else if new_sp & 0xFF <= (self.cpu.sp & 0xFF) {
            1
        } else {
            0
        };
        self.cpu.sp = new_sp;

        self.cpu.z_flag = 0;
        self.cpu.n_flag = 0;
        self.cpu.pc += 2;
    }
    fn ld_hl_sp_i8(&mut self, _command: u8) {
        let val = self.get_memory(self.cpu.pc + 1, SOURCE) as i8;
        let new_hl = self.cpu.sp.wrapping_add(val as u16);
        self.cpu.h_flag = if val >= 0 {
            if (self.cpu.sp & 0xF) as i8 + (val & 0xF) > 0xF {
                1
            } else {
                0
            }
        } else if new_hl & 0xF <= self.cpu.sp & 0xF {
            1
        } else {
            0
        };

        self.cpu.c_flag = if val >= 0 {
            if (self.cpu.sp & 0xFF) as i16 + val as i16 > 0xFF {
                1
            } else {
                0
            }
        } else if new_hl & 0xFF <= (self.cpu.sp & 0xFF) {
            1
        } else {
            0
        };
        let (hl_val_high, hl_val_low) = split_u16(new_hl);
        self.cpu.regs[REG_H] = hl_val_high;
        self.cpu.regs[REG_L] = hl_val_low;
        self.cpu.z_flag = 0;
        self.cpu.n_flag = 0;
        self.cpu.pc += 2;
    }
    fn ld_sp_hl(&mut self, _command: u8) {
        self.cpu.sp = combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]);
        self.cpu.pc += 1;
    }
    fn ei(&mut self, _command: u8) {
        self.cpu.reenable_interrupts = true;
        self.cpu.pc += 1;
    }
    fn di(&mut self, _command: u8) {
        self.cpu.ime = false;
        self.cpu.pc += 1;
    }
    fn cb(&mut self, _command: u8) {
        let mut addr_val_ref = 0;
        let mut mem = false;
        let cb_command = self.get_memory(self.cpu.pc + 1, SOURCE);
        let (cb_command_high, cb_command_low) = split_byte(cb_command);
        let cb_command_low_second_half = cb_command_low >= 0x8;
        let bit_num = if cb_command_low_second_half {
            (cb_command_high % 4) * 2 + 1
        } else {
            (cb_command_high % 4) * 2
        };

        let reg = match cb_command_low % 8 {
            0x0 => &mut self.cpu.regs[REG_B],
            0x1 => &mut self.cpu.regs[REG_C],
            0x2 => &mut self.cpu.regs[REG_D],
            0x3 => &mut self.cpu.regs[REG_E],
            0x4 => &mut self.cpu.regs[REG_H],
            0x5 => &mut self.cpu.regs[REG_L],
            0x6 => {
                addr_val_ref = self.get_memory(
                    combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]),
                    SOURCE,
                );
                mem = true;
                self.cpu.cycle_modification = 16;
                &mut addr_val_ref
            }

            0x7 => &mut self.cpu.regs[REG_A],
            _ => panic!(
                "{}",
                format!("Unrecognized subcommand {:X} at CB!", cb_command)
            ),
        };
        match cb_command_high {
            0x0 => {
                let moved_bit = if cb_command_low_second_half {
                    //rrc
                    let bit_0 = *reg & 1;
                    *reg >>= 1;
                    *reg += bit_0 << 7;
                    bit_0
                } else {
                    //rlc
                    let bit_7 = (*reg >> 7) & 1;
                    *reg <<= 1;
                    *reg += bit_7;
                    bit_7
                };

                self.cpu.c_flag = moved_bit;
                self.cpu.h_flag = 0;
                self.cpu.n_flag = 0;
                self.cpu.z_flag = if *reg == 0 { 1 } else { 0 };
            }
            1 => {
                let moved_bit = if cb_command_low_second_half {
                    //rr
                    let bit_0 = *reg & 1;
                    *reg >>= 1;
                    *reg += self.cpu.c_flag << 7;
                    bit_0
                } else {
                    //rl
                    let bit_7 = (*reg >> 7) & 1;
                    *reg <<= 1;
                    *reg += self.cpu.c_flag;
                    bit_7
                };

                self.cpu.c_flag = moved_bit;
                self.cpu.h_flag = 0;
                self.cpu.n_flag = 0;
                self.cpu.z_flag = if *reg == 0 { 1 } else { 0 };
            }
            2 => {
                let bit_7 = (*reg >> 7) & 1;
                let moved_bit = if cb_command_low_second_half {
                    //sra

                    let bit_0 = *reg & 1;
                    *reg >>= 1;
                    *reg += bit_7 << 7;
                    bit_0
                } else {
                    //sla
                    *reg <<= 1;
                    bit_7
                };
                self.cpu.c_flag = moved_bit;
                self.cpu.h_flag = 0;
                self.cpu.n_flag = 0;
                self.cpu.z_flag = if *reg == 0 { 1 } else { 0 };
            }
            0x3 => {
                if cb_command_low_second_half {
                    //srl
                    let bit_0 = *reg & 1;
                    self.cpu.c_flag = bit_0;
                    *reg >>= 1;
                } else {
                    //swap
                    let (high_nib, low_nib) = split_byte(*reg);
                    *reg = (low_nib << 4) + high_nib;
                    self.cpu.c_flag = 0;
                }
                self.cpu.h_flag = 0;
                self.cpu.n_flag = 0;
                self.cpu.z_flag = if *reg == 0 { 1 } else { 0 };
            }
            4..=7 => {
                // bit
                if mem {
                    self.cpu.cycle_modification = 12;
                }
                self.cpu.z_flag = 1 - ((*reg >> bit_num) & 1);
                self.cpu.n_flag = 0;
                self.cpu.h_flag = 1;
                mem = false;
            }
            0x8..=0xB => {
                //res
                *reg &= 255 - 2u8.pow(bit_num as u32);
            }
            0xC..=0xF => {
                //set
                *reg |= 1 << bit_num;
            }
            _ => panic!(
                "{}",
                format!("Unrecognized subcommand {:X} at CB!", cb_command)
            ),
        };
        if mem {
            self.write_memory(
                combine_bytes(self.cpu.regs[REG_H], self.cpu.regs[REG_L]),
                addr_val_ref,
                SOURCE,
            );
        }
        self.cpu.pc += 2;
    }
}

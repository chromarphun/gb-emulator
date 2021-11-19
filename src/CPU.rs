use std::convert::TryInto;

const REG_A: usize = 0;
const REG_B: usize = 1;
const REG_C: usize = 2;
const REG_D: usize = 3;
const REG_E: usize = 4;
const REG_H: usize = 5;
const REG_L: usize = 6;
const CARRY_LIMIT: u16 = 255;

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

fn get_function_map() -> [fn(&mut CentralProcessingUnit) -> String; 256] {

    [
        //0x00
        CentralProcessingUnit::nop, CentralProcessingUnit::ld_reg_16, CentralProcessingUnit::ld_addr_a, CentralProcessingUnit::inc_reg_16,
        CentralProcessingUnit::inc_reg_8, CentralProcessingUnit::dec_reg_8, CentralProcessingUnit::ld_reg_8, CentralProcessingUnit::rot_a_left_carry,
        CentralProcessingUnit::ld_addr_sp, CentralProcessingUnit::add_hl_reg_16, CentralProcessingUnit::ld_a_addr, CentralProcessingUnit::dec_reg_16,
        CentralProcessingUnit::inc_reg_8, CentralProcessingUnit::dec_reg_8, CentralProcessingUnit::ld_reg_8, CentralProcessingUnit::rot_a_right_carry,
        //0x10
        CentralProcessingUnit::stop, CentralProcessingUnit::ld_reg_16, CentralProcessingUnit::ld_addr_a, CentralProcessingUnit::inc_reg_16,
        CentralProcessingUnit::inc_reg_8, CentralProcessingUnit::dec_reg_8, CentralProcessingUnit::ld_reg_8, CentralProcessingUnit::rot_a_left,
        CentralProcessingUnit::jr, CentralProcessingUnit::add_hl_reg_16, CentralProcessingUnit::ld_a_addr, CentralProcessingUnit::dec_reg_16,
        CentralProcessingUnit::inc_reg_8, CentralProcessingUnit::dec_reg_8, CentralProcessingUnit::ld_reg_8, CentralProcessingUnit::rot_a_right,
        //0x20
        CentralProcessingUnit::jr, CentralProcessingUnit::ld_reg_16, CentralProcessingUnit::ld_addr_a, CentralProcessingUnit::inc_reg_16,
        CentralProcessingUnit::inc_reg_8, CentralProcessingUnit::dec_reg_8, CentralProcessingUnit::ld_reg_8, CentralProcessingUnit::daa,
        CentralProcessingUnit::jr, CentralProcessingUnit::add_hl_reg_16, CentralProcessingUnit::ld_a_addr, CentralProcessingUnit::dec_reg_16,
        CentralProcessingUnit::inc_reg_8, CentralProcessingUnit::dec_reg_8, CentralProcessingUnit::ld_reg_8, CentralProcessingUnit::cpl,
        //0x30
        CentralProcessingUnit::jr, CentralProcessingUnit::ld_reg_16, CentralProcessingUnit::ld_addr_a, CentralProcessingUnit::inc_reg_16,
        CentralProcessingUnit::inc_reg_8, CentralProcessingUnit::dec_reg_8, CentralProcessingUnit::ld_reg_8, CentralProcessingUnit::scf,
        CentralProcessingUnit::jr, CentralProcessingUnit::add_hl_reg_16, CentralProcessingUnit::ld_a_addr, CentralProcessingUnit::dec_reg_16,
        CentralProcessingUnit::inc_reg_8, CentralProcessingUnit::dec_reg_8, CentralProcessingUnit::ld_reg_8, CentralProcessingUnit::ccf,
        //0x40
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, 
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_hl_addr, CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, 
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_hl_addr, CentralProcessingUnit::ld_reg_reg,
        //0x50
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, 
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_hl_addr, CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, 
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_hl_addr, CentralProcessingUnit::ld_reg_reg,
        //0x60
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, 
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_hl_addr, CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, 
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_hl_addr, CentralProcessingUnit::ld_reg_reg,
        //0x70
        CentralProcessingUnit::ld_hl_addr_reg, CentralProcessingUnit::ld_hl_addr_reg, CentralProcessingUnit::ld_hl_addr_reg, CentralProcessingUnit::ld_hl_addr_reg, 
        CentralProcessingUnit::ld_hl_addr_reg, CentralProcessingUnit::ld_hl_addr_reg, CentralProcessingUnit::halt, CentralProcessingUnit::ld_hl_addr_reg,
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, 
        CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_reg, CentralProcessingUnit::ld_reg_hl_addr, CentralProcessingUnit::ld_reg_reg,
        //0x80
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, 
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a,
        //0x90
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, 
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a,
        //0xA0
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, 
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a,
        //0xB0
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, 
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::arthimetic_a,
        //0xC0
        CentralProcessingUnit::ret, CentralProcessingUnit::pop, CentralProcessingUnit::jp, CentralProcessingUnit::jp, 
        CentralProcessingUnit::call, CentralProcessingUnit::push, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::rst,
        CentralProcessingUnit::ret, CentralProcessingUnit::ret, CentralProcessingUnit::jp, CentralProcessingUnit::nop,
        CentralProcessingUnit::call, CentralProcessingUnit::call, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::rst,
        //0xD0
        CentralProcessingUnit::ret, CentralProcessingUnit::pop, CentralProcessingUnit::jp, CentralProcessingUnit::fail, 
        CentralProcessingUnit::call, CentralProcessingUnit::push, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::rst,
        CentralProcessingUnit::ret, CentralProcessingUnit::ret, CentralProcessingUnit::jp, CentralProcessingUnit::fail,
        CentralProcessingUnit::call, CentralProcessingUnit::fail, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::rst,
        //0xE0
        CentralProcessingUnit::ld_addr_a, CentralProcessingUnit::pop, CentralProcessingUnit::ld_addr_a, CentralProcessingUnit::fail, 
        CentralProcessingUnit::fail, CentralProcessingUnit::push, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::rst,
        CentralProcessingUnit::add_sp_i8, CentralProcessingUnit::jp, CentralProcessingUnit::ld_addr_a, CentralProcessingUnit::fail,
        CentralProcessingUnit::fail, CentralProcessingUnit::fail, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::rst,
        //0xF0
        CentralProcessingUnit::ld_a_addr, CentralProcessingUnit::pop, CentralProcessingUnit::ld_a_addr, CentralProcessingUnit::di, 
        CentralProcessingUnit::fail, CentralProcessingUnit::push, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::rst,
        CentralProcessingUnit::ld_hl_sp_i8, CentralProcessingUnit::ld_sp_hl, CentralProcessingUnit::ld_a_addr, CentralProcessingUnit::ei,
        CentralProcessingUnit::fail, CentralProcessingUnit::fail, CentralProcessingUnit::arthimetic_a, CentralProcessingUnit::rst,
    ]
}

fn get_cycles_map() -> [u8; 256] {
    [
        //0x0
        04, 12, 08, 08, 04, 04, 08, 04, 20, 08, 08, 08, 04, 04, 08, 04,
        //0x1
        04, 12, 08, 08, 04, 04, 08, 04, 12, 08, 08, 08, 04, 04, 08, 04,        
        //0x2
        08, 12, 08, 08, 04, 04, 08, 04, 08, 08, 08, 08, 04, 04, 08, 04,        
        //0x3
        08, 12, 08, 08, 12, 12, 12, 04, 08, 08, 08, 08, 04, 04, 08, 04,
        //0x4
        04, 04, 04, 04, 04, 04, 08, 04, 04, 04, 04, 04, 04, 04, 08, 04,
        //0x5
        04, 04, 04, 04, 04, 04, 08, 04, 04, 04, 04, 04, 04, 04, 08, 04,        
        //0x6
        04, 04, 04, 04, 04, 04, 08, 04, 04, 04, 04, 04, 04, 04, 08, 04,
        //0x7
        08, 08, 08, 08, 08, 08, 04, 08, 04, 04, 04, 04, 04, 04, 08, 04,
        //0x8
        04, 04, 04, 04, 04, 04, 08, 04, 04 ,04 ,04 ,04 ,04 ,04 ,08, 04,
        //0x9
        04, 04, 04, 04, 04, 04, 08, 04, 04 ,04 ,04 ,04 ,04 ,04 ,08, 04,    
        //0xA
        04, 04, 04, 04, 04, 04, 08, 04, 04 ,04 ,04 ,04 ,04 ,04 ,08, 04,  
        //0xB
        04, 04, 04, 04, 04, 04, 08, 04, 04 ,04 ,04 ,04 ,04 ,04 ,08, 04,
        //0xC
        08, 12, 12, 16, 12, 16, 08, 16, 08, 16, 12, 04, 12, 24, 08, 16,
        //0xD
        08, 12, 12, 00, 12, 16, 08, 16, 08, 16, 12, 00, 12, 00, 08, 16,
        //0xE
        12, 12, 08, 00, 00, 16, 08, 16, 16, 04, 16, 00, 00, 00, 08, 16,
        //0xF
        12, 12, 08, 04, 00, 16, 08, 16, 12, 08, 16, 04, 00, 00, 08, 16
    ]
}
pub struct CentralProcessingUnit {
    regs: [u8; 7],
    reg_letter_map: [String; 7],
    pc: u16,
    sp: u16,
    cycle_modification: u8,
    z_flag: u8,
    n_flag: u8,
    h_flag: u8,
    c_flag: u8,
    interrupts_enable: bool,
    function_map: [fn(&mut CentralProcessingUnit) -> String; 256],
    cycles_map: [u8; 256],
    memory: [u8; 65536]
}

impl CentralProcessingUnit {

    pub fn new() -> CentralProcessingUnit {
        let regs = [0u8; 7];
        let reg_letter_map = [
            'A'.to_string(),
            'B'.to_string(),
            'C'.to_string(),
            'D'.to_string(),
            'E'.to_string(),
            'H'.to_string(),
            'L'.to_string(),
        ];
        let mut pc: u16 = 0x100;
        let mut sp: u16 = 0xFFFE;
        let mut interrupts_enable: bool = false;
        let mut z_flag: u8 = 0;
        let mut n_flag: u8 = 0;
        let mut h_flag: u8 = 0;
        let mut c_flag: u8 = 0;
        let function_map: [fn(&mut CentralProcessingUnit) -> String; 256] = get_function_map();
        let cycles_map: [u8; 256] = get_cycles_map();
        let mut cycle_modification:u8 = 0;
        let mut memory: [u8; 65536] = [0; 65536];
        CentralProcessingUnit {
            regs,
            reg_letter_map,
            pc,
            sp,
            cycle_modification,
            z_flag,
            n_flag,
            h_flag,
            c_flag,
            interrupts_enable,
            function_map,
            cycles_map,
            memory
        }
    }
    fn run(&mut self) {

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
    #[inline]
    fn push_stack(&mut self, high_val: u8, low_val: u8) {
        self.write_memory(self.sp - 2, [low_val, high_val, 0, 0], 2);
        self.sp -= 2;
    }
    fn pop_stack(&mut self) -> [u8; 2] {
        let [val1, val2, _, _] = self.get_memory(self.sp, 2);
        self.sp += 2;
        [val1, val2]
    }
    fn write_memory(&self, addr: u16, vals: [u8; 4], length: u8) {

    }
    fn get_memory(&self, addr: u16, length: u8) -> [u8; 4] {
        [0,0,0,0]
    }
    fn add_set_flags(&mut self, val1: &u16, val2: &u16, z: bool, h:bool, c:bool) {
        if z {
            self.z_flag = if (val1 + val2) == 0 {1} else {0};
        }
        if h {
            self.h_flag = if (((val1 & 0xF) + (val2 & 0xF)) & 0x10) == 0x10 {1} else {0};
        }
        if c {
            self.c_flag = if (val1 + val2) > CARRY_LIMIT {1} else {0};
        }
    }
    fn sub_set_flags(&mut self, val1: u16, val2: u16, z: bool, h:bool, c:bool) {
        if z {
            self.z_flag = if (val1 - val2) == 0 {1} else {0};
        }
        if h {
            self.h_flag = if (((val1 & 0xF) - (val2 & 0xF)) & 0x10) == 0x10 {1} else {0};
        }
        if c {
            self.c_flag = if val1 < val2 {1} else {0};
        }
    }
    fn fail(&mut self) -> String {
        let [command, _, _, _] = self.get_memory(self.pc, 1);
        panic!("{}", format!("Unrecognized command {:X} at ld_reg_16!", command));
        "FAIL".to_string()
    }  
    fn nop(&mut self) -> String {
        "NOP".to_string()
    }
    fn stop(&mut self) -> String {
        "STOP".to_string()
    }
    fn ld_reg_16(&mut self) -> String {
        let [command, high_byte, low_byte, _ ] =self.get_memory(self.pc, 3);
        let code= match command {
            0x01 => {
                self.regs[REG_B] = high_byte;
                self.regs[REG_C] = low_byte;
                format!("LD rBC {:X}", combine_bytes(high_byte, low_byte)) 
            },
            0x11 => {
                self.regs[REG_D] = high_byte;
                self.regs[REG_E] = low_byte;
                format!("LD rDE {:X}", combine_bytes(high_byte, low_byte)) 
            }
            0x21 => {
                self.regs[REG_H] = high_byte;
                self.regs[REG_L] = low_byte;
                format!("LD rHL {:X}", combine_bytes(high_byte, low_byte)) 
            }
            0x31 => {
                self.sp = (high_byte << 8) as u16 + low_byte as u16;
                format!("LD rSP {:X}", combine_bytes(high_byte, low_byte)) 
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at ld_reg_16!", command))
        };
        self.pc += 3;
        code
    }
    fn ld_addr_a(&mut self) -> String {
        let [command, adding_1, adding_2, _ ] =self.get_memory(self.pc, 3);
        let (code, addr) = match command {
            0x02 => {
                ("LD (rBC) rA".to_string(), combine_bytes(self.regs[REG_B], self.regs[REG_C]))
            },
            0x12 => {
                ("LD (rDE) rA".to_string(), combine_bytes(self.regs[REG_D], self.regs[REG_E]))
            }
            0x22 => {
                let hl: u16 = combine_bytes(self.regs[REG_H], self.regs[REG_H]);
                let addr = hl;
                hl.wrapping_add(1);
                let (new_h, new_l) = split_u16(hl);
                self.regs[REG_L] = new_h;
                self.regs[REG_H] = new_l;
                ("LD (rHL+) rA".to_string(), addr)
            }
            0x32 => {
                let hl: u16 = combine_bytes(self.regs[REG_H], self.regs[REG_H]);
                let addr = hl;
                hl.wrapping_sub(1);
                let (new_h, new_l) = split_u16(hl);
                self.regs[REG_L] = new_h;
                self.regs[REG_H] = new_l;
                ("LD (rHL+) rA".to_string(), addr)

            }
            0xE0 => {
                let adding = adding_1 as u16;
                let addr = 0xFF00 + adding;
                self.pc += 1;
                (format!("LD (FF00+{:X}) rA", adding), addr)
            }
            0xE2 => {
                let addr = 0xFF00 + self.regs[REG_C] as u16;
                ("LD (FF00 + rC) rA".to_string(), addr)
            }
            0xEA => {
                let addr = combine_bytes(adding_2, adding_1);
                self.pc += 2;
                (format!("LD ({:X}) A", addr), addr)
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at ld_reg_addr_a!", command))
        };
        self.write_memory(addr, [self.regs[REG_A], 0, 0, 0], 1);
        self.pc += 1;
        code
    }
    fn inc_reg_16(&mut self) -> String {
        let [command, _, _, _ ] =self.get_memory(self.pc, 1);
        let code = if command == 0x33 {
            self.sp.wrapping_add(1);
            "INC rSP".to_string()
        } else {
            let mut r_low: usize = 9;
            let mut r_high: usize = 9;
            let (r_low, r_high, code) = match command  {
                0x03 => {
                    (REG_C, REG_B, "INC rBC".to_string())
                }
                0x13 => {
                    (REG_E, REG_D, "INC rDE".to_string())
                }
                0x23 => {
                    (REG_L, REG_H, "INC rHL".to_string())
                }
                _ => panic!("{}", format!("Unrecognized command {:X} at inc_reg_16!", command))
            };
            if self.regs[r_low] != 0xFF {
                self.regs[r_low] += 1;
            } else {
                if self.regs[r_high] != 0xFF {
                    self.regs[r_high] += 1;
                    self.regs[r_low] = 0;
                } else {
                    self.regs[r_high] = 0;
                    self.regs[r_low] = 0
                }
            }
            code
        };
        self.pc += 1;
        code
    }
    fn inc_reg_8(&mut self) -> String {
        let [command, _, _, _ ] =self.get_memory(self.pc, 1);
        let (code , val) = match command {
            0x04 => {
                self.regs[REG_B] = self.regs[REG_B].wrapping_add(1);
                ("INC rB".to_string(), self.regs[REG_B])
            }
            0x14 => {
                self.regs[REG_D] = self.regs[REG_D].wrapping_add(1);
                ("INC rD".to_string(), self.regs[REG_D])
            }
            0x24 => {
                self.regs[REG_H] = self.regs[REG_H].wrapping_add(1);
                ("INC rH".to_string(), self.regs[REG_H])
            }
            0x34 => {
                let addr: u16 = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
                let [mut val, _, _, _ ] = self.get_memory(addr, 1);
                val = val.wrapping_add(1);
                self.write_memory(addr, [val, 0, 0, 0], 1);
                ("INC (rHL)".to_string(), val)
            }
            0x0C => {
                self.regs[REG_C].wrapping_add(1);
                ("INC rC".to_string(), self.regs[REG_C])
            }
            0x1C => {
                self.regs[REG_E].wrapping_add(1);
                ("INC rE".to_string(), self.regs[REG_E])
            }
            0x2C => {
                self.regs[REG_L].wrapping_add(1);
                ("INC rL".to_string(), self.regs[REG_L])
            }
            0x3C => {
                self.regs[REG_A].wrapping_add(1);
                ("INC rA".to_string(), self.regs[REG_A])
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at inc_reg_8!", command))
        };
        self.z_flag = if val == 0 {1} else {0};
        self.h_flag = if (val & 0xF) == 0 {1} else {0};
        self.n_flag = 0;
        self.pc += 1;
        code
    }
    fn dec_reg_8(&mut self) -> String {
        let [command, _, _, _ ] =self.get_memory(self.pc, 1);
        let val: u8 = 0;
        let (code , val) = match command {
            0x05 => {
                self.regs[REG_B] = self.regs[REG_B].wrapping_sub(1);
                ("DEC rB".to_string(), self.regs[REG_B])
            }
            0x15 => {
                self.regs[REG_D] = self.regs[REG_D].wrapping_sub(1);
                ("DEC rD".to_string(), self.regs[REG_D])
            }
            0x25 => {
                self.regs[REG_H] = self.regs[REG_H].wrapping_sub(1);
                ("DEC rH".to_string(), self.regs[REG_H])
            }
            0x35 => {
                let addr: u16 = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
                let [mut val, _, _, _ ] = self.get_memory(addr, 1);
                val = val.wrapping_sub(1);
                self.write_memory(addr, [val, 0, 0, 0], 1);
                ("DEC (rHL)".to_string(), val)
            }
            0x0D => {
                self.regs[REG_C].wrapping_sub(1);
                ("DEC rC".to_string(), self.regs[REG_C])
            }
            0x1D => {
                self.regs[REG_E].wrapping_sub(1);
                ("DEC rE".to_string(), self.regs[REG_E])
            }
            0x2D => {
                self.regs[REG_L].wrapping_sub(1);
                ("DEC rL".to_string(), self.regs[REG_L])
            }
            0x3D => {
                self.regs[REG_A].wrapping_sub(1);
                ("DEC rA".to_string(), self.regs[REG_A])
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at dec_reg_8!", command))
        };
        self.z_flag = if val == 0 {1} else {0};
        self.h_flag = if (val & 0xF) == 0xF {1} else {0};
        self.n_flag = 1;
        self.pc += 1;
        code
    }
    fn ld_reg_8(&mut self) -> String {
        let [command, to_load, _, _ ] =self.get_memory(self.pc, 2);
        let code = match command {
            0x06 => {
                self.regs[REG_B] = to_load;
                format!("LD rB {:X}", to_load)
            },
            0x16 => {
                self.regs[REG_D] = to_load;
                format!("LD rD {:X}", to_load)
            }
            0x26 => {
                self.regs[REG_H] = to_load;
                format!("LD rH {:X}", to_load)
            }
            0x36 => {
                let addr: u16 = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
                self.write_memory(addr, [to_load, 0, 0, 0], 1);
                format!("LD (rHL) {:X}", to_load)
            }
            0x0E => {
                self.regs[REG_C] = to_load;
                format!("LD rC {:X}", to_load)
            }
            0x1E => {
                self.regs[REG_E] = to_load;
                format!("LD rE {:X}", to_load)
            }
            0x2E => {
                self.regs[REG_L] = to_load;
                format!("LD rL {:X}", to_load)
            }
            0x3E => {
                self.regs[REG_A] = to_load;
                format!("LD rA {:X}", to_load)
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at ld_reg_8!", command))
        };
        self.pc += 2;
        code
    }
    fn rot_a_left(&mut self) -> String {
        let bit = self.regs[REG_A] >> 7;
        self.regs[REG_A] <<= 1;
        self.regs[REG_A] += bit;
        self.z_flag = 0;
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = bit;
        self.pc +=1 ;
        "RLCA".to_string()
    }
    fn rot_a_left_carry(&mut self) -> String {
        let last_bit = self.regs[REG_A] >> 7;
        self.regs[REG_A] <<= 1;
        self.regs[REG_A] += self.c_flag;
        self.z_flag = 0;
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = last_bit;
        self.pc += 1;
        "RLA".to_string()
    }
    fn daa(&mut self) -> String {
        if self.n_flag == 0 {  // after an addition, adjust if (half-)carry occurred or if result is out of bounds
            if self.c_flag == 1 || self.regs[REG_A] > 0x99 { self.regs[REG_A] += 0x60; self.c_flag = 1; }
            if self.h_flag == 1 || (self.regs[REG_A] & 0x0F) > 0x09 { self.regs[REG_A] += 0x6; }
        } else {  // after a subtraction, only adjust if (half-)carry occurred
            if self.c_flag == 1{ self.regs[REG_A] -= 0x60; }
            if self.h_flag == 1{ self.regs[REG_A] -= 0x6; }
        }
        self.pc+=1;
        if self.regs[REG_A] == 0 {
            self.z_flag = 1;
        }
        self.h_flag = 0;
        "DAA".to_string()
    }
    fn scf(&mut self) -> String {
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = 1;
        self.pc += 1;
        "SCF".to_string()
    }
    fn ld_addr_sp(&mut self) -> String {
        let [command, addr_low, addr_high, _ ] =self.get_memory(self.pc, 3);
        let addr = combine_bytes(addr_high, addr_low);
        let (high_sp, low_sp) = split_u16(self.sp);
        self.write_memory(addr, [low_sp, high_sp, 0, 0], 2);
        self.pc += 3;
        format!("LD ({:X}) rSP", addr)
    }
    fn jr(&mut self) -> String {
        let [command, add, _, _] = self.get_memory(self.pc, 2);
        let (code, condition) = match command {
            0x18 => {
                ("JR".to_string(), true)
            },
            0x20 => {
                ("JR NZ".to_string(), self.z_flag == 0)
            },
            0x28 => {
                ("JR Z".to_string(), self.z_flag == 1)
            },
            0x30 => {
                ("JR NC".to_string(), self.c_flag == 0)
            },
            0x38 => {
                ("JR C".to_string(), self.c_flag == 1)
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at jr!", command))
        };
        if condition {
            self.cycle_modification = 12;
            self.pc = self.pc.wrapping_add((add as i8) as u16);
        }
        code
    }
    fn add_hl_reg_16(&mut self) -> String {
        let [command, _, _, _] = self.get_memory(self.pc, 1);
        let mut hl = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
        let (reg16, code) = match command {
            0x09 => (combine_bytes(self.regs[REG_B], self.regs[REG_C]), "ADD HL, BC".to_string()),
            0x19 => (combine_bytes(self.regs[REG_D], self.regs[REG_E]), "ADD HL, DE".to_string()),
            0x29 => (combine_bytes(self.regs[REG_H], self.regs[REG_L]), "ADD HL, HL".to_string()),
            0x09 => (self.sp, "ADD HL, SP".to_string()),
            _ => panic!("{}", format!("Unrecognized command {:X} at jr!", command)) 
        };
        self.add_set_flags(&hl, &reg16, false, true, true);
        hl = hl.wrapping_add(reg16);
        self.n_flag = 0;
        self.pc += 1;
        code
    }
    fn ld_a_addr(&mut self) -> String {
        let [command, addr_low, addr_high, _] = self.get_memory(self.pc, 3);
        let (code, addr) = match command {
            0x0A => {
                ("LD A (BC)".to_string(), combine_bytes(self.regs[REG_B], self.regs[REG_C]))
            }
            0x1A => {
                ("LD A (DE)".to_string(), combine_bytes(self.regs[REG_D], self.regs[REG_E]))
            }
            0x2A => {
                let hl_old: u16 = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
                let hl_new = hl_old.wrapping_add(1);
                let (H_new, L_new) = split_u16(hl_new);
                self.regs[REG_L] = L_new;
                self.regs[REG_H] = H_new;
                ("LD A (HL +)".to_string(), hl_old)
            }
            0x3A => {
                let hl_old: u16 = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
                let hl_new = hl_old.wrapping_sub(1);
                let (H_new, L_new) = split_u16(hl_new);
                self.regs[REG_L] = L_new;
                self.regs[REG_H] = H_new;
                ("LD A (HL -)".to_string(), hl_old)
            }
            0xF0 => {
                self.pc += 1;
                (format!("LD A (FF00 + {:X})", addr_low), 0xFF00 + (addr_low as u16))
            }
            0xF2 => {
                (format!("LD A (FF00 + C)"), 0xFF00 + (self.regs[REG_C] as u16))
            }
            0xFA => {
                let addr = combine_bytes(addr_high, addr_low);
                self.pc += 2;
                (format!("({:X})", addr), addr)
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at ld_a_reg_addr!", command))
        };
        let [new_val, _, _, _] = self.get_memory(addr, 1);
        self.regs[REG_A] = new_val;
        self.pc += 1;
        code
    }
    fn dec_reg_16(&mut self) -> String {
        let [command, _, _, _ ] =self.get_memory(self.pc, 1);
        let code = if command == 0x3B {
            self.sp.wrapping_add(1);
            "DEC rSP".to_string()
        } else {
            let (r_low, r_high, code) = match command  {
                0x0B => {
                    (REG_C, REG_B, "DEC rBC".to_string())
                }
                0x1B => {
                    (REG_E, REG_D, "DEC rDE".to_string())
                }
                0x2B => {
                    (REG_L, REG_H, "DEC rHL".to_string())
                }
                _ => panic!("{}", format!("Unrecognized command {:X} at dec_reg_16!", command))
            };
            if self.regs[r_low] != 0x00 {
                self.regs[r_low] -= 1;
            } else {
                if self.regs[r_high] != 0x00 {
                    self.regs[r_high] -= 1;
                    self.regs[r_low] = 0xFF;
                } else {
                    self.regs[r_high] = 0xFF;
                    self.regs[r_low] = 0xFF;
                }
            }
            code
        };
        self.pc += 1;
        code
    }
    fn rot_a_right(&mut self) -> String {
        let bit = self.regs[REG_A] & 1;
        self.regs[REG_A] >>= 1;
        self.regs[REG_A] += bit << 7;

        self.z_flag = 0;
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = bit;
        self.pc +=1 ;
        "RRCA".to_string()
    }
    fn rot_a_right_carry(&mut self) -> String {
        let bit = self.regs[REG_A] & 1;
        self.regs[REG_A] >>= 1;
        self.regs[REG_A] += self.c_flag << 7;
        self.z_flag = 0;
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = bit;
        self.pc += 1;
        "RRA".to_string()
    }
    fn cpl(&mut self) -> String {
        self.n_flag = 0;
        self.h_flag = 0;
        self.regs[REG_A] = !self.regs[REG_A];
        "CPL".to_string()
    }
    fn ccf(&mut self) -> String {
        self.c_flag = if self.c_flag == 1 {0} else {1};
        self.n_flag = 0;
        self.h_flag = 0;
        "CCF".to_string()
    }
    fn ld_reg_reg(&mut self) -> String {
        let [command, _, _, _] = self.get_memory(self.pc, 1);
        let (command_high, command_low) = split_byte(command);
        let reg_1 = match command_high {
            0x4 => {
                if command_low <= 0x7 {REG_A} else {REG_B}
            }
            0x5 => {
                if command_low <= 0x7 {REG_C} else {REG_D}
            }
            0x6 => {
                if command_low <= 0x7 {REG_H} else {REG_L}
            }
            0x7 => REG_A,
            _ => panic!("{}", format!("Unrecognized subcommand {:X} at ld_reg_reg!", command_high))        
        };
        let reg_2 = match command_low % 8 {
            0x0 => REG_B,
            0x1 => REG_C,
            0x2 => REG_D,
            0x3 => REG_E,
            0x4 => REG_H,
            0x5 => REG_L,
            0x7 => REG_A,
            _ => panic!("{}", format!("Unrecognized subcommand {:X} at ld_reg_reg!", command_low))
        };
        self.regs[reg_1] = self.regs[reg_2];
        self.pc += 1;
        format!("LD r{}, r{}", self.reg_letter_map[reg_1], self.reg_letter_map[reg_2])
    }
    fn ld_reg_hl_addr(&mut self) -> String {
        let [command, _, _, _] = self.get_memory(self.pc, 1);
        let (command_high, command_low) = split_byte(command);
        let addr = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
        let reg = if command_low == 0x6 {
            match command_high {
                0x4 => REG_B,
                0x5 => REG_D,
                0x6 => REG_H,
                _ => panic!("{}", format!("Unrecognized subcommand {:X} at ld_reg_hl_addr!", command_high))
            }
        } else {
            match command_high {
                0x4 => REG_C,
                0x5 => REG_E,
                0x6 => REG_L,
                0x7 => REG_A,
                _ => panic!("{}", format!("Unrecognized subcommand {:X} at ld_reg_hl_addr!", command_high))
            }
        };
        let [new_val, _, _, _] = self.get_memory(addr, 1);
        self.regs[reg] = new_val;
        format!("LD r{}, (rHL)", self.reg_letter_map[reg])
    }
    fn ld_hl_addr_reg(&mut self) -> String {
        let [command, _, _, _] = self.get_memory(self.pc, 1);
        let addr = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
        let reg = match command {
            0x70 => REG_B,
            0x71 => REG_C,
            0x72 => REG_D,
            0x73 => REG_E,
            0x74 => REG_H,
            0x75 => REG_L,
            0x77 => REG_A,
            _ => panic!("{}", format!("Unrecognized command {:X} at ld_hl_addr_reg!", command))
        };
        self.write_memory(addr, [self.regs[reg], 0, 0, 0], 1);
        format!("LD (rHL), r{}", self.reg_letter_map[reg])
    }
    fn halt(&mut self) -> String {
        "HALT".to_string()
    }
    fn arthimetic_a(&mut self) -> String {
        let [command, additional_val, _, _] = self.get_memory(self.pc, 2);
        let (command_high, command_low) = split_byte(command);
        let (op_val, string_val) = if command_high <= 0xB {
            match command_low % 8 {
                0x0 => {
                    (self.regs[REG_B], "rB".to_string())
                }
                0x1 => {
                    (self.regs[REG_C], "rC".to_string())
                }
                0x2 => {
                    (self.regs[REG_D], "rD".to_string())
                }
                0x3 => {
                    (self.regs[REG_E], "rE".to_string())
                }
                0x4 => {
                    (self.regs[REG_H], "rH".to_string())
                }
                0x5 => {
                    (self.regs[REG_L], "rL".to_string())
                }
                0x6 => {
    
                    let val = if command_high <= 0xB {
                        let addr = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
                        let recv = self.get_memory(addr, 1);
                        recv[0]
                    } else {
                        additional_val
                    };
                    (val, format!("({:X})", val))
                }
                0x7 => {
                    (self.regs[REG_A], "rA".to_string())
                }
                _ => panic!("{}", format!("Unrecognized subcommand {:X} at arthimetic!", command_low % 8))
            }
        } else {
            self.pc += 1;
            let [val, _, _, _] = self.get_memory(self.pc, 1);
            (val, format!("{:X}", val))
        };
        let (op_high, op_low) = split_byte(op_val);
        let command_high_mod = (command_high-0x8) % 4;
        let (additional, second_half) = if command_low >= 0x8 {
            if command_high_mod != 0x3 {
                (self.c_flag, true)
            } else {
                (0, true)
            }
        } else {(0, false)};
        let code_first = if !second_half && command_high_mod == 0x3 {
            self.regs[REG_A] |= op_val;
            self.n_flag = 0;
            self.h_flag = 0;
            self.c_flag = 0;
            "OR rA, ".to_string()
        } else {
            match command_high_mod {
                0x0 => {
                    let carry_over = op_high + command_high + additional - 15;
                    if carry_over > 0 {
                        self.h_flag = 1;
                    }
                    if op_low + command_low + carry_over >= 16 {
                        self.c_flag = 1;
                    }
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(op_val).wrapping_add(additional);
                    self.n_flag = 0;
                    if second_half {
                        "ADC A, ".to_string()
                    } else {
                        "ADD A, ".to_string()
                    }
                }
                0x1 | 0x3 => {
                    let carry_over = command_high - (op_high + additional);
                    if carry_over < 0 {
                        self.h_flag = 1;
                    }
                    if command_low - op_low + carry_over < 0 {
                        self.c_flag  = 1;
                    }
                    if !second_half {
                        self.regs[REG_A] = self.regs[REG_A].wrapping_sub(op_val).wrapping_sub(additional);
                    }
                    self.regs[5] |= 1 << 6;
                    if command_high_mod == 0x1 {
                        if second_half {
                            "SBC A, ".to_string()
                        } else {
                            "SUB A, ".to_string()
                        }
                    } else {
                        "CP A, ".to_string()
                    }
                }
                0x2 => {
                    if second_half {
                        self.regs[REG_A] ^= op_val;
                        self.n_flag = 0;
                        self.h_flag = 0;
                        self.c_flag = 0;
                        "XOR A, ".to_string()
                    } else {
                        self.regs[REG_A] &= op_val;
                        self.n_flag = 0;
                        self.c_flag = 0;
                        "AND A, ".to_string()
                    }
                }
                _ => panic!("{}", format!("Unrecognized subcommand {:X} at arthimetic!", command_high_mod))
            }
        };
        if self.regs[REG_A] == 0 {
            self.z_flag = 1;
        }
        self.pc +=1;
        format!("{}{}", code_first, string_val)
    }
    fn ret(&mut self) -> String {
        let [command, _, _, _] = self.get_memory(self.pc, 1);
        let [addr_low, addr_high] = self.pop_stack();
        let addr = combine_bytes(addr_high, addr_low);
        self.sp -= 2;
        self.pc += 1;
        match command {
            0xC0 => {
                if self.z_flag == 0 {
                    self.pc = addr;
                    self.cycle_modification = 20;
                }
                "RET NZ".to_string()
            }
            0xD0 => {
                if self.c_flag == 0 {
                    self.pc = addr;
                    self.cycle_modification = 20;
                }
                "RET NC".to_string()
            }
            0xC8 => {
                if self.z_flag == 1{
                    self.pc = addr;
                    self.cycle_modification = 20;
                }
                "RET Z".to_string()
            }
            0xD8 => {
                if self.c_flag == 1{
                    self.pc = addr;
                    self.cycle_modification = 20;
                }
                "RET C".to_string()
            }
            0xC9 => {
                self.pc = addr;
                "RET".to_string()
            }
            0xD9 => {
                self.pc = addr;
                self.interrupts_enable = true;
                "RETI".to_string()
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at ret!", command))
        }
    }
    fn pop(&mut self) -> String {
        let [command, _, _, _] = self.get_memory(self.pc, 1);
        let [low_val, high_val] = self.pop_stack();
        self.pc += 1;
        self.sp += 2;
        match command {
            0xC2 => {
                self.regs[REG_C] = low_val;
                self.regs[REG_B] = high_val;
                "POP BC".to_string()
            }
            0xD2 => {
                self.regs[REG_E] = low_val;
                self.regs[REG_D] = high_val;
                "POP DE".to_string()
            }
            0xE2 => {
                self.regs[REG_L] = low_val;
                self.regs[REG_H] = high_val;
                "POP HL".to_string()
            }
            0xF2 => {
                self.write_f(low_val);
                self.regs[REG_A] = high_val;
                "POP AF".to_string()
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at pop!", command))
        }
    }
    fn push(&mut self) -> String {
        let [command,_,_,_] = self.get_memory(self.pc, 1);
        self.pc += 1;
        let ( high_val, low_val, code) = match command {
            0xC2 => {
                (self.regs[REG_B], self.regs[REG_C], "PUSH rBC".to_string())
            }
            0xD2 => {
                (self.regs[REG_D], self.regs[REG_E], "PUSH rDE".to_string())
            }
            0xE2 => {
                (self.regs[REG_H], self.regs[REG_L], "PUSH rHL".to_string())
            }
            0xF2 => {
                (self.regs[REG_A], self.get_f(), "PUSH rAF".to_string())
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at push!", command))
        };
        self.push_stack(high_val, low_val);
        code
    }
    fn call(&mut self) -> String {
        let [command, addr_low, addr_high, _] = self.get_memory(self.pc, 3);
        let addr = combine_bytes(addr_high, addr_low);
        let (pc_high, pc_low) = split_u16(self.pc);
        self.pc += 3;
        match command {
            0xC4 => {
                if self.z_flag == 0 {
                    self.push_stack(pc_high, pc_low);
                    self.pc = addr;
                    self.cycle_modification = 24;
                }
                format!("CALL NZ {:X}", addr)
            }
            0xD4 => {
                if self.c_flag == 0 {
                    self.push_stack(pc_high, pc_low);
                    self.pc = addr;
                    self.cycle_modification = 24;
                }
                format!("CALL NC {:X}", addr)
            }
            0xCC => {
                if self.z_flag == 1 {
                    self.push_stack(pc_high, pc_low);
                    self.pc = addr;
                    self.cycle_modification = 24;
                }
                format!("CALL Z {:X}", addr)
            }
            0xDC => {
                if self.c_flag == 1 {
                    self.push_stack(pc_high, pc_low);
                    self.pc = addr;
                    self.cycle_modification = 24;
                }
                format!("CALL C {:X}", addr)
            }
            0xCD => {
                self.push_stack(pc_high, pc_low);
                self.pc = addr;
                self.cycle_modification = 24;
                format!("CALL {:X}", addr)
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at call!", command))
            
        }

    }
    fn rst(&mut self) -> String {
        let [command, _, _, _] = self.get_memory(self.pc, 1);
        let (high_command, low_command) = split_byte(command);
        let (high_pc, low_pc) = split_u16(self.pc);
        self.push_stack(high_pc,low_pc);
        let new_pc = if low_command == 0xF {
            (10 * (high_command as u16) - 0xC) + 8
        } else {
            10 * (high_command as u16 - 0xC)
        };
        format!("RST {:X}", new_pc)
    }
    fn jp(&mut self) -> String {
        let [command, low_byte, high_byte, _] = self.get_memory(self.pc, 3);
        let addr = combine_bytes(high_byte, low_byte);
        let (condition, code) = match command {
            0xC2 => (self.z_flag == 0, "JP NZ ".to_string()),
            0xD2 => (self.c_flag == 0, "JP NC ".to_string()),
            0xC3 => (true, "JP ".to_string()),
            0xCA => (self.z_flag == 1, "JP Z ".to_string()),
            0xDA => (self.c_flag == 1, "JP C ".to_string()),
            _ => panic!("{}", format!("Unrecognized command {:X} at jp!", command))
        };
        self.pc = if condition {
            self.cycle_modification = 16;
            addr
        } else {
            self.pc + 3
        };
        format!("{}({:X})", code, addr)
    }
    fn add_sp_i8(&mut self) -> String {
        let [_, val , _, _] = self.get_memory(self.pc, 2);
        self.sp = self.sp.wrapping_add((val as i8) as u16);
        self.sp = if (val as i8) < 0 {
            let minus_val: u8 = ((val as i8) * -1).try_into().unwrap();
            self.h_flag = if (((self.sp & 0xF) as u8 + (minus_val & 0xF)) & 0x10) == 0x10 {
                1
            } else {
                0
            };
            self.c_flag = if self.sp < minus_val as u16 {
                1
            } else {
                0
            };
            self.sp - minus_val as u16
        } else {
            self.h_flag = if (((val & 0xF) + (self.sp & 0xF) as u8) & 0x10) == 0x10 {
                1
            } else {
                0
            };
            self.c_flag = if val as u16 + self.sp > CARRY_LIMIT {
                1
            } else {
                0
            };
            self.sp + val as u16
        };
        self.z_flag = 0;
        self.n_flag = 0;
        self.pc += 2;
        format!("ADD SP, {:X}", val)
    }
    fn ld_hl_sp_i8(&mut self) -> String {
        let [_, val , _, _] = self.get_memory(self.pc, 2);
        let (hl_val_high, hl_val_low) = split_u16(self.sp.wrapping_add((val as i8) as u16));
        self.regs[REG_H] = hl_val_high;
        self.regs[REG_L] = hl_val_low;
        self.pc += 2;
        format!("LD HL, SP + {:X}", val)
    }
    fn ld_sp_hl(&mut self) -> String {
        self.sp = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
        self.pc += 1;
        "LD SP, HL".to_string()
    }
    fn ei(&mut self) -> String {
        self.interrupts_enable = true;
        self.pc += 1;
        "EI".to_string()
    }
    fn di(&mut self) -> String {
        self.interrupts_enable = false;
        self.pc += 1;
        "DI".to_string()
    }
}

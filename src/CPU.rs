const REG_A: u8 = 0;
const REG_B: u8 = 1;
const REG_C: u8 = 2;
const REG_D: u8 = 3;
const REG_E: u8 = 4;
const REG_H: u8 = 5;
const REG_L: u8 = 6;


pub struct CentralProcessingUnit {
    regs: [u8; 7],
    reg_letter_map = [String; 7],
    pc: u16,
    sp: u16,
    z_flag: u8,
    n_flag: u8,
    h_flag: u8,
    c_flag: u8,
    memory_mut: Arc<Mutex<[u8; 65536]>>,
    function_map: [fn() -> String; 256]
}

impl CentralProcessingUnit {

    pub fn new(memory_mut: Arc<Mutex<[u8; 65536]>>) -> CentralProcessingUnit {
        regs: [0u8; 7];
        reg_letter_map = [
            'A'.to_string(),
            'B'.to_string(),
            'C'.to_string(),
            'D'.to_string(),
            'E'.to_string(),
            'H'.to_string(),
            'L'.to_string(),
        ]
        let mut pc = 0x100;
        let mut sp = 0xFFFE;
        let mut interrupts_enable = false;
        CentralProcessingUnit {
            regs,
            reg_letter_map,
            pc,
            sp,
            memory_mut,
        }
    }
    fn get_f(&self) -> u16 {
        (self.z_flag << 7) + (self.n_flag << 6) + (self.h_flag << 5) + (self.c_flag << 4)
    }
    fn write_f(&mut self, val: u8) {
        self.z_flag = (val >> 7) & 1;
        self.n_flag = (val >> 6) & 1;
        self.h_flag = (val >> 5) & 1;
        self.c_flag = (val >> 4) & 1;
    }
    fn nop(&mut self) -> String {
        self.pc += 1
        "NOP".to_string()
    }
    fn ld_reg_16(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte_1 = *memory[pc];
        let byte_2 = *memory[pc+1];
        let byte_3 = *memory[pc+2];
        let code = "LD ".to_string()
        match byte_1 {
            0x01 => {
                self.regs[REG_B] = byte_2;
                self.regs[REG_C] = byte_3;
                code.push_str("BC ");

            },
            0x11 => {
                self.regs[REG_D] = byte_2;
                self.regs[REG_E] = byte_3;
                code.push_str("DE ");
            }
            0x21 => {
                self.regs[REG_H] = byte_2;
                self.regs[REG_L] = byte_3;
                code.push_str("HL ");
            }
            0x31 => {
                self.sp = (byte_2 >> 8) + byte_3;
                code.push_str("SP ");
            }
        }
        code.push_str(&format!("{:X}", byte_2));
        code.push_str(&format!("{:X}", byte_3));
        self.pc += 3;
        code
    }
    fn ld_reg_addr_a(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte = *memory[pc];
        let mut addr: u16 = 0;
        let code = "LD ".to_string()
        match byte {
            0x02 => {
                addr = (self.regs[REG_B] << 8) + self.regs[REG_C];
                code.push_str("(BC) ");

            },
            0x12 => {
                addr = (self.regs[REG_D] << 8) + self.regs[REG_E];
                code.push_str("(DE) ");
            }
            0x22 => {
                let HL: u16 = (self.regs[REG_H] << 8) + self.regs[REG_L];
                addr = HL;
                HL.wrapping_add(1);
                self.regs[REG_L] = HL & 0b11111111;
                self.regs[REG_H] = HL >> 8;
                code.push_str("(HL +) ");
            }
            0x32 => {
                let HL: u16 = (self.regs[REG_H] << 8) + self.regs[REG_L];
                addr = HL;
                HL.wrapping_sub(1);
                self.regs[REG_L] = HL & 0b11111111;
                self.regs[REG_H] = HL >> 8;
                code.push_str("(HL -) ");
            }
        }
        *memory[addr] = self.regs[REG_A];
        self.pc += 1;
        code
    }
    fn inc_reg_16(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte = *memory[self.pc];
        let code = "INC ".to_string()
        if byte_1 == 0x33 {
            self.sp.wrapping_add(1);
            code.push_str("SP");
        } else {
            let mut r_low: usize = 9;
            let mut r_high: usize = 9;
            match byte_1  {
                0x03 => {
                    r_low = REG_C;
                    r_high = REG_B;
                    code.push_str("BC");
                }
                0x13 => {
                    r_low = REG_E;
                    r_high = REG_D;
                    code.push_str("DE");
                }
                0x23 => {
                    r_low = REG_L;
                    r_high = REG_H;
                    code.push_str("HL");
                }
            }
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
        }
        self.pc += 1;
        code
    }
    fn inc_reg_8(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte = *memory[self.pc];
        let code = "INC ".to_string();
        let val: u8 = 0;
        match byte {
            0x04 => {
                self.regs[REG_B].wrapping_add(1);
                let val = self.regs[REG_B];
                code.push_str("B");
            }
            0x14 => {
                self.regs[REG_D].wrapping_add(1);
                let val = self.regs[REG_D];
                code.push_str("D");
            }
            0x24 => {
                self.regs[REG_H].wrapping_add(1);
                let val = self.regs[REG_H];
                code.push_str("H");
            }
            0x34 => {
                let addr: u16 = (self.regs[REG_H] << 8) + self.regs[REG_L];
                *memory[addr].wrapping_add(1);
                let val = *memory[addr];
                code.push_str("(HL)");
            }
            0x0C => {
                self.regs[REG_C].wrapping_add(1);
                let val = self.regs[REG_C];
                code.push_str("C");
            }
            0x1C => {
                self.regs[REG_E].wrapping_add(1);
                let val = self.regs[REG_E];
                code.push_str("E");
            }
            0x2C => {
                self.regs[REG_L].wrapping_add(1);
                let val = self.regs[REG_L];
                code.push_str("L");
            }
            0x3C => {
                self.regs[REG_A].wrapping_add(1);
                let val = self.regs[REG_A];
                code.push_str("A");
            }
        }
        if val == 0 {
            self.z_flag = 1;
        }
        self.n_flag = 0;
        if (val & 0b00001111) == 0 {
            self.h_flag = 1;
        }
        self.pc += 1;
        code
    }
    fn dec_reg_8(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte = *memory[self.pc];
        let code = "INC ".to_string();
        let val: u8 = 0;
        match byte {
            0x05 => {
                self.regs[REG_B].wrapping_sub(1);
                let val = self.regs[REG_B];
                code.push_str("B");
            }
            0x15 => {
                self.regs[REG_D].wrapping_sub(1);
                let val = self.regs[REG_D];
                code.push_str("D");
            }
            0x25 => {
                self.regs[REG_H].wrapping_sub(1);
                let val = self.regs[REG_H];
                code.push_str("H");
            }
            0x35 => {
                let addr: u16 = (self.regs[REG_H] << 8) + self.regs[REG_L];
                *memory[addr].wrapping_sub(1);
                let val = *memory[addr];
                code.push_str("(HL)");
            }
            0x0D => {
                self.regs[REG_C].wrapping_sub(1);
                let val = self.regs[REG_C];
                code.push_str("C");
            }
            0x1D => {
                self.regs[REG_E].wrapping_sub(1);
                let val = self.regs[REG_E];
                code.push_str("E");
            }
            0x2D => {
                self.regs[REG_L].wrapping_sub(1);
                let val = self.regs[REG_L];
                code.push_str("L");
            }
            0x3D => {
                self.regs[REG_A].wrapping_sub(1);
                let val = self.regs[REG_A];
                code.push_str("A");
            }
        }
        if val == 0 {
            self.z_flag = 1;
        }
        self.n_flag = 0;
        if (val & 0b00001111) == 0b1111 {
            self.h_flag = 0;
        }
        self.pc += 1;
        code
    }
    fn ld_reg_8(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte_1 = *memory[pc];
        let byte_2 = *memory[pc+1];
        let code = "LD ".to_string()
        match byte_1 {
            0x06 => {
                self.regs[REG_B] = byte_2;
                code.push_str("B ");
            },
            0x16 => {
                self.regs[REG_D] = byte_2;
                code.push_str("D ");
            }
            0x26 => {
                self.regs[REG_H] = byte_2;
                code.push_str("H ");
            }
            0x36 => {
                let addr: u16 = (self.regs[REG_H] << 8) + self.regs[REG_L];
                *memory[addr] = byte_2;
                code.push_str("(HL) ");
            }
            0x0E => {
                self.regs[REG_C] = byte_2;
                code.push_str("C ");
            },
            0x1E => {
                self.regs[REG_E] = byte_2;
                code.push_str("E ");
            },
            0x2E => {
                self.regs[REG_L] = byte_2;
                code.push_str("L ");
            },
            0x3E => {
                self.regs[REG_A] = byte_2;
                code.push_str("A ");
            },
        }
        code.push_str(&format!("{:X}", byte_2));
        self.pc += 2;
        code
    }
    fn rot_a_left(&mut self) -> String {
        let bit = self.regs[REG_A] >> 7;
        self.regs[REG_A] <<= 1;
        self.regs[REG_A] += bit;
        regs[5] &= 0b00011111;
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

        if (!self.n_flag) {  // after an addition, adjust if (half-)carry occurred or if result is out of bounds
            if (self.c_flag || regs[REG_A] > 0x99) { regs[REG_A] += 0x60; self.c_flag = 1; }
            if (self.h_flag || (regs[REG_A] & 0x0F) > 0x09) { regs[REG_A] += 0x6; }
        } else {  // after a subtraction, only adjust if (half-)carry occurred
            if (self.c_flag) { regs[REG_A] -= 0x60; }
            if (self.h_flag) { regs[REG_A] -= 0x6; }
        }
        self.pc+=1;
        if regs[REG_A] == 0 {
            self.z_flag = 1;
        }
        self.h_flag = 0;
        "DAA".to_string()
    }
    fn scf(&mut self) -> String {
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = 1
        self.pc += 1;
        "SCF".to_string()
    }
    fn ld_addr_sp(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let addr = (self.pc + 1 << 8) + self.pc
        *memory[addr] = self.sp & 0xFF;
        *memory[addr + 1] = self.sp >> 4;
        let mut code = "LD ".to_string();
        code.push_str("(");
        code.push_str(&format!("{:X}", addr));
        code.push_str(") SP");
        self.pc += 2;
        code
    }
    fn jr(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte_1 = *memory[pc];
        let add = *memory[pc + 1] as i8;
        let mut condition: = false;
        let code = "JR ".to_string()
        match byte_2 {
            0x18 => {
                condition = true;

            },
            0x20 => {
                condition = ((self.z_flag) == 0);
                code.push_str("NZ ");
            },
            0x28 => {
                condition = ((self.z_flag) == 1);
                code.push_str("Z ");
            },
            0x30 => {
                condition = ((self.c_flag) == 0);
                code.push_str("NC ");
            },
            0x38 => {
                condition = ((self.c_flag) == 1);
                code.push_str("C ");
            },
        }
        code.push_str(&format!("{:X}", add));
        if condition {
            self.pc = self.pc.wrapping_add(add as u16);
        }
        code
    }
    fn ld_a_reg_addr(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte = *memory[pc];
        let mut addr: u16 = 0;
        let code = "LD A".to_string()
        match byte {
            0x0A => {
                addr = (self.regs[REG_B] << 8) + self.regs[REG_C];
                code.push_str("(BC)");

            },
            0x1A => {
                addr = (self.regs[REG_D] << 8) + self.regs[REG_E];
                code.push_str("(DE)");
            }
            0x2A => {
                let HL: u16 = (self.regs[REG_H] << 8) + self.regs[REG_L];
                addr = HL;
                HL.wrapping_add(1);
                self.regs[REG_L] = HL & 0b11111111;
                self.regs[REG_H] = HL >> 8;
                code.push_str("(HL +)");
            }
            0x3A => {
                let HL: u16 = (self.regs[REG_H] << 8) + self.regs[REG_L];
                addr = HL;
                HL.wrapping_sub(1);
                self.regs[REG_L] = HL & 0b11111111;
                self.regs[REG_H] = HL >> 8;
                code.push_str("(HL -)");
            }
        }
        self.regs[REG_A] = *memory[addr];
        self.pc += 1;
        code
    }
    fn dec_reg_16(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte = *memory[self.pc];
        let code = "DEC ".to_string()
        if byte_1 == 0x3B {
            self.sp.wrapping_sub(1);
            code.push_str("SP");
        } else {
            let mut r_low: usize = 9;
            let mut r_high: usize = 9;
            match byte_1  {
                0x0B => {
                    r_low = self.regs[REG_C];
                    r_high = self.regs[REG_B];
                    code.push_str("BC");
                }
                0x1B => {
                    r_low = self.regs[REG_E];
                    r_high = self.regs[REG_D];
                    code.push_str("DE");
                }
                0x2B => {
                    r_low = self.regs[REG_L];
                    r_high = self.regs[REG_H;
                    code.push_str("HL");
                }
            }
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
        }
        self.pc += 1;
        code
    }
    fn rot_a_right(&mut self) -> String {
        let bit = self.regs[REG_A] & 1;
        self.regs[REG_A] >>= 1;
        self.regs[REG_A] += (bit << 7);

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
        self.regs[REG_A] += (self.c_flag << 7);
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
        regs[REG_A] = !regs[REG_A];
        "CPL".to_string();
    }
    fn ccf(&mut self) -> String {
        if self.c_flag == 1 {
            self.c_flag = 0;

        } else {
            self.c_flag = 1;
        }
        self.n_flag = 0;
        self.h_flag = 0;
        "CCF".to_string();
    }
    fn ld_reg_reg(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte = *memory[pc];
        let nib_1 = byte >> 4;
        let nib_2 = byte & 0b1111;
        let reg_1 = 0;
        let reg_2 = 1;
        let mut code = "LD ".to_string()
        match nib_1 {
            0x4 => {
                if nib_2 <= 0x7 {
                    reg_1 = REG_A;
                } else {
                    reg_1 = REG_B;
                }
            }
            0x5 => {
                if nib_2 <= 0x7 {
                    reg_1 = REG_C;
                } else {
                    reg_1 = REG_D;
                }
            }
            0x6 => {
                if nib_2 <= 0x7 {
                    reg_1 = REG_H;
                } else {
                    reg_1 = REG_L;
                }
            }
            0x7 => reg_1 = REG_L            
        }
        match nib_2 % 8 {
            0x0 => reg_2 = REG_B,
            0x1 => reg_2 = REG_C,
            0x2 => reg_2 = REG_D,
            0x3 => reg_2 = REG_E,
            0x4 => reg_2 = REG_H,
            0x5 => reg_2 = REG_L,
            0x7 => reg_2 = REG_A,
        }
        self.regs[reg_1] = self.regs[regs_2];
        code.push_str(&self.reg_letter_map[reg_1]);
        code.push_str(' ');
        code.push_str(&self.reg_letter_map[reg_2]);
        code
    }
    fn ld_reg_hl_addr(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte = *memory[pc];
        let nib_1 = byte >> 4;
        let nib_2 = byte & 0b1111;
        let addr = (self.regs[REG_H] << 8) + self.regs[REG_L];
        let mut reg = 0;
        let mut code = "LD ".to_string()
        if nib_2 == 6 {
            match nib_1 {
                0x4 => reg = REG_B,
                0x5 => reg = REG_D,
                0x6 => reg = REG_H,
            }
        } else {
            match nib_1 {
                0x4 => reg = REG_C,
                0x5 => reg = REG_E,
                0x6 => reg = REG_L,
                0x7 => reg = REG_A
            }
        }
        self.regs[reg] = *memory[addr];
        code.push_str(&self.reg_letter_map[reg]);
        code.push_str(" ");
        code.push_str(&format!("({:X})", addr));
        code 
    }
    fn ld_hl_addr_reg(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte = *memory[self.pc];
        let nib_1 = byte >> 4;
        let nib_2 = byte & 0b1111;
        let addr = (self.regs[REG_H] << 8) + self.regs[REG_L];
        let mut reg = 0;
        let mut code = "LD ".to_string()
        if nib_2 == 6 {
            match nib_1 {
                0x4 => reg = REG_B,
                0x5 => reg = REG_D,
                0x6 => reg = REG_H,
            }
        } else {
            match nib_1 {
                0x4 => reg = REG_C,
                0x5 => reg = REG_E,
                0x6 => reg = REG_L,
                0x7 => reg = REG_A,
            }
        }
        *memory[addr] = self.regs[reg];
        code.push_str(&format!("({:X})", addr));
        code.push_str(" ");
        code.push_str(&self.reg_letter_map[reg]);
        code 
    }
    fn halt(&self) -> String {
        "HALT"
    }
    fn arthimetic_a(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let byte = *memory[self.pc];
        let nib_1 = byte >> 4;
        let nib_2 = byte & 0b1111;
        let mut op_val = 0;
        let mut code_val = ""
        if nib_1 <= 0xB {
            op_val = match nib_2 % 8 {
                0x0 => {

                    code_val = "B".to_string();
                    regs[REG_B]
                }
                0x1 => {

                    code_val = "C".to_string();
                    regs[REG_C]
                }
                0x2 => {

                    code_val = "D".to_string();
                    regs[REG_D]
                }
                0x3 => {

                    code_val = "E".to_string();
                    regs[REG_E]
                }
                0x4 => {

                    code_val = "H".to_string();
                    regs[REG_H]
                }
                0x5 => {

                    code_val = "L".to_string();
                    regs[REG_L]
                }
                0x6 => {
                    addr = (self.regs[REG_H] << 8) + self.regs[REG_L]

                    code_val = format!("({:X})".to_string());
                    *memory[addr]
                }
                0x7 => {
                    code_val = "A".to_string();
                    regs[REG_A]
                }
            }
        } else {
            op_val = *memory[self.pc + 1];
            self.pc += 1;
            code_val = format!("{:X}".to_string());
        }
        let op_nib_1 = op_val >> 4;
        let op_nib_2 = op_val & 0b1111;
        let mut additional = 0;
        let nib_1_mod = (nib_1-0x8) % 4
        let mut second_half = false;
        let mut code = "";
        if nib_2 >= 0x8 {
            if nib_1_mod != 0x3 {
                additional += (self.c_flag) & 1;
            }
            second_half = true;
        }
        if !second_half && nib_1_mod == 0x3 {
            regs[REG_A] |= op_val;
            self.n_flag = 0;
            self.h_flag = 0;
            self.c_flag = 0;
            code = "OR A, ".to_string();
        } else {
            match nib_1_mod {
                0x0 => {
                    let carry_over = op_nib_1 + nib_1 + additional - 15;
                    if carry_over > 0 {
                        self.h_flag = 1;
                    }
                    if op_nib_2 + nib_2 + carry_over >= 16 {
                        self.c_flag = 1;
                    }
                    regs[REG_A] = regs[REG_A].wrapping_add(op_val);
                    self.n_flag = 0;
                    if second_half {
                        code = "ADC A, ".to_string();
                    } else {
                        code = "ADD A, ".to_string();
                    }
                }
                0x1 | 0x3 => {
                    let carry_over = nib_1 - (op_nib_1 + additional);
                    if carry_over < 0 {
                        self.h_flag = 1;
                    }
                    if nib_2 - op_nib_2 + carry_over < 0 {
                        self.c_flag  = 1;
                    }
                    if !second_half {
                        regs[REG_A] = regs[REG_A].wrapping_sub(op_val);
                    }
                    regs[5] |= (1 << 6);
                    if nib_1_mod == 0x1 {
                        if second_half {
                            code = "SBC A, ".to_string();
                        } else {
                            code = "SUB A, ".to_string();
                        }
                    } else {
                        code = "CP A, ";
                    }
                }
                0x2 => {
                    if second_half {
                        regs[REG_A] ^= op_val;
                        self.n_flag = 0;
                        self.h_flag = 0;
                        self.c_flag = 0;
                        code = "XOR A, ".to_string();
                    } else {
                        regs[REG_A] &= op_val;
                        self.n_flag = 0;
                        self.c_flag = 0;
                        code = "AND A, ".to_string();
                    }
                }
            }
        }
        if regs[REG_A] == 0 {
            self.z_flag = 1;
        }
        self.pc +=1;
        code.push_str(&code_val);
        code
    }
    fn ret(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let command = *memory[self.pc];
        let byte_1 = *memory[self.sp];
        let byte_2 = *memory[self.sp - 1];
        addr = (byte_2 << 8) + byte_1;
        let mut code = "RET ".to_string()
        self.sp -= 2;
        match command {
            0xC0 => {
                if !self.z_flag {
                    self.pc = addr;
                    code.push_str("NZ");
                }
            }
            0xD0 => {
                if !self.c_flag {
                    self.pc = addr;
                    code.push_str("NC");
                }
            }
            0xC8 => {
                if self.z_flag {
                    self.pc = addr;
                    code.push_str("Z");
                }
            }
            0xD8 => {
                if self.c_flag {
                    self.pc = addr;
                    code.push_str("C");
                }
            }
            0xC9 => {
                self.pc = addr;
            }
            0xD9 => {
                self.pc = addr;
                interrupts_enable = true;
            }
        }
        self.pc += 1;
        code
    }
    fn pop(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let command = *memory[self.pc];
        let low_val = *memory[self.sp];
        let high_val = *memory[self.sp + 1];
        let mut code = "POP ".to_string()
        self.sp += 2;
        match command {
            0xC2 => {
                self.regs[REG_C] = low_val;
                self.regs[REG_B] = high_val;
                code.push_str("BC");
            }
            0xD2 => {
                self.regs[REG_E] = low_val;
                self.regs[REG_D] = high_val;
                code.push_str("DE");
            }
            0xE2 => {
                self.regs[REG_L] = low_val;
                self.regs[REG_H] = high_val;
                code.push_str("HL");
            }
            0xF2 => {
                self.write_f(low_val);
                self.regs[REG_A] = high_val;
                code.push_str("AF");
            }
        }
        code
        self.pc += 1;
    }
    fn push(&mut self) -> String {
        let memory = memory_mut.lock().unwrap();
        let command = *memory[self.pc];
        let mut low_val = 0;
        let mut high_val = 0;
        let mut code = "PUSH ".to_string()
        match command {
            0xC2 => {
                low_val = self.regs[REG_C];
                high_val = self.regs[REG_B];
                code.push_str("BC");
            }
            0xD2 => {
                low_val = self.regs[REG_E];
                high_val = self.regs[REG_D];
                code.push_str("DE");
            }
            0xE2 => {
                low_val = self.regs[REG_L];
                high_val = self.regs[REG_H];
                code.push_str("HL");
            }
            0xF2 => {
                low_val = self.get_f();
                high_val = self.regs[REG_A];
                code.push_str("AF");
            }
        }
        *memory[self.sp] = high_val;
        *memory[self.sp - 1] = low_val;
        self.sp += 2;
        self.pc += 1;
        code
    }
}

pub struct CentralProcessingUnit {
    regs: [u8; 9],
    reg_letter_map = [String; 9],
    pc: u16,
    sp: u16,
    memory_mut: Arc<Mutex<[u8; 65536]>>,
    function_map: [fn() -> String; 256]
}

impl CentralProcessingUnit {
    pub fn new(memory_mut: Arc<Mutex<[u8; 65536]>>) -> CentralProcessingUnit {
        regs: [0u8; 9];
        reg_letter_map = [
            'A'.to_string(),
            'B'.to_string(),
            'C'.to_string(),
            'D'.to_string(),
            'E'.to_string(),
            'F'.to_string(),
            'G'.to_string(),
            'H'.to_string(),
            'L'.to_string(),
        ]
        let pc = 0x100;
        let sp = 0xFFFE;
        CentralProcessingUnit {
            regs,
            reg_letter_map,
            pc,
            sp,
            memory_mut,
        }
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
                self.regs[1] = byte_2;
                self.regs[2] = byte_3;
                code.push_str("BC ");

            },
            0x11 => {
                self.regs[3] = byte_2;
                self.regs[4] = byte_3;
                code.push_str("DE ");
            }
            0x21 => {
                self.regs[7] = byte_2;
                self.regs[8] = byte_3;
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
                addr = (self.regs[1] << 8) + self.regs[2];
                code.push_str("(BC) ");

            },
            0x12 => {
                addr = (self.regs[3] << 8) + self.regs[4];
                code.push_str("(DE) ");
            }
            0x22 => {
                let HL: u16 = (self.regs[7] << 8) + self.regs[8];
                addr = HL;
                HL.wrapping_add(1);
                self.regs[8] = HL & 0b11111111;
                self.regs[7] = HL >> 8;
                code.push_str("(HL +) ");
            }
            0x32 => {
                let HL: u16 = (self.regs[7] << 8) + self.regs[8];
                addr = HL;
                HL.wrapping_sub(1);
                self.regs[8] = HL & 0b11111111;
                self.regs[7] = HL >> 8;
                code.push_str("(HL -) ");
            }
        }
        *memory[addr] = self.regs[0];
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
                    r_low = 2;
                    r_high = 1;
                    code.push_str("BC");
                }
                0x13 => {
                    r_low = 3;
                    r_high = 4;
                    code.push_str("DE");
                }
                0x23 => {
                    r_low = 7;
                    r_high = 8;
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
                self.regs[1].wrapping_add(1);
                let val = self.regs[1];
                code.push_str("B");
            }
            0x14 => {
                self.regs[3].wrapping_add(1);
                let val = self.regs[3];
                code.push_str("D");
            }
            0x24 => {
                self.regs[7].wrapping_add(1);
                let val = self.regs[7];
                code.push_str("H");
            }
            0x34 => {
                let addr: u16 = (self.regs[7] << 8) + self.regs[8];
                *memory[addr].wrapping_add(1);
                let val = *memory[addr];
                code.push_str("(HL)");
            }
            0x0C => {
                self.regs[2].wrapping_add(1);
                let val = self.regs[2];
                code.push_str("C");
            }
            0x1C => {
                self.regs[4].wrapping_add(1);
                let val = self.regs[4];
                code.push_str("E");
            }
            0x2C => {
                self.regs[8].wrapping_add(1);
                let val = self.regs[8];
                code.push_str("L");
            }
            0x3C => {
                self.regs[0].wrapping_add(1);
                let val = self.regs[0];
                code.push_str("A");
            }
        }
        if val == 0 {
            regs[5] |= 0b10000000;
        }
        regs[5] &= 0b10111111;
        if (val & 0b00001111) == 0 {
            regs[5]|=  0b00100000;
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
                self.regs[1].wrapping_sub(1);
                let val = self.regs[1];
                code.push_str("B");
            }
            0x15 => {
                self.regs[3].wrapping_sub(1);
                let val = self.regs[3];
                code.push_str("D");
            }
            0x25 => {
                self.regs[7].wrapping_sub(1);
                let val = self.regs[7];
                code.push_str("H");
            }
            0x35 => {
                let addr: u16 = (self.regs[7] << 8) + self.regs[8];
                *memory[addr].wrapping_sub(1);
                let val = *memory[addr];
                code.push_str("(HL)");
            }
            0x0D => {
                self.regs[2].wrapping_sub(1);
                let val = self.regs[2];
                code.push_str("C");
            }
            0x1D => {
                self.regs[4].wrapping_sub(1);
                let val = self.regs[4];
                code.push_str("E");
            }
            0x2D => {
                self.regs[8].wrapping_sub(1);
                let val = self.regs[8];
                code.push_str("L");
            }
            0x3D => {
                self.regs[0].wrapping_sub(1);
                let val = self.regs[0];
                code.push_str("A");
            }
        }
        if val == 0 {
            regs[5] |= 0b10000000;
        }
        regs[5] &= 0b10111111;
        if (val & 0b00001111) == 0b1111 {
            regs[5]|=  0b00100000;
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
                self.regs[1] = byte_2;
                code.push_str("B ");
            },
            0x16 => {
                self.regs[3] = byte_2;
                code.push_str("D ");
            }
            0x26 => {
                self.regs[7] = byte_2;
                code.push_str("H ");
            }
            0x36 => {
                let addr: u16 = (self.regs[7] << 8) + self.regs[8];
                *memory[addr] = byte_2;
                code.push_str("(HL) ");
            }
            0x0E => {
                self.regs[2] = byte_2;
                code.push_str("C ");
            },
            0x1E => {
                self.regs[4] = byte_2;
                code.push_str("E ");
            },
            0x2E => {
                self.regs[8] = byte_2;
                code.push_str("L ");
            },
            0x3E => {
                self.regs[0] = byte_2;
                code.push_str("A ");
            },
        }
        code.push_str(&format!("{:X}", byte_2));
        self.pc += 2;
        code
    }
    fn rot_a_left(&mut self) -> String {
        let bit = self.regs[0] >> 7;
        self.regs[0] <<= 1;
        self.regs[0] += bit;
        regs[5] &= 0b00011111;
        regs[5] |= (bit << 4);
        self.pc +=1 ;
        "RLCA".to_string()
    }
    fn rot_a_left_carry(&mut self) -> String {
        let last_bit = self.regs[0] >> 7;
        let c_flag = (self.regs[5] >> 4) & 1;
        self.regs[0] <<= 1;
        self.regs[0] += c_flag;
        regs[5] &= 0b00011111;
        regs[5] |= (last_bit << 4);
        self.pc += 1;
        "RLA".to_string()
    }
    fn daa(&mut self) -> String {
        let n_flag = (regs[5] >> 6) & 1;
        let h_flag = (regs[5] >> 5) & 1;
        let c_flag = (regs[5] >> 4) & 1;
        if (!n_flag) {  // after an addition, adjust if (half-)carry occurred or if result is out of bounds
            if (c_flag || regs[0] > 0x99) { a += 0x60; c_flag = 1; }
            if (h_flag || (regs[0] & 0x0F) > 0x09) { a += 0x6; }
        } else {  // after a subtraction, only adjust if (half-)carry occurred
            if (c_flag) { regs[0] -= 0x60; }
            if (h_flag) { regs[0] -= 0x6; }
        }
        self.pc+=1;
        if regs[0] == 0 {
            regs[5] |= 0x80;
            regs[5] |= (1 << 5);
        }
        "DAA".to_string()
    }
    fn scf(&mut self) -> String {
        regs[5] &= 0b10011111;
        regs[5] |= 0x10;
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
                condition = ((regs[5]>>7) == 0);
                code.push_str("NZ ");
            },
            0x28 => {
                condition = ((regs[5]>>7) == 1);
                code.push_str("Z ");
            },
            0x30 => {
                condition = ((regs[5]>>4) == 0);
                code.push_str("NC ");
            },
            0x38 => {
                condition = ((regs[5]>>4) == 1);
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
                addr = (self.regs[1] << 8) + self.regs[2];
                code.push_str("(BC)");

            },
            0x1A => {
                addr = (self.regs[3] << 8) + self.regs[4];
                code.push_str("(DE)");
            }
            0x2A => {
                let HL: u16 = (self.regs[7] << 8) + self.regs[8];
                addr = HL;
                HL.wrapping_add(1);
                self.regs[8] = HL & 0b11111111;
                self.regs[7] = HL >> 8;
                code.push_str("(HL +)");
            }
            0x3A => {
                let HL: u16 = (self.regs[7] << 8) + self.regs[8];
                addr = HL;
                HL.wrapping_sub(1);
                self.regs[8] = HL & 0b11111111;
                self.regs[7] = HL >> 8;
                code.push_str("(HL -)");
            }
        }
        self.regs[0] = *memory[addr];
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
                    r_low = 2;
                    r_high = 1;
                    code.push_str("BC");
                }
                0x1B => {
                    r_low = 3;
                    r_high = 4;
                    code.push_str("DE");
                }
                0x2B => {
                    r_low = 7;
                    r_high = 8;
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
        let bit = self.regs[0] & 1;
        self.regs[0] >>= 1;
        self.regs[0] += (bit << 7);
        regs[5] &= 0b00011111;
        regs[5] |= (bit << 4);
        self.pc +=1 ;
        "RRCA".to_string()
    }
    fn rot_a_right_carry(&mut self) -> String {
        let bit = self.regs[0] & 1;
        let c_flag = (self.regs[5] >> 4) & 1;
        self.regs[0] >>= 1;
        self.regs[0] += (c_flag << 7);
        regs[5] &= 0b00011111;
        regs[5] |= (bit << 4);
        self.pc += 1;
        "RRA".to_string()
    }
    fn cpl(&mut self) -> String {
        regs[5] |= 0b01100000;
        regs[0] = !regs[0];
        "CPL".to_string();
    }
    fn ccf(&mut self) -> String {
        let c_flag = (regs[5] >> 4) & 1;
        if c_flag {
            regs[5] &= 0b10001111;
        } else {
            regs[5] |= 0b00010000;
            regs[5] &= 0b10011111;
        }
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
                    reg_1 = 1;
                } else {
                    reg_1 = 2;
                }
            }
            0x5 => {
                if nib_2 <= 0x7 {
                    reg_1 = 3;
                } else {
                    reg_1 = 4;
                }
            }
            0x6 => {
                if nib_2 <= 0x7 {
                    reg_1 = 7;
                } else {
                    reg_1 = 8;
                }
            }
            0x7 => reg_1 = 0            
        }
        match nib_2 {
            0x0 => reg_2 = 1,
            0x1 => reg_2 = 2,
            0x2 => reg_2 = 3,
            0x3 => reg_2 = 4,
            0x4 => reg_2 = 7,
            0x5 => reg_2 = 8,
            0x7 => reg_2 = 0,
            0x8 => reg_2 = 1,
            0x9 => reg_2 = 2,
            0xA => reg_2 = 3,
            0xB => reg_2 = 4,
            0xC => reg_2 = 7,
            0xD => reg_2 = 8,
            0xF => reg_2 = 0,
        }
        regs[reg_1] = regs[regs_2];
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
        let addr = (self.regs[7] << 8) + self.regs[8];
        let mut reg = 0;
        let mut code = "LD ".to_string()
        if nib_2 == 6 {
            match nib_1 {
                0x4 => reg = 1,
                0x5 => reg = 3,
                0x6 => reg = 7,
            }
        } else {
            match nib_1 {
                0x4 => reg = 2,
                0x5 => reg = 4,
                0x6 => reg = 8,
                0x7 => reg = 0
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
        let byte = *memory[pc];
        let nib_1 = byte >> 4;
        let nib_2 = byte & 0b1111;
        let addr = (self.regs[7] << 8) + self.regs[8];
        let mut reg = 0;
        let mut code = "LD ".to_string()
        if nib_2 == 6 {
            match nib_1 {
                0x4 => reg = 1,
                0x5 => reg = 3,
                0x6 => reg = 7,
            }
        } else {
            match nib_1 {
                0x4 => reg = 2,
                0x5 => reg = 4,
                0x6 => reg = 8,
                0x7 => reg = 0
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
}
}
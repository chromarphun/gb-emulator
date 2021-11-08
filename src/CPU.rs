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
        let byte = *memory[self.pc];
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
                    regs[1]
                }
                0x1 => {

                    code_val = "C".to_string();
                    regs[2]
                }
                0x2 => {

                    code_val = "D".to_string();
                    regs[3]
                }
                0x3 => {

                    code_val = "E".to_string();
                    regs[4]
                }
                0x4 => {

                    code_val = "H".to_string();
                    regs[7]
                }
                0x5 => {

                    code_val = "L".to_string();
                    regs[8]
                }
                0x6 => {
                    addr = (self.regs[7] << 8) + self.regs[8]

                    code_val = format!("({:X})".to_string());
                    *memory[addr]
                }
                0x7 => {
                    code_val = "A".to_string();
                    regs[0]
                }
            }
        } else {
            op_val = *memory[self.pc + 1];
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
                additional += (regs[5] >> 4) & 1;
            }
            second_half = true;
        }
        if !second_half && nib_1_mod == 0x3 {
            regs[0] |= op_val;
            regs[5] &= 0b10001111;
            code = "OR A, ".to_string();
        } else {
            match nib_1_mod {
                0x0 => {
                    let carry_over = op_nib_1 + nib_1 + additional - 15;
                    if carry_over > 0 {
                        regs[5] |= (1 << 5);
                    }
                    if op_nib_2 + nib_2 + carry_over >= 16 {
                        regs[5] |= (1 << 4);
                    }
                    regs[0] = regs[0].wrapping_add(op_val);
                    regs[5] &= 0b10111111;
                    if second_half {
                        code = "ADC A, ".to_string();
                    } else {
                        code = "ADD A, ".to_string();
                    }
                }
                0x1 | 0x3 => {
                    let carry_over = nib_1 - (op_nib_1 + additional);
                    if carry_over < 0 {
                        regs[5] |= (1 << 5);
                    }
                    if nib_2 - op_nib_2 + carry_over < 0 {
                        regs[5] |= (1 << 4);
                    }
                    if !second_half {
                        regs[0] = regs[0].wrapping_sub(op_val);
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
                        regs[0] ^= op_val;
                        regs[5] &= 0b10001111;
                        code = "XOR A, ".to_string();
                    } else {
                        regs[0] &= op_val;
                        regs[5] &= 0b10101111;
                        code = "AND A, ".to_string();
                    }
                }
            }
        }
        if regs[0] == 0 {
            regs[5] |= (1 << 7);
        }
        code.push_str(&code_val);
        code
    }
    fn ret(&mut self) -> String {
        let z_flag = regs[5] >> 7;
        let c_flag = (regs[5] >> 5) & 1;
        let memory = memory_mut.lock().unwrap();
        let command = *memory[self.pc];
        let byte_1 = *memory[self.sp];
        let byte_2 = *memory[self.sp - 1];
        addr = (byte_2 << 8) + byte_1;
        self.sp -= 2;
        match command {
            0xC0 => {
                if !z_flag {
                    self.pc = addr;
                }
            }
            0xD0 => {
                if !c_flag {
                    self.pc = addr;
                }
            }
            0xC8 => {
                if z_flag {
                    self.pc = addr;
                }
            }
            0xD8 => {
                if c_flag {
                    self.pc = addr;
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
    }
}

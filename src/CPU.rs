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
    fn ld_reg_addr_A(&mut self) -> String {
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
        *memory[addr] = self.regs[0]
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
        }
        code.push_str(&format!("{:X}", byte_2));
        self.pc += 2;
        code
    }
    fn rot_a_left(&mut self) -> String {
        let bit = self.regs[0] >> 7;
        self.regs[0] <<= 1;
        regs[5] &= 0b00011111;
        regs[5] |= (bit << 4);
        self.pc +=1 ;
        "RLCA".to_string()
    }
    fn rot_a_left_carry(&mut self) -> String {
        let last_bit = self.regs[0] >> 7;
        let first_bit = self.regs[0] & 1;
        self.regs[0] <<= 1;
        self.regs[0] |= first_bit;
        regs[5] &= 0b00011111;
        regs[5] |= (last_bit << 4);
        self.pc += 1;
        "RLC".to_string()
    }
}
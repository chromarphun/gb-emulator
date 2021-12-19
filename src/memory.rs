use std::collections::HashSet;
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::iter::FromIterator;
use std::path::PathBuf;

const VRAM_SIZE: usize = 0x2000;
const IRAM_SIZE: usize = 0x2000;
const OAM_SIZE: usize = 160;
const IO_SIZE: usize = 0x80;
const HRAM_SIZE: usize = 127;
const CART_TYPE_ADDR: usize = 0x147;
const ROM_BANK_ADDR: usize = 0x148;
const RAM_BANK_ADDR: usize = 0x149;
const INT_FLAG_ADDR: usize = 0xFF0F;
const P1_ADDR: usize = 0xFF00;

const BOOT_ROM: [u8; 256] = [
    0x31, 0xFE, 0xFF, 0xAF, 0x21, 0xFF, 0x9F, 0x32, 0xCB, 0x7C, 0x20, 0xFB, 0x21, 0x26, 0xFF, 0x0E,
    0x11, 0x3E, 0x80, 0x32, 0xE2, 0x0C, 0x3E, 0xF3, 0xE2, 0x32, 0x3E, 0x77, 0x77, 0x3E, 0xFC, 0xE0,
    0x47, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1A, 0xCD, 0x95, 0x00, 0xCD, 0x96, 0x00, 0x13, 0x7B,
    0xFE, 0x34, 0x20, 0xF3, 0x11, 0xD8, 0x00, 0x06, 0x08, 0x1A, 0x13, 0x22, 0x23, 0x05, 0x20, 0xF9,
    0x3E, 0x19, 0xEA, 0x10, 0x99, 0x21, 0x2F, 0x99, 0x0E, 0x0C, 0x3D, 0x28, 0x08, 0x32, 0x0D, 0x20,
    0xF9, 0x2E, 0x0F, 0x18, 0xF3, 0x67, 0x3E, 0x64, 0x57, 0xE0, 0x42, 0x3E, 0x91, 0xE0, 0x40, 0x04,
    0x1E, 0x02, 0x0E, 0x0C, 0xF0, 0x44, 0xFE, 0x90, 0x20, 0xFA, 0x0D, 0x20, 0xF7, 0x1D, 0x20, 0xF2,
    0x0E, 0x13, 0x24, 0x7C, 0x1E, 0x83, 0xFE, 0x62, 0x28, 0x06, 0x1E, 0xC1, 0xFE, 0x64, 0x20, 0x06,
    0x7B, 0xE2, 0x0C, 0x3E, 0x87, 0xE2, 0xF0, 0x42, 0x90, 0xE0, 0x42, 0x15, 0x20, 0xD2, 0x05, 0x20,
    0x4F, 0x16, 0x20, 0x18, 0xCB, 0x4F, 0x06, 0x04, 0xC5, 0xCB, 0x11, 0x17, 0xC1, 0xCB, 0x11, 0x17,
    0x05, 0x20, 0xF5, 0x22, 0x23, 0x22, 0x23, 0xC9, 0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
    0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
    0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
    0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E, 0x3C, 0x42, 0xB9, 0xA5, 0xB9, 0xA5, 0x42, 0x3C,
    0x21, 0x04, 0x01, 0x11, 0xA8, 0x00, 0x1A, 0x13, 0xBE, 0x20, 0xFE, 0x23, 0x7D, 0xFE, 0x34, 0x20,
    0xF5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xFB, 0x86, 0x20, 0xFE, 0x3E, 0x01, 0xE0, 0x50,
];

const MASKING_BITS: [usize; 9] = [0x0, 0x1, 0x3, 0x7, 0xF, 0x1F, 0x3F, 0x7F, 0xFF];

enum CartType {
    Uninitialized,
    RomOnly,
    Mbc1,
    Mbc3,
}

pub struct MemoryUnit {
    pub rom: Vec<u8>,
    vram: [u8; VRAM_SIZE],
    external_ram: Vec<u8>,
    internal_ram: [u8; IRAM_SIZE],
    oam: [u8; OAM_SIZE],
    io_registers: [u8; IO_SIZE],
    high_ram: [u8; HRAM_SIZE],
    interrupt_enable: u8,
    memory_mode: u8,
    rom_bank: usize,
    ram_bank: usize,
    mbc1_0_bank: usize,
    mbc1_5_bit_reg: usize,
    mbc1_2_bit_reg: usize,
    rom_bank_bits: usize,
    ram_enable: bool,
    cartridge_type: CartType,
    available_rom_banks: u8,
    available_ram_banks: u8,
    hold_mem: [u8; 256],
    pub in_boot_rom: bool,
    pub directional_presses: u8,
    pub action_presses: u8,
    dma_cycles: u16,
    invalid_io: HashSet<usize>,
}

impl MemoryUnit {
    pub fn new() -> MemoryUnit {
        let rom = Vec::new();
        let vram = [0; VRAM_SIZE];
        let external_ram = Vec::new();
        let internal_ram = [0; IRAM_SIZE];
        let oam = [0; OAM_SIZE];
        let io_registers = [0; IO_SIZE];
        let memory_mode = 0;
        let rom_bank = 1;
        let ram_bank = 0;
        let rom_bank_bits = 0;
        let ram_enable = false;
        let cartridge_type = CartType::Uninitialized;
        let available_rom_banks = 0;
        let available_ram_banks = 0;
        let hold_mem = [0; 256];
        let interrupt_enable = 0;
        let high_ram = [0; HRAM_SIZE];
        let in_boot_rom = true;
        let directional_presses = 0xF;
        let action_presses = 0xF;
        let dma_cycles = 0;
        MemoryUnit {
            rom,
            vram,
            external_ram,
            internal_ram,
            oam,
            io_registers,
            interrupt_enable,
            high_ram,
            memory_mode,
            rom_bank,
            ram_bank,
            mbc1_0_bank: 0,
            mbc1_5_bit_reg: 0,
            mbc1_2_bit_reg: 0,
            rom_bank_bits,
            ram_enable,
            cartridge_type,
            available_rom_banks,
            available_ram_banks,
            hold_mem,
            in_boot_rom,
            directional_presses,
            action_presses,
            dma_cycles,
            invalid_io: HashSet::from_iter(vec![
                0xFF03, 0xFF08, 0xFF09, 0xFF0A, 0xFF0B, 0xFF0C, 0xFF0D, 0xFF0E, 0xFF15, 0xFF1F,
                0xFF27, 0xFF28, 0xFF29,
            ]),
        }
    }
    pub fn get_memory(&self, addr: impl Into<usize>) -> u8 {
        let addr = addr.into() as usize;
        match addr {
            0x0000..=0x3FFF => match self.cartridge_type {
                CartType::RomOnly | CartType::Mbc3 => self.rom[addr],
                CartType::Mbc1 => self.rom[self.mbc1_0_bank * 0x4000 + addr],
                _ => panic!("Bad cart type."),
            },
            0x4000..=0x7FFF => match self.cartridge_type {
                CartType::RomOnly => self.rom[addr],
                CartType::Mbc1 | CartType::Mbc3 => self.rom[addr + 0x4000 * (self.rom_bank - 1)],
                _ => panic!("Bad cart type."),
            },
            0x8000..=0x9FFF => self.vram[addr - 0x8000],
            0xA000..=0xBFFF => {
                if self.available_ram_banks == 0 || !self.ram_enable || self.memory_mode == 1 {
                    return 0xFF;
                }
                match self.cartridge_type {
                    CartType::RomOnly => self.external_ram[addr - 0xA000],
                    CartType::Mbc1 | CartType::Mbc3 => {
                        self.external_ram[addr - 0xA000 + 0x2000 * self.ram_bank]
                    }
                    _ => panic!("Bad cart type."),
                }
            }
            0xC000..=0xDFFF => self.internal_ram[addr - 0xC000],
            0xE000..=0xFDFF => self.internal_ram[addr - 0xE000],
            0xFE00..=0xFE9F => self.oam[addr - 0xFE00],
            0xFF00..=0xFF4B => {
                if self.invalid_io.contains(&addr) {
                    0xFF
                } else {
                    self.io_registers[addr - 0xFF00]
                }
            }
            0xFF80..=0xFFFE => self.high_ram[addr - 0xFF80],
            0xFFFF => self.interrupt_enable,
            _ => 0xFF,
        }
    }
    pub fn write_memory(&mut self, addr: impl Into<usize>, val: u8) {
        let addr = addr.into() as usize;
        match addr {
            0x0000..=0x1FFF => match val {
                0x0 => self.ram_enable = false,
                0xA => self.ram_enable = true,
                _ => {}
            },
            0x2000..=0x3FFF => match self.cartridge_type {
                CartType::RomOnly => {}
                CartType::Mbc1 => {
                    println!("changing mbc1_5_reg to {}", val);
                    if val & MASKING_BITS[self.rom_bank_bits] as u8 == 0 {
                        self.mbc1_5_bit_reg = (val as usize & MASKING_BITS[self.rom_bank_bits]) + 1
                    } else {
                        self.mbc1_5_bit_reg = val as usize & MASKING_BITS[self.rom_bank_bits];
                    }
                    self.rom_bank = if self.rom_bank_bits > 5 {
                        self.mbc1_5_bit_reg + (self.mbc1_2_bit_reg << 5)
                    } else {
                        self.mbc1_5_bit_reg
                    };
                    println!(
                        "mbc1_5_bit_reg now {}, rom bank {}",
                        self.mbc1_5_bit_reg, self.rom_bank
                    );
                }
                CartType::Mbc3 => {
                    self.rom_bank = val as usize & MASKING_BITS[self.rom_bank_bits];
                }
                _ => panic!("Bad cart type."),
            },
            0x4000..=0x5FFF => match self.cartridge_type {
                CartType::RomOnly => {}
                CartType::Mbc1 => {
                    println!("changing mbc1_2_reg to {}", val);
                    self.mbc1_2_bit_reg = val as usize & 0b11;
                    if self.memory_mode == 1 && self.available_ram_banks == 4 {
                        self.ram_bank = self.mbc1_2_bit_reg;
                    }
                    if self.rom_bank_bits > 5 {
                        self.rom_bank = self.mbc1_5_bit_reg + (self.mbc1_2_bit_reg << 5);
                        if self.memory_mode == 1 {
                            self.mbc1_0_bank = self.mbc1_2_bit_reg << 5;
                        }
                    }
                }
                CartType::Mbc3 => {
                    if val < 0x4 {
                        self.memory_mode = 0;
                        self.ram_bank = val as usize & 0b11;
                    } else if val >= 0x8 {
                        self.memory_mode = 1;
                    }
                }
                _ => panic!("Bad cart type."),
            },
            0x6000..=0x7FFF => match self.cartridge_type {
                CartType::RomOnly => {}
                CartType::Mbc1 => {
                    self.memory_mode = val;
                    println!("changing banking to {}", val);
                    if val == 0 {
                        self.rom_bank = if self.rom_bank_bits > 5 {
                            self.mbc1_5_bit_reg + (self.mbc1_2_bit_reg << 5)
                        } else {
                            self.mbc1_5_bit_reg
                        };
                        self.ram_bank = 0;
                    } else {
                        if self.available_ram_banks == 4 {
                            self.ram_bank = self.mbc1_2_bit_reg;
                        } else {
                            self.ram_bank = 0;
                        }
                        if self.rom_bank_bits > 5 {
                            self.rom_bank = self.mbc1_5_bit_reg + (self.mbc1_2_bit_reg << 5);
                            self.mbc1_0_bank = self.mbc1_2_bit_reg << 5;
                        } else {
                            self.rom_bank = self.mbc1_5_bit_reg;
                        }
                    }
                }
                CartType::Mbc3 => {}
                _ => panic!("Bad cart type."),
            },
            0x8000..=0x9FFF => self.vram[addr - 0x8000] = val,
            0xA000..=0xBFFF => self.external_ram[addr - 0xA000] = val,
            0xC000..=0xDFFF => self.internal_ram[addr - 0xC000] = val,
            0xE000..=0xFDFF => self.internal_ram[addr - 0xE000] = val,
            0xFE00..=0xFE9F => self.oam[addr - 0xFE00] = val,
            0xFF00 => {
                let mut p1 = self.get_memory(P1_ADDR);
                let prev_p1 = p1;
                p1 &= 0b001111;
                p1 |= val & 0b110000;
                let p14 = (p1 >> 4) & 1;
                let p15 = (p1 >> 5) & 1;
                let mut new_bits = 0xF;
                p1 &= 0b110000;

                if p14 == 0 {
                    new_bits &= self.directional_presses;
                }
                if p15 == 0 {
                    new_bits &= self.action_presses;
                }
                p1 += new_bits;
                if ((prev_p1 | p1) - p1) & 0xF != 0 {
                    self.write_memory(INT_FLAG_ADDR, self.get_memory(INT_FLAG_ADDR) | (1 << 4));
                }
                self.io_registers[addr - 0xFF00] = p1;
            }
            0xFF46 => {
                self.io_registers[addr - 0xFF00] = val;
                self.dma_transfer(val as usize);
            }
            0xFF50 => {
                if self.in_boot_rom {
                    self.unload_boot_rom();
                    self.in_boot_rom = false;
                }
            }
            0xFF00..=0xFF7F => self.io_registers[addr - 0xFF00] = val,
            0xFF80..=0xFFFE => self.high_ram[addr - 0xFF80] = val,
            0xFFFF => self.interrupt_enable = val,
            _ => {}
        }
    }
    fn load_boot_rom(&mut self) {
        self.hold_mem.copy_from_slice(&self.rom[..256]);
        self.rom[..256].copy_from_slice(&BOOT_ROM);
    }
    fn unload_boot_rom(&mut self) {
        self.rom[..256].copy_from_slice(&self.hold_mem);
    }
    pub fn load_rom(&mut self, path: &PathBuf) {
        let mut f = File::open(path).expect("File problem!");
        f.read_to_end(&mut self.rom).expect("Read issue!");
        self.cartridge_type = match self.rom[CART_TYPE_ADDR] {
            0 => CartType::RomOnly,
            1..=3 => CartType::Mbc1,
            0xF..=0x13 => CartType::Mbc3,
            _ => CartType::Uninitialized,
        };
        self.available_rom_banks = 1 << (self.get_memory(ROM_BANK_ADDR) + 1);
        self.available_ram_banks = match self.get_memory(RAM_BANK_ADDR) {
            0 => 0,
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            _ => 0,
        };
        self.external_ram
            .extend(vec![0; 0x2000 * self.available_ram_banks as usize]);
        self.rom_bank_bits = self.available_rom_banks.trailing_zeros() as usize + 1;
        println!(
            "rom banks: {}, ram banks: {}, rom bank bits: {}",
            self.available_rom_banks, self.available_ram_banks, self.rom_bank_bits
        );
        self.load_boot_rom();
    }
    fn dma_transfer(&mut self, reg: usize) {
        let start_address = reg << 8;

        match reg >> 4 {
            0x0..=0x3 => {
                let end_address = (start_address + 0xA0) as usize;
                self.oam
                    .copy_from_slice(&self.rom[start_address..end_address]);
            }
            0x4..=0x7 => {
                let adjusted_start_address = 16384 * (self.rom_bank) - 0x4000 + reg;
                let adjusted_end_address = adjusted_start_address + 0xA0;
                self.oam
                    .copy_from_slice(&self.rom[adjusted_start_address..adjusted_end_address]);
            }
            0x8..=0x9 => {
                let adjusted_start_address = start_address - 0x8000;
                let adjusted_end_address = adjusted_start_address + 0xA0;
                self.oam
                    .copy_from_slice(&self.vram[adjusted_start_address..adjusted_end_address]);
            }
            0xA..=0xB => {
                let adjusted_start_address = 8192 * (self.ram_bank) + start_address - 0xA000;
                let adjusted_end_address = adjusted_start_address + 0xA0;
                self.oam.copy_from_slice(
                    &self.external_ram[adjusted_start_address..adjusted_end_address],
                );
            }
            0xC..=0xD => {
                let adjusted_start_address = (start_address - 0xC000) as usize;
                let adjusted_end_address = adjusted_start_address + 0xA0;
                self.oam.copy_from_slice(
                    &self.internal_ram[adjusted_start_address..adjusted_end_address],
                );
            }
            _ => println!("DMA FAILURE"),
        }
        self.dma_cycles = 640;
    }
    pub fn dma_tick(&mut self) {
        if self.dma_cycles > 0 {
            self.dma_cycles -= 4;
        }
    }
    pub fn get_wave_ram(&self) -> [u8; 16] {
        self.io_registers[0x30..0x40]
            .try_into()
            .expect("weird length error?")
    }
}

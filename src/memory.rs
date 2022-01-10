use crate::emulator::GameBoyEmulator;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::cpu::CentralProcessingUnit;
use crate::emulator::RequestSource;

use crate::bootroms::*;
use crate::constants::*;
use crate::ppu::PictureProcessingUnit;
use crate::timing::Timer;
use std::sync::atomic::Ordering;

const SOURCE: RequestSource = RequestSource::MAU;

const MASKING_BITS: [usize; 9] = [0x0, 0x1, 0x3, 0x7, 0xF, 0x1F, 0x3F, 0x7F, 0xFF];

#[inline]
fn combine_bytes(high_byte: u8, low_byte: u8) -> u16 {
    ((high_byte as u16) << 8) + low_byte as u16
}

#[inline]
fn split_u16(val: u16) -> (u8, u8) {
    ((val >> 8) as u8, (val & 0xFF) as u8)
}

#[inline]
fn convert_to_8_bit(color5: u8) -> u8 {
    (color5 << 3) | (color5 >> 2)
}

#[derive(Serialize, Deserialize, Clone, Copy)]
enum CartType {
    Uninitialized,
    RomOnly,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
}
#[derive(Serialize, Deserialize)]
struct SaveGame {
    vram_0: Vec<u8>,
    vram_1: Vec<u8>,
    external_ram: Vec<u8>,
    internal_ram: Vec<u8>,
    oam: Vec<u8>,
    io_registers: Vec<u8>,
    high_ram: Vec<u8>,
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
    available_rom_banks: usize,
    available_ram_banks: u8,
    hold_mem: Vec<u8>,
    in_boot_rom: bool,
    directional_presses: u8,
    action_presses: u8,
    dma_cycles: u32,
    ppu_mode: u8,
    bg_color_ram: Vec<u8>,
    obj_color_ram: Vec<u8>,
    bg_color_inc: bool,
    obj_color_inc: bool,
    vram_bank: u8,
    wram_bank: usize,
    cgb: bool,
    hdma_primed: bool,
    hdma_blocks: u8,
    hdma_active: bool,
    hdma_current_dest_addr: usize,
    hdma_current_source_addr: usize,
    valid_io: Vec<bool>,
    cpu: CentralProcessingUnit,
    ppu: PictureProcessingUnit,
    timer: Timer,
}

pub struct MemoryUnit {
    rom: Vec<u8>,
    vram_0: Vec<u8>,
    vram_1: Vec<u8>,
    external_ram: Vec<u8>,
    internal_ram: Vec<u8>,
    oam: Vec<u8>,
    io_registers: Vec<u8>,
    high_ram: Vec<u8>,
    pub interrupt_enable: u8,
    memory_mode: u8,
    rom_bank: usize,
    ram_bank: usize,
    mbc1_0_bank: usize,
    mbc1_5_bit_reg: usize,
    mbc1_2_bit_reg: usize,
    rom_bank_bits: usize,
    ram_enable: bool,
    cartridge_type: CartType,
    available_rom_banks: usize,
    available_ram_banks: u8,
    hold_mem: Vec<u8>,
    in_boot_rom: bool,
    pub directional_presses: u8,
    pub action_presses: u8,
    dma_cycles: u32,
    pub ppu_mode: u8,
    bg_color_ram: Vec<u8>,
    obj_color_ram: Vec<u8>,
    bg_color_inc: bool,
    obj_color_inc: bool,
    vram_bank: u8,
    wram_bank: usize,
    cgb: bool,
    hdma_primed: bool,
    hdma_blocks: u8,
    hdma_active: bool,
    hdma_current_dest_addr: usize,
    hdma_current_source_addr: usize,
    valid_io: Vec<bool>,
}

impl MemoryUnit {
    pub fn new() -> MemoryUnit {
        let mut valid_io = vec![true; 0x80];
        for ind in NON_BLOCK_INVALID_IO.iter() {
            valid_io[*ind] = false;
        }
        valid_io[0x4C..0x80].copy_from_slice(&[false; (0x80 - 0x4C)]);
        MemoryUnit {
            rom: Vec::new(),
            vram_0: vec![0; VRAM_SIZE],
            vram_1: vec![0; VRAM_SIZE],
            external_ram: Vec::new(),
            internal_ram: vec![0; IRAM_SIZE],
            oam: vec![0; OAM_SIZE],
            io_registers: vec![0; IO_SIZE],
            interrupt_enable: 0,
            high_ram: vec![0; HRAM_SIZE],
            memory_mode: 0,
            rom_bank: 1,
            ram_bank: 0,
            mbc1_0_bank: 0,
            mbc1_5_bit_reg: 0,
            mbc1_2_bit_reg: 0,
            rom_bank_bits: 0,
            ram_enable: false,
            cartridge_type: CartType::Uninitialized,
            available_rom_banks: 0,
            available_ram_banks: 0,
            hold_mem: vec![0; 2048],
            in_boot_rom: true,
            directional_presses: 0xF,
            action_presses: 0xF,
            dma_cycles: 0,
            ppu_mode: 0,
            bg_color_ram: vec![0; 64],
            obj_color_ram: vec![0; 64],
            bg_color_inc: false,
            obj_color_inc: false,
            vram_bank: 0,
            wram_bank: 1,
            cgb: false,
            hdma_primed: false,
            hdma_blocks: 0,
            hdma_active: false,
            hdma_current_dest_addr: 0,
            hdma_current_source_addr: 0,
            valid_io,
        }
    }
}
impl GameBoyEmulator {
    pub fn get_memory(&self, addr: impl Into<usize>, source: RequestSource) -> u8 {
        let addr = addr.into() as usize;
        match addr {
            0x0000..=0x3FFF => match self.mem_unit.cartridge_type {
                CartType::RomOnly | CartType::Mbc2 | CartType::Mbc3 | CartType::Mbc5 => {
                    self.mem_unit.rom[addr]
                }
                CartType::Mbc1 => {
                    self.mem_unit.rom[self.mem_unit.mbc1_0_bank * ROM_BANK_SIZE + addr]
                }
                _ => panic!("Bad cart type."),
            },
            0x4000..=0x7FFF => match self.mem_unit.cartridge_type {
                CartType::RomOnly => self.mem_unit.rom[addr],
                CartType::Mbc1 | CartType::Mbc2 | CartType::Mbc3 | CartType::Mbc5 => {
                    self.mem_unit.rom[addr + ROM_BANK_SIZE * self.mem_unit.rom_bank - ROM_BANK_SIZE]
                }
                _ => panic!("Bad cart type."),
            },
            0x8000..=0x9FFF => {
                if self.mem_unit.ppu_mode != DRAWING_MODE || source == RequestSource::PPU {
                    if self.mem_unit.vram_bank == 0 {
                        self.mem_unit.vram_0[addr - VRAM_START_ADDR]
                    } else {
                        self.mem_unit.vram_1[addr - VRAM_START_ADDR]
                    }
                } else {
                    0xFF
                }
            }
            0xA000..=0xBFFF => match self.mem_unit.cartridge_type {
                CartType::RomOnly => 0xFF,
                CartType::Mbc1 | CartType::Mbc3 => {
                    if self.mem_unit.available_ram_banks == 0
                        || !self.mem_unit.ram_enable
                        || self.mem_unit.memory_mode == 1
                    {
                        0xFF
                    } else {
                        self.mem_unit.external_ram
                            [addr - ERAM_START_ADDR + ERAM_BANK_SIZE * self.mem_unit.ram_bank]
                    }
                }
                CartType::Mbc2 => {
                    if !self.mem_unit.ram_enable {
                        0xFF
                    } else {
                        self.mem_unit.external_ram[(addr - ERAM_START_ADDR) % 0x200] & 0xF
                    }
                }
                CartType::Mbc5 => {
                    if !self.mem_unit.ram_enable {
                        0xFF
                    } else {
                        self.mem_unit.external_ram
                            [addr - ERAM_START_ADDR + ERAM_BANK_SIZE * self.mem_unit.ram_bank]
                    }
                }
                _ => panic!("Bad cart type."),
            },
            0xC000..=0xCFFF => self.mem_unit.internal_ram[addr - WRAM_START_ADDR],
            0xD000..=0xDFFF => {
                self.mem_unit.internal_ram[addr - WRAM_START_ADDR - WRAM_BANK_SIZE
                    + WRAM_BANK_SIZE * self.mem_unit.wram_bank]
            }
            0xE000..=0xFDFF => self.mem_unit.internal_ram[addr - 0xE000],
            0xFE00..=0xFE9F => {
                if source == RequestSource::MAU
                    || source == RequestSource::PPU
                    || self.mem_unit.ppu_mode == HBLANK_MODE
                    || self.mem_unit.ppu_mode == VBLANK_MODE
                {
                    self.mem_unit.oam[addr - OAM_START_ADDR]
                } else {
                    0xFF
                }
            }

            NR10_ADDR => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[NR10_ADDR - IO_START_ADDR] | 0x80
                } else {
                    self.mem_unit.io_registers[NR10_ADDR - IO_START_ADDR]
                }
            }
            NR11_ADDR => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[NR11_ADDR - IO_START_ADDR] | 0x3F
                } else {
                    self.mem_unit.io_registers[NR11_ADDR - IO_START_ADDR]
                }
            }
            NR14_ADDR => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[NR14_ADDR - IO_START_ADDR] | 0xBF
                } else {
                    self.mem_unit.io_registers[NR14_ADDR - IO_START_ADDR]
                }
            }
            NR21_ADDR => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[NR21_ADDR - IO_START_ADDR] | 0x3F
                } else {
                    self.mem_unit.io_registers[NR21_ADDR - IO_START_ADDR]
                }
            }
            NR24_ADDR => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[NR24_ADDR - IO_START_ADDR] | 0xBF
                } else {
                    self.mem_unit.io_registers[NR24_ADDR - IO_START_ADDR]
                }
            }
            NR30_ADDR => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[NR30_ADDR - IO_START_ADDR] | 0x7F
                } else {
                    self.mem_unit.io_registers[NR30_ADDR - IO_START_ADDR]
                }
            }
            NR32_ADDR => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[NR32_ADDR - IO_START_ADDR] | 0x9F
                } else {
                    self.mem_unit.io_registers[NR32_ADDR - IO_START_ADDR]
                }
            }
            NR34_ADDR => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[NR34_ADDR - IO_START_ADDR] | 0xBF
                } else {
                    self.mem_unit.io_registers[NR34_ADDR - IO_START_ADDR]
                }
            }
            NR44_ADDR => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[NR44_ADDR - IO_START_ADDR] | 0xBF
                } else {
                    self.mem_unit.io_registers[NR44_ADDR - IO_START_ADDR]
                }
            }
            NR52_ADDR => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[NR52_ADDR - IO_START_ADDR] | 0x70
                } else {
                    self.mem_unit.io_registers[NR52_ADDR - IO_START_ADDR]
                }
            }
            0xFF30..=0xFF3F => {
                if source == RequestSource::CPU {
                    self.wave_ram_read(addr)
                } else {
                    self.mem_unit.io_registers[addr - IO_START_ADDR]
                }
            }
            BCPS_ADDR => {
                self.mem_unit.bg_color_ram
                    [self.mem_unit.io_registers[BCPS_ADDR - IO_START_ADDR] as usize]
            }
            BCPD_ADDR => {
                self.mem_unit.obj_color_ram
                    [self.mem_unit.io_registers[BCPD_ADDR - IO_START_ADDR] as usize]
            }
            0xFF00..=0xFF7F => {
                if self.mem_unit.valid_io[addr - IO_START_ADDR] || source != RequestSource::CPU {
                    self.mem_unit.io_registers[addr - IO_START_ADDR]
                } else {
                    0xFF
                }
            }
            0xFF80..=0xFFFE => self.mem_unit.high_ram[addr - HRAM_START_ADDR],
            INT_ENABLE_ADDR => self.mem_unit.interrupt_enable,
            _ => 0xFF,
        }
    }
    pub fn write_memory(&mut self, addr: impl Into<usize>, val: u8, source: RequestSource) {
        let addr = addr.into() as usize;

        match addr {
            0x0000..=0x1FFF => match self.mem_unit.cartridge_type {
                CartType::RomOnly => {}
                CartType::Mbc1 | CartType::Mbc3 | CartType::Mbc5 => match val & 0xF {
                    0x0 => self.mem_unit.ram_enable = false,
                    0xA => self.mem_unit.ram_enable = true,
                    _ => {}
                },
                CartType::Mbc2 => {
                    let bit_8_reset = ((addr >> 8) & 1) == 0;
                    if bit_8_reset {
                        match val {
                            0x0 => self.mem_unit.ram_enable = false,
                            0xA => self.mem_unit.ram_enable = true,
                            _ => {}
                        }
                    } else {
                        self.mem_unit.rom_bank = val as usize & 0xF;
                        if self.mem_unit.rom_bank == 0 {
                            self.mem_unit.rom_bank += 1;
                        }
                    }
                }
                _ => panic!("Bad cart type."),
            },
            0x2000..=0x3FFF => match self.mem_unit.cartridge_type {
                CartType::RomOnly => {}
                CartType::Mbc1 => {
                    if val & 0x1F == 0 {
                        self.mem_unit.mbc1_5_bit_reg =
                            (val as usize & MASKING_BITS[self.mem_unit.rom_bank_bits]) + 1
                    } else {
                        self.mem_unit.mbc1_5_bit_reg =
                            val as usize & MASKING_BITS[self.mem_unit.rom_bank_bits];
                    }
                    self.mem_unit.rom_bank = if self.mem_unit.rom_bank_bits > 5 {
                        self.mem_unit.mbc1_5_bit_reg + (self.mem_unit.mbc1_2_bit_reg << 5)
                    } else {
                        self.mem_unit.mbc1_5_bit_reg
                    };
                }
                CartType::Mbc2 => {
                    let bit_8_reset = ((addr >> 8) & 1) == 0;
                    if bit_8_reset {
                        match val {
                            0x0 => self.mem_unit.ram_enable = false,
                            0xA => self.mem_unit.ram_enable = true,
                            _ => {}
                        }
                    } else {
                        self.mem_unit.rom_bank = val as usize & 0xF;
                        if self.mem_unit.rom_bank == 0 {
                            self.mem_unit.rom_bank += 1;
                        }
                    }
                }
                CartType::Mbc3 => {
                    self.mem_unit.rom_bank =
                        val as usize & MASKING_BITS[self.mem_unit.rom_bank_bits];
                }
                CartType::Mbc5 => {
                    if addr < 0x3000 {
                        self.mem_unit.rom_bank &= 0x100;
                        self.mem_unit.rom_bank |= val as usize;
                    } else {
                        self.mem_unit.rom_bank &= 0x0FF;
                        self.mem_unit.rom_bank |= (val as usize & 1) << 8;
                    }
                    self.mem_unit.rom_bank %= self.mem_unit.available_rom_banks;
                }
                _ => panic!("Bad cart type."),
            },
            0x4000..=0x5FFF => match self.mem_unit.cartridge_type {
                CartType::RomOnly => {}
                CartType::Mbc1 => {
                    self.mem_unit.mbc1_2_bit_reg = val as usize & 0b11;
                    if self.mem_unit.memory_mode == 1 && self.mem_unit.available_ram_banks == 4 {
                        self.mem_unit.ram_bank = self.mem_unit.mbc1_2_bit_reg;
                    }
                    if self.mem_unit.rom_bank_bits > 5 {
                        self.mem_unit.rom_bank =
                            self.mem_unit.mbc1_5_bit_reg + (self.mem_unit.mbc1_2_bit_reg << 5);
                        if self.mem_unit.memory_mode == 1 {
                            self.mem_unit.mbc1_0_bank = self.mem_unit.mbc1_2_bit_reg << 5;
                        }
                    }
                }
                CartType::Mbc3 => {
                    if val < 0x4 {
                        self.mem_unit.memory_mode = 0;
                        self.mem_unit.ram_bank = val as usize & 0b11;
                    } else if val >= 0x8 {
                        self.mem_unit.memory_mode = 1;
                    }
                }
                CartType::Mbc5 => {
                    self.mem_unit.ram_bank = val as usize & 0xF;
                }
                _ => panic!("Bad cart type."),
            },
            0x6000..=0x7FFF => match self.mem_unit.cartridge_type {
                CartType::RomOnly => {}
                CartType::Mbc1 => {
                    self.mem_unit.memory_mode = val;
                    if val == 0 {
                        self.mem_unit.rom_bank = if self.mem_unit.rom_bank_bits > 5 {
                            self.mem_unit.mbc1_5_bit_reg + (self.mem_unit.mbc1_2_bit_reg << 5)
                        } else {
                            self.mem_unit.mbc1_5_bit_reg
                        };
                        self.mem_unit.ram_bank = 0;
                    } else {
                        if self.mem_unit.available_ram_banks == 4 {
                            self.mem_unit.ram_bank = self.mem_unit.mbc1_2_bit_reg;
                        } else {
                            self.mem_unit.ram_bank = 0;
                        }
                        if self.mem_unit.rom_bank_bits > 5 {
                            self.mem_unit.rom_bank =
                                self.mem_unit.mbc1_5_bit_reg + (self.mem_unit.mbc1_2_bit_reg << 5);
                            self.mem_unit.mbc1_0_bank = self.mem_unit.mbc1_2_bit_reg << 5;
                        } else {
                            self.mem_unit.rom_bank = self.mem_unit.mbc1_5_bit_reg;
                        }
                    }
                }
                CartType::Mbc3 => {}
                CartType::Mbc5 => {}
                _ => panic!("Bad cart type."),
            },
            0x8000..=0x9FFF => {
                if self.mem_unit.ppu_mode != DRAWING_MODE || source == RequestSource::PPU {
                    if self.mem_unit.vram_bank == 0 {
                        self.mem_unit.vram_0[addr - VRAM_START_ADDR] = val;
                    } else {
                        self.mem_unit.vram_1[addr - VRAM_START_ADDR] = val;
                    }
                }
            }
            0xA000..=0xBFFF => {
                match self.mem_unit.cartridge_type {
                    CartType::RomOnly => {}
                    CartType::Mbc1 | CartType::Mbc3 => {
                        if !(self.mem_unit.available_ram_banks == 0
                            || !self.mem_unit.ram_enable
                            || self.mem_unit.memory_mode == 1)
                        {
                            self.mem_unit.external_ram[addr - ERAM_START_ADDR
                                + ERAM_BANK_SIZE * self.mem_unit.ram_bank] = val;
                        }
                    }
                    CartType::Mbc2 => {
                        if self.mem_unit.ram_enable {
                            self.mem_unit.external_ram[(addr - ERAM_START_ADDR) % 0x200] = val;
                        }
                    }
                    CartType::Mbc5 => {
                        if self.mem_unit.ram_enable {
                            self.mem_unit.external_ram[addr - ERAM_START_ADDR
                                + ERAM_BANK_SIZE * self.mem_unit.ram_bank] = val;
                        }
                    }
                    _ => panic!("Bad cart type."),
                }
            }
            0xC000..=0xCFFF => self.mem_unit.internal_ram[addr - WRAM_START_ADDR] = val,
            0xD000..=0xDFFF => {
                self.mem_unit.internal_ram[addr - WRAM_START_ADDR - WRAM_BANK_SIZE
                    + WRAM_BANK_SIZE * self.mem_unit.wram_bank] = val;
            }
            0xE000..=0xFDFF => self.mem_unit.internal_ram[addr - 0xE000] = val,
            0xFE00..=0xFE9F => {
                if source == RequestSource::MAU
                    || self.mem_unit.ppu_mode == HBLANK_MODE
                    || self.mem_unit.ppu_mode == VBLANK_MODE
                {
                    self.mem_unit.oam[addr - OAM_START_ADDR] = val
                }
            }
            0xFF00 => {
                let mut p1 = self.get_memory(P1_ADDR, SOURCE);
                let prev_p1 = p1;
                p1 &= 0b001111;
                p1 |= val & 0b110000;
                let p14 = (p1 >> 4) & 1;
                let p15 = (p1 >> 5) & 1;
                let mut new_bits = 0xF;
                p1 &= 0b110000;

                if p14 == 0 {
                    new_bits &= self.mem_unit.directional_presses;
                }
                if p15 == 0 {
                    new_bits &= self.mem_unit.action_presses;
                }
                p1 += new_bits;
                if ((prev_p1 | p1) - p1) & 0xF != 0 {
                    self.write_memory(
                        INT_FLAG_ADDR,
                        self.get_memory(INT_FLAG_ADDR, SOURCE) | (1 << 4),
                        SOURCE,
                    );
                }
                self.mem_unit.io_registers[addr - IO_START_ADDR] = p1;
            }
            DIV_ADDR => {
                self.mem_unit.io_registers[DIV_ADDR - IO_START_ADDR] = 0;
            }
            0xFF10..=0xFF2F => {
                if addr == NR52_ADDR {
                    if source == RequestSource::CPU {
                        let old_power_val = self.mem_unit.io_registers[0x26] >> 7;
                        self.mem_unit.io_registers[NR52_ADDR - IO_START_ADDR] &= 0x7F;
                        if (val >> 7) == 0 {
                            for ind in 0x10..0x30_usize {
                                self.mem_unit.io_registers[ind] = 0;
                            }
                            self.apu.apu_power = false;
                            self.apu.all_sound_enable.store(false, Ordering::Relaxed);
                        } else if old_power_val == 0 {
                            self.apu_power_up();
                        }
                        self.mem_unit.io_registers[NR52_ADDR - IO_START_ADDR] |=
                            (val & 0x80) | 0x70;
                    } else {
                        self.mem_unit.io_registers[NR52_ADDR - IO_START_ADDR] = val;
                    }
                } else if (self.mem_unit.io_registers[NR52_ADDR - IO_START_ADDR] >> 7) == 1 {
                    if source != RequestSource::APU {
                        match addr {
                            0xFF10 => self.sweep_var_update(val),
                            0xFF11 => self.nrx1_write(1, val),
                            0xFF12 => self.vol_env_write(1, val),
                            0xFF13 => self.update_frequency_internal_low(1, val),
                            0xFF14 => self.nrx4_write(1, val),
                            0xFF16 => self.nrx1_write(2, val),
                            0xFF17 => self.vol_env_write(2, val),
                            0xFF18 => self.update_frequency_internal_low(2, val),
                            0xFF19 => self.nrx4_write(2, val),
                            0xFF1A => {
                                if (val >> 7) == 0 {
                                    self.disable_channel(3)
                                }
                            }
                            0xFF1B => self.nrx1_write(3, val),
                            0xFF1C => self.apu.channel_3_output_level.store(
                                VOLUME_SHIFT_CONVERSION[(val as usize >> 5) & 0x3],
                                Ordering::Relaxed,
                            ),
                            0xFF1D => self.update_frequency_internal_low(3, val),
                            0xFF1E => self.nrx4_write(3, val),
                            0xFF20 => self.nrx1_write(4, val),
                            0xFF21 => self.vol_env_write(4, val),
                            0xFF22 => self.poly_count_var_update(val),
                            0xFF23 => self.nrx4_write(4, val),
                            0xFF24 => self.nr50_write(val),
                            0xFF25 => self.nr51_write(val),
                            _ => {}
                        }
                    }

                    self.mem_unit.io_registers[addr - IO_START_ADDR] = val;
                }
            }
            0xFF30..=0xFF3F => {
                if source == RequestSource::CPU {
                    self.wave_ram_write(addr, val);
                } else {
                    self.mem_unit.io_registers[addr - IO_START_ADDR] = val;
                }
            }
            STAT_ADDR => {
                if source == RequestSource::PPU {
                    self.mem_unit.io_registers[STAT_ADDR - IO_START_ADDR] = val;
                } else {
                    self.mem_unit.io_registers[STAT_ADDR - IO_START_ADDR] &= 0b0000111;
                    self.mem_unit.io_registers[STAT_ADDR - IO_START_ADDR] |= val & 0b1111000
                }
            }
            LY_ADDR => {
                if source == RequestSource::PPU {
                    self.mem_unit.io_registers[LY_ADDR - IO_START_ADDR] = val;
                } else {
                    panic!("LY WRITE");
                }
            }
            LYC_ADDR => {
                self.mem_unit.io_registers[LYC_ADDR - IO_START_ADDR] = val;
                self.check_lyc_flag();
            }
            DMA_ADDR => {
                self.mem_unit.io_registers[addr - IO_START_ADDR] = val;
                self.dma_transfer(val as usize);
            }
            KEY1_ADDR => {
                if self.mem_unit.cgb {
                    if source != RequestSource::SPEC {
                        self.mem_unit.io_registers[KEY1_ADDR - IO_START_ADDR] &= 0xFE;
                        self.mem_unit.io_registers[KEY1_ADDR - IO_START_ADDR] |= val & 1;
                    } else {
                        self.mem_unit.io_registers[KEY1_ADDR - IO_START_ADDR] = val;
                    }
                }
            }
            VBK_ADDR => {
                if self.mem_unit.cgb {
                    self.mem_unit.vram_bank = val & 1;
                    self.mem_unit.io_registers[VBK_ADDR - IO_START_ADDR] = 0b11111110 | (val & 1);
                }
            }
            0xFF50 => {
                if self.mem_unit.in_boot_rom {
                    self.unload_boot_rom();
                    self.mem_unit.in_boot_rom = false;
                }
            }
            HDMA2_ADDR => {
                self.mem_unit.io_registers[addr - IO_START_ADDR] = val | 0xF;
            }
            HDMA3_ADDR => {
                self.mem_unit.io_registers[addr - IO_START_ADDR] = val | 0xE0;
            }
            HDMA4_ADDR => {
                self.mem_unit.io_registers[addr - IO_START_ADDR] = val | 0xF;
            }
            HDMA5_ADDR => {
                if self.mem_unit.cgb {
                    if self.mem_unit.hdma_active {
                        if (val >> 7) == 0 {
                            self.mem_unit.hdma_active = false;
                            self.mem_unit.io_registers[HDMA5_ADDR - IO_START_ADDR] |= 0x80;
                        }
                    } else {
                        self.mem_unit.io_registers[HDMA5_ADDR - IO_START_ADDR] = val;
                        self.hdma_transfer();
                    }
                }
            }
            BCPS_ADDR => {
                self.mem_unit.bg_color_inc = (val >> 7) == 1;
                self.mem_unit.io_registers[BCPD_ADDR - IO_START_ADDR] = val & 0x3F;
                self.mem_unit.io_registers[BCPS_ADDR - IO_START_ADDR] = val;
            }
            BCPD_ADDR => {
                self.mem_unit.bg_color_ram
                    [self.mem_unit.io_registers[BCPD_ADDR - IO_START_ADDR] as usize] = val;
                if self.mem_unit.bg_color_inc {
                    self.mem_unit.io_registers[BCPD_ADDR - IO_START_ADDR] =
                        (self.mem_unit.io_registers[BCPD_ADDR - IO_START_ADDR] + 1) % 64;
                }
            }
            OCPS_ADDR => {
                self.mem_unit.obj_color_inc = (val >> 7) == 1;
                self.mem_unit.io_registers[OCPD_ADDR - IO_START_ADDR] = val & 0x3F;
                self.mem_unit.io_registers[OCPS_ADDR - IO_START_ADDR] = val;
            }
            OCPD_ADDR => {
                self.mem_unit.obj_color_ram
                    [self.mem_unit.io_registers[OCPD_ADDR - IO_START_ADDR] as usize] = val;
                if self.mem_unit.obj_color_inc {
                    self.mem_unit.io_registers[OCPD_ADDR - IO_START_ADDR] =
                        (self.mem_unit.io_registers[OCPD_ADDR - IO_START_ADDR] + 1) % 64;
                }
            }
            SVBK_ADDR => {
                if self.mem_unit.cgb {
                    self.mem_unit.wram_bank = (val & 0b111) as usize;
                    if self.mem_unit.wram_bank == 0 {
                        self.mem_unit.wram_bank += 1;
                    }
                    self.mem_unit.io_registers[SVBK_ADDR - IO_START_ADDR] =
                        0b11111000 | self.mem_unit.wram_bank as u8;
                }
            }
            0xFF00..=0xFF7F => self.mem_unit.io_registers[addr - IO_START_ADDR] = val,
            0xFF80..=0xFFFE => self.mem_unit.high_ram[addr - HRAM_START_ADDR] = val,
            INT_ENABLE_ADDR => self.mem_unit.interrupt_enable = val,
            _ => {
                println!("bad general write!")
            }
        }
    }

    fn dma_transfer(&mut self, reg: usize) {
        let start_address = reg << 8;
        match reg >> 4 {
            0x0..=0x3 => {
                let end_address = (start_address + DMA_LENGTH) as usize;
                self.mem_unit
                    .oam
                    .copy_from_slice(&self.mem_unit.rom[start_address..end_address]);
            }
            0x4..=0x7 => {
                let adjusted_start_address =
                    start_address - ROM_BANK_SIZE + ROM_BANK_SIZE * self.mem_unit.rom_bank;
                let adjusted_end_address = adjusted_start_address + DMA_LENGTH;
                self.mem_unit.oam.copy_from_slice(
                    &self.mem_unit.rom[adjusted_start_address..adjusted_end_address],
                );
            }
            0x8..=0x9 => {
                let adjusted_start_address = start_address - VRAM_START_ADDR;
                let adjusted_end_address = adjusted_start_address + DMA_LENGTH;
                if self.mem_unit.vram_bank == 0 {
                    self.mem_unit.oam.copy_from_slice(
                        &self.mem_unit.vram_0[adjusted_start_address..adjusted_end_address],
                    );
                } else {
                    self.mem_unit.oam.copy_from_slice(
                        &self.mem_unit.vram_1[adjusted_start_address..adjusted_end_address],
                    );
                }
            }
            0xA..=0xB => {
                let adjusted_start_address =
                    start_address - ERAM_START_ADDR + ERAM_BANK_SIZE * (self.mem_unit.ram_bank);
                let adjusted_end_address = adjusted_start_address + DMA_LENGTH;
                self.mem_unit.oam.copy_from_slice(
                    &self.mem_unit.external_ram[adjusted_start_address..adjusted_end_address],
                );
            }
            0xC => {
                let adjusted_start_address = (start_address - WRAM_START_ADDR) as usize;
                let adjusted_end_address = adjusted_start_address + DMA_LENGTH;
                self.mem_unit.oam.copy_from_slice(
                    &self.mem_unit.internal_ram[adjusted_start_address..adjusted_end_address],
                );
            }
            0xD => {
                let adjusted_start_address = (start_address - WRAM_START_ADDR - WRAM_BANK_SIZE
                    + WRAM_BANK_SIZE * self.mem_unit.wram_bank)
                    as usize;
                let adjusted_end_address = adjusted_start_address + DMA_LENGTH;
                self.mem_unit.oam.copy_from_slice(
                    &self.mem_unit.internal_ram[adjusted_start_address..adjusted_end_address],
                );
            }
            _ => println!("DMA FAILURE"),
        }
        self.mem_unit.dma_cycles = 640;
    }
    fn hdma_transfer(&mut self) {
        let hdma5 = self.get_memory(HDMA5_ADDR, SOURCE);
        self.mem_unit.hdma_current_source_addr = combine_bytes(
            self.get_memory(HDMA1_ADDR, SOURCE),
            self.get_memory(HDMA2_ADDR, SOURCE),
        ) as usize
            & 0xFFF0;
        self.mem_unit.hdma_current_dest_addr = (combine_bytes(
            self.get_memory(HDMA3_ADDR, SOURCE),
            self.get_memory(HDMA4_ADDR, SOURCE),
        ) as usize
            & 0x1FF0)
            | 0x8000;
        self.mem_unit.hdma_blocks = (hdma5 & 0x7F) + 1;
        let gen_dma = (hdma5 >> 7) == 0;
        if gen_dma {
            for _ in 0..self.mem_unit.hdma_blocks {
                self.hdma_block_transfer()
            }
        } else {
            self.mem_unit.hdma_active = true;
            self.mem_unit.hdma_primed = true;
            self.mem_unit.io_registers[HDMA5_ADDR - IO_START_ADDR] &= 0x7F;
        }
    }
    pub fn hdma_block_transfer(&mut self) {
        let high_nibble_source = self.mem_unit.hdma_current_source_addr >> 12;
        let copy_data = match high_nibble_source {
            0x0..=0x3 => {
                &self.mem_unit.rom[self.mem_unit.hdma_current_source_addr
                    ..(self.mem_unit.hdma_current_source_addr + HDMA_BLOCK_LENGTH)]
            }
            0x4..=0x7 => {
                let adjusted_address = self.mem_unit.hdma_current_source_addr
                    + ROM_BANK_SIZE * self.mem_unit.rom_bank
                    - ROM_BANK_SIZE;
                &self.mem_unit.rom[adjusted_address..(adjusted_address + HDMA_BLOCK_LENGTH)]
            }
            0xA..=0xB => {
                let adjusted_address = self.mem_unit.hdma_current_source_addr - ERAM_START_ADDR
                    + ERAM_BANK_SIZE * self.mem_unit.ram_bank;
                &self.mem_unit.external_ram
                    [adjusted_address..(adjusted_address + HDMA_BLOCK_LENGTH)]
            }
            0xC => {
                let adjusted_address = self.mem_unit.hdma_current_source_addr - WRAM_START_ADDR;
                &self.mem_unit.internal_ram
                    [adjusted_address..(adjusted_address + HDMA_BLOCK_LENGTH)]
            }
            0xD => {
                let adjusted_address =
                    self.mem_unit.hdma_current_source_addr - WRAM_START_ADDR - WRAM_BANK_SIZE
                        + WRAM_BANK_SIZE * self.mem_unit.wram_bank;
                &self.mem_unit.internal_ram
                    [adjusted_address..(adjusted_address + HDMA_BLOCK_LENGTH)]
            }
            _ => {
                panic!("HDMA BAD ADDRESS");
            }
        };
        let adjusted_dest_address = self.mem_unit.hdma_current_dest_addr - VRAM_START_ADDR;
        if self.mem_unit.vram_bank == 0 {
            self.mem_unit.vram_0
                [adjusted_dest_address..(adjusted_dest_address + HDMA_BLOCK_LENGTH)]
                .copy_from_slice(copy_data);
        }
        if self.mem_unit.vram_bank == 1 {
            self.mem_unit.vram_1
                [adjusted_dest_address..(adjusted_dest_address + HDMA_BLOCK_LENGTH)]
                .copy_from_slice(copy_data);
        }
        self.mem_unit.hdma_current_source_addr += HDMA_BLOCK_LENGTH;
        self.mem_unit.hdma_current_dest_addr += HDMA_BLOCK_LENGTH;
        let (source_high, source_low) = split_u16(self.mem_unit.hdma_current_source_addr as u16);
        self.write_memory(HDMA1_ADDR, source_high, SOURCE);
        self.write_memory(HDMA2_ADDR, source_low, SOURCE);

        let (dest_high, dest_low) = split_u16(self.mem_unit.hdma_current_dest_addr as u16);
        self.write_memory(HDMA3_ADDR, dest_high, SOURCE);
        self.write_memory(HDMA4_ADDR, dest_low, SOURCE);
        self.mem_unit.hdma_blocks -= 1;
        self.mem_unit.io_registers[HDMA5_ADDR - IO_START_ADDR] = self.mem_unit.hdma_blocks;
        self.cpu.waiting = true;
        self.cpu.cycle_goal += if self.double_speed { 64 } else { 32 };
        if self.mem_unit.hdma_blocks == 0 {
            self.mem_unit.hdma_active = false;
            self.mem_unit.io_registers[HDMA5_ADDR - IO_START_ADDR] = 0xFF;
        }
    }
    pub fn dma_tick(&mut self) {
        if self.mem_unit.dma_cycles > 0 {
            self.mem_unit.dma_cycles -= ADVANCE_CYCLES;
        }
        if self.mem_unit.hdma_active {
            if self.mem_unit.hdma_primed && self.mem_unit.ppu_mode == HBLANK_MODE {
                self.hdma_block_transfer();
                self.mem_unit.hdma_primed = false;
            } else if !self.mem_unit.hdma_primed && self.mem_unit.ppu_mode != HBLANK_MODE {
                self.mem_unit.hdma_primed = true;
            }
        }
    }
    pub fn access_vram(&self, addr: impl Into<usize>, bank: u8) -> u8 {
        let addr = addr.into() as usize;
        if bank == 0 {
            self.mem_unit.vram_0[addr - VRAM_START_ADDR]
        } else {
            self.mem_unit.vram_1[addr - VRAM_START_ADDR]
        }
    }
    pub fn get_bg_rbg(&self, palette: u8) -> [[u8; 4]; 4] {
        let mut out = [[0; 4]; 4];
        let mut index = (palette * 8) as usize;
        for i in 0..4 {
            let color_data = self.mem_unit.bg_color_ram[index] as u16
                + ((self.mem_unit.bg_color_ram[index + 1] as u16) << 8);
            let red = (color_data & 0x1F) as u8;
            let green = ((color_data >> 5) & 0x1F) as u8;
            let blue = ((color_data >> 10) & 0x1F) as u8;
            index += 2;
            out[i] = [
                convert_to_8_bit(red),
                convert_to_8_bit(green),
                convert_to_8_bit(blue),
                0xFF,
            ]
        }
        out
    }
    pub fn get_obj_rbg(&self, palette: u8) -> [[u8; 4]; 4] {
        let mut out = [[0; 4]; 4];
        let mut index = (palette * 8) as usize;
        for i in 0..4 {
            let color_data = self.mem_unit.obj_color_ram[index] as u16
                + ((self.mem_unit.obj_color_ram[index + 1] as u16) << 8);
            let red = (color_data & 0x1F) as u8;
            let green = ((color_data >> 5) & 0x1F) as u8;
            let blue = ((color_data >> 10) & 0x1F) as u8;
            index += 2;
            out[i] = [
                convert_to_8_bit(red),
                convert_to_8_bit(green),
                convert_to_8_bit(blue),
                0xFF,
            ]
        }
        out
    }
    fn memory_initialize_after_boot(&mut self) {
        self.mem_unit.io_registers[P1_ADDR - IO_START_ADDR] = 0xCF;
        self.mem_unit.io_registers[TIMA_ADDR - IO_START_ADDR] = 0x00;
        self.mem_unit.io_registers[TMA_ADDR - IO_START_ADDR] = 0x00;
        self.mem_unit.io_registers[TAC_ADDR - IO_START_ADDR] = 0xF8;
        self.mem_unit.io_registers[INT_FLAG_ADDR - IO_START_ADDR] = 0xE1;
        self.mem_unit.io_registers[LCDC_ADDR - IO_START_ADDR] = 0x91;
        self.mem_unit.io_registers[SCX_ADDR - IO_START_ADDR] = 0x00;
        self.mem_unit.io_registers[SCY_ADDR - IO_START_ADDR] = 0x00;
        self.mem_unit.io_registers[LYC_ADDR - IO_START_ADDR] = 0x00;
        self.mem_unit.io_registers[BGP_ADDR - IO_START_ADDR] = 0xFC;
        self.mem_unit.io_registers[WY_ADDR - IO_START_ADDR] = 0x00;
        self.mem_unit.io_registers[WX_ADDR - IO_START_ADDR] = 0x00;
        self.mem_unit.io_registers[KEY1_ADDR - IO_START_ADDR] = 0;
        self.mem_unit.io_registers[HDMA1_ADDR - IO_START_ADDR] = 0xFF;
        self.mem_unit.io_registers[HDMA2_ADDR - IO_START_ADDR] = 0xFF;
        self.mem_unit.io_registers[HDMA3_ADDR - IO_START_ADDR] = 0xFF;
        self.mem_unit.io_registers[HDMA4_ADDR - IO_START_ADDR] = 0xFF;
        self.mem_unit.io_registers[HDMA5_ADDR - IO_START_ADDR] = 0xFF;

        self.mem_unit.io_registers[BGP_ADDR - IO_START_ADDR] = 0xFC;
        self.mem_unit.bg_color_ram.copy_from_slice(&[255; 64]);
        self.write_memory(SVBK_ADDR, 1, SOURCE);
        self.mem_unit.interrupt_enable = 0;
    }
    fn load_boot_rom(&mut self) {
        self.mem_unit.hold_mem[..0x100].copy_from_slice(&self.mem_unit.rom[..0x100]);
        if self.mem_unit.cgb {
            self.mem_unit.hold_mem[0x100..0x800].copy_from_slice(&self.mem_unit.rom[0x200..0x900]);
            self.mem_unit.rom[..0x100].copy_from_slice(&CGB_BOOTROM_1);
            self.mem_unit.rom[0x200..0x900].copy_from_slice(&CGB_BOOTROM_2);
        } else {
            self.mem_unit.rom[..0x100].copy_from_slice(&DMG_BOOTROM);
        }
    }
    fn unload_boot_rom(&mut self) {
        self.mem_unit.rom[..0x100].copy_from_slice(&self.mem_unit.hold_mem[..0x100]);
        if self.mem_unit.cgb {
            self.mem_unit.rom[0x200..0x900].copy_from_slice(&self.mem_unit.hold_mem[0x100..0x800]);
        }
        self.cpu_initialize_after_boot();
        self.memory_initialize_after_boot();
    }

    pub fn load_rom(&mut self, path: &Path) {
        let mut f = File::open(path).expect("File problem!");
        f.read_to_end(&mut self.mem_unit.rom).expect("Read issue!");
        self.mem_unit.cartridge_type = match self.mem_unit.rom[CART_TYPE_ADDR] {
            0 => CartType::RomOnly,
            1..=3 => CartType::Mbc1,
            5..=6 => CartType::Mbc2,
            0xF..=0x13 => CartType::Mbc3,
            0x19..=0x1E => CartType::Mbc5,
            _ => panic!("Bad cart type."),
        };
        self.mem_unit.available_rom_banks = 1usize << (self.get_memory(ROM_BANK_ADDR, SOURCE) + 1);
        self.mem_unit.available_ram_banks = match self.get_memory(RAM_BANK_ADDR, SOURCE) {
            0 => 0,
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            _ => 0,
        };
        self.mem_unit
            .external_ram
            .extend(vec![0; 0x2000 * self.mem_unit.available_ram_banks as usize]);
        self.mem_unit.rom_bank_bits = (self.get_memory(ROM_BANK_ADDR, SOURCE) + 1) as usize;
        self.cgb = (self.mem_unit.rom[0x143] >> 7) == 1;
        self.mem_unit.cgb = self.cgb;
        if self.cgb {
            for ind in NON_BLOCK_CGB_VALID_IO.iter() {
                self.mem_unit.valid_io[*ind] = true;
            }
        }
        self.load_boot_rom();
    }

    pub fn save_game(&self, path: &Path) {
        let save_file = File::create(&path).unwrap();
        let save_data = SaveGame {
            vram_0: self.mem_unit.vram_0.clone(),
            vram_1: self.mem_unit.vram_1.clone(),
            external_ram: self.mem_unit.external_ram.clone(),
            internal_ram: self.mem_unit.internal_ram.clone(),
            oam: self.mem_unit.oam.clone(),
            io_registers: self.mem_unit.io_registers.clone(),
            high_ram: self.mem_unit.high_ram.clone(),
            interrupt_enable: self.mem_unit.interrupt_enable,
            memory_mode: self.mem_unit.memory_mode,
            rom_bank: self.mem_unit.rom_bank,
            ram_bank: self.mem_unit.ram_bank,
            mbc1_0_bank: self.mem_unit.mbc1_0_bank,
            mbc1_5_bit_reg: self.mem_unit.mbc1_5_bit_reg,
            mbc1_2_bit_reg: self.mem_unit.mbc1_2_bit_reg,
            rom_bank_bits: self.mem_unit.rom_bank_bits,
            ram_enable: self.mem_unit.ram_enable,
            cartridge_type: self.mem_unit.cartridge_type,
            available_rom_banks: self.mem_unit.available_rom_banks,
            available_ram_banks: self.mem_unit.available_ram_banks,
            hold_mem: self.mem_unit.hold_mem.clone(),
            in_boot_rom: self.mem_unit.in_boot_rom,
            directional_presses: self.mem_unit.directional_presses,
            action_presses: self.mem_unit.action_presses,
            dma_cycles: self.mem_unit.dma_cycles,
            ppu_mode: self.mem_unit.ppu_mode,
            bg_color_ram: self.mem_unit.bg_color_ram.clone(),
            obj_color_ram: self.mem_unit.obj_color_ram.clone(),
            bg_color_inc: self.mem_unit.bg_color_inc,
            obj_color_inc: self.mem_unit.obj_color_inc,
            vram_bank: self.mem_unit.vram_bank,
            wram_bank: self.mem_unit.wram_bank,
            cgb: self.mem_unit.cgb,
            hdma_primed: self.mem_unit.hdma_primed,
            hdma_blocks: self.mem_unit.hdma_blocks,
            hdma_active: self.mem_unit.hdma_active,
            hdma_current_dest_addr: self.mem_unit.hdma_current_dest_addr,
            hdma_current_source_addr: self.mem_unit.hdma_current_source_addr,
            valid_io: self.mem_unit.valid_io.clone(),
            cpu: self.cpu,
            ppu: self.ppu.clone(),
            timer: self.timer,
        };
        bincode::serialize_into(save_file, &save_data).unwrap();
    }
    pub fn open_game(&mut self, path: &Path) {
        let open_file = File::open(&path).unwrap();
        let open_data: SaveGame = bincode::deserialize_from(open_file).unwrap();
        self.mem_unit.vram_0 = open_data.vram_0;
        self.mem_unit.vram_1 = open_data.vram_1;
        self.mem_unit.external_ram = open_data.external_ram;
        self.mem_unit.internal_ram = open_data.internal_ram;
        self.mem_unit.oam = open_data.oam;
        self.mem_unit.io_registers = open_data.io_registers;
        self.mem_unit.high_ram = open_data.high_ram;
        self.mem_unit.interrupt_enable = open_data.interrupt_enable;
        self.mem_unit.memory_mode = open_data.memory_mode;
        self.mem_unit.rom_bank = open_data.rom_bank;
        self.mem_unit.ram_bank = open_data.ram_bank;
        self.mem_unit.mbc1_0_bank = open_data.mbc1_0_bank;
        self.mem_unit.mbc1_5_bit_reg = open_data.mbc1_5_bit_reg;
        self.mem_unit.mbc1_2_bit_reg = open_data.mbc1_2_bit_reg;
        self.mem_unit.rom_bank_bits = open_data.rom_bank_bits;
        self.mem_unit.ram_enable = open_data.ram_enable;
        self.mem_unit.cartridge_type = open_data.cartridge_type;
        self.mem_unit.available_rom_banks = open_data.available_rom_banks;
        self.mem_unit.available_ram_banks = open_data.available_ram_banks;
        self.mem_unit.hold_mem = open_data.hold_mem;
        self.mem_unit.in_boot_rom = open_data.in_boot_rom;
        self.mem_unit.directional_presses = open_data.directional_presses;
        self.mem_unit.action_presses = open_data.action_presses;
        self.mem_unit.dma_cycles = open_data.dma_cycles;
        self.mem_unit.ppu_mode = open_data.ppu_mode;
        self.mem_unit.bg_color_ram = open_data.bg_color_ram;
        self.mem_unit.obj_color_ram = open_data.obj_color_ram;
        self.mem_unit.bg_color_inc = open_data.bg_color_inc;
        self.mem_unit.obj_color_inc = open_data.obj_color_inc;
        self.mem_unit.vram_bank = open_data.vram_bank;
        self.mem_unit.wram_bank = open_data.wram_bank;
        self.mem_unit.cgb = open_data.cgb;
        self.mem_unit.hdma_primed = open_data.hdma_primed;
        self.mem_unit.hdma_blocks = open_data.hdma_blocks;
        self.mem_unit.hdma_active = open_data.hdma_active;
        self.mem_unit.hdma_current_dest_addr = open_data.hdma_current_dest_addr;
        self.mem_unit.hdma_current_source_addr = open_data.hdma_current_source_addr;
        self.mem_unit.valid_io = open_data.valid_io;
        self.cpu = open_data.cpu;
        self.ppu = open_data.ppu;
        self.timer = open_data.timer;
    }
}

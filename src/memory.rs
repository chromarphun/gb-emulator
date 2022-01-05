use crate::emulator::GameBoyEmulator;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::TryInto;
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
    vram: Vec<u8>,
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
    ram_enable: bool,
    hold_mem: Vec<u8>,
    pub in_boot_rom: bool,
    dma_cycles: u16,
    cpu: CentralProcessingUnit,
    ppu: PictureProcessingUnit,
    timer: Timer,
}

pub struct MemoryUnit {
    pub rom: Vec<u8>,
    vram_0: Vec<u8>,
    vram_1: Vec<u8>,
    external_ram: Vec<u8>,
    internal_ram: Vec<u8>,
    oam: Vec<u8>,
    pub io_registers: Vec<u8>,
    high_ram: Vec<u8>,
    pub interrupt_enable: u8,
    memory_mode: u8,
    pub rom_bank: usize,
    pub ram_bank: usize,
    mbc1_0_bank: usize,
    mbc1_5_bit_reg: usize,
    mbc1_2_bit_reg: usize,
    rom_bank_bits: usize,
    ram_enable: bool,
    cartridge_type: CartType,
    available_rom_banks: usize,
    available_ram_banks: u8,
    hold_mem: Vec<u8>,
    pub in_boot_rom: bool,
    pub directional_presses: u8,
    pub action_presses: u8,
    dma_cycles: u16,
    pub ppu_mode: u8,
    bg_color_ram: [u8; 64],
    obj_color_ram: [u8; 64],
    bg_color_inc: bool,
    obj_color_inc: bool,
    vram_bank: u8,
    pub wram_bank: usize,
    cgb: bool,
    hdma_primed: bool,
    hdma_blocks: u8,
    hdma_active: bool,
    hdma_current_dest_addr: usize,
    hdma_current_source_addr: usize,
    valid_io: [bool; 0x80],
    pub cpu_initialize: bool,
    debug_var: HashSet<usize>,
}

impl MemoryUnit {
    pub fn new() -> MemoryUnit {
        let rom = Vec::new();
        let vram_0 = vec![0; VRAM_SIZE];
        let vram_1 = vec![0; VRAM_SIZE];
        let external_ram = Vec::new();
        let internal_ram = vec![0; IRAM_SIZE];
        let oam = vec![0; OAM_SIZE];
        let io_registers = vec![0; IO_SIZE];
        let memory_mode = 0;
        let rom_bank = 1;
        let ram_bank = 0;
        let rom_bank_bits = 0;
        let ram_enable = false;
        let cartridge_type = CartType::Uninitialized;
        let available_rom_banks = 0;
        let available_ram_banks = 0;
        let hold_mem = vec![0; 2048];
        let interrupt_enable = 0;
        let high_ram = vec![0; HRAM_SIZE];
        let in_boot_rom = true;
        let directional_presses = 0xF;
        let action_presses = 0xF;
        let dma_cycles = 0;
        let mut valid_io = [true; 0x80];
        for ind in NON_BLOCK_INVALID_IO.iter() {
            valid_io[*ind] = false;
        }
        valid_io[0x4C..0x80].copy_from_slice(&[false; (0x80 - 0x4C)]);
        MemoryUnit {
            rom,
            vram_0,
            vram_1,
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
            ppu_mode: 0,
            bg_color_ram: [0; 64],
            obj_color_ram: [0; 64],
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
            cpu_initialize: false,
            debug_var: HashSet::new(),
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
                CartType::Mbc1 => self.mem_unit.rom[self.mem_unit.mbc1_0_bank * 0x4000 + addr],
                _ => panic!("Bad cart type."),
            },
            0x4000..=0x7FFF => match self.mem_unit.cartridge_type {
                CartType::RomOnly => self.mem_unit.rom[addr],
                CartType::Mbc1 | CartType::Mbc2 | CartType::Mbc3 | CartType::Mbc5 => {
                    self.mem_unit.rom[addr + 0x4000 * self.mem_unit.rom_bank - 0x4000]
                }
                _ => panic!("Bad cart type."),
            },
            0x8000..=0x9FFF => {
                if self.mem_unit.ppu_mode != 3 || source == RequestSource::PPU {
                    if self.mem_unit.vram_bank == 0 {
                        self.mem_unit.vram_0[addr - 0x8000]
                    } else {
                        self.mem_unit.vram_1[addr - 0x8000]
                    }
                } else {
                    panic!("breaking on vram reading");
                    //println!("bad vram read!");
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
                        self.mem_unit.external_ram[addr - 0xA000 + 0x2000 * self.mem_unit.ram_bank]
                    }
                }
                CartType::Mbc2 => {
                    if !self.mem_unit.ram_enable {
                        0xFF
                    } else {
                        self.mem_unit.external_ram[(addr - 0xA000) % 0x200] & 0xF
                    }
                }
                CartType::Mbc5 => {
                    if !self.mem_unit.ram_enable {
                        0xFF
                    } else {
                        self.mem_unit.external_ram[addr - 0xA000 + 0x2000 * self.mem_unit.ram_bank]
                    }
                }
                _ => panic!("Bad cart type."),
            },
            0xC000..=0xCFFF => self.mem_unit.internal_ram[addr - 0xC000],
            0xD000..=0xDFFF => {
                self.mem_unit.internal_ram[addr - 0xD000 + 0x1000 * self.mem_unit.wram_bank]
            }
            0xE000..=0xFDFF => self.mem_unit.internal_ram[addr - 0xE000],
            0xFE00..=0xFE9F => {
                if source == RequestSource::MAU
                    || source == RequestSource::PPU
                    || self.mem_unit.ppu_mode == 0
                    || self.mem_unit.ppu_mode == 1
                {
                    self.mem_unit.oam[addr - 0xFE00]
                } else {
                    0xFF
                }
            }
            0xFF69 => self.mem_unit.bg_color_ram[self.mem_unit.io_registers[0x69] as usize],
            0xFF6B => self.mem_unit.obj_color_ram[self.mem_unit.io_registers[0x6B] as usize],
            0xFF10 => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[0x10] | 0x80
                } else {
                    self.mem_unit.io_registers[0x10]
                }
            }
            0xFF11 => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[0x11] | 0x3F
                } else {
                    self.mem_unit.io_registers[0x11]
                }
            }
            0xFF14 => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[0x14] | 0xBF
                } else {
                    self.mem_unit.io_registers[0x14]
                }
            }
            0xFF16 => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[0x16] | 0x3F
                } else {
                    self.mem_unit.io_registers[0x16]
                }
            }
            0xFF19 => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[0x19] | 0xBF
                } else {
                    self.mem_unit.io_registers[0x19]
                }
            }
            0xFF1A => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[0x1A] | 0x7F
                } else {
                    self.mem_unit.io_registers[0x1A]
                }
            }
            0xFF1C => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[0x1C] | 0x9F
                } else {
                    self.mem_unit.io_registers[0x1C]
                }
            }
            0xFF1E => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[0x1E] | 0xBF
                } else {
                    self.mem_unit.io_registers[0x1E]
                }
            }
            0xFF23 => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[0x23] | 0xBF
                } else {
                    self.mem_unit.io_registers[0x23]
                }
            }
            0xFF26 => {
                if source == RequestSource::CPU {
                    self.mem_unit.io_registers[0x26] | 0x70
                } else {
                    self.mem_unit.io_registers[0x26]
                }
            }
            0xFF30..=0xFF3F => {
                if source == RequestSource::CPU {
                    self.wave_ram_read(addr)
                } else {
                    self.mem_unit.io_registers[addr - 0xFF00]
                }
            }
            0xFF00..=0xFF7F => {
                if self.mem_unit.valid_io[addr - 0xFF00] || source != RequestSource::CPU {
                    self.mem_unit.io_registers[addr - 0xFF00]
                } else {
                    println!("{}", format!("hitting invalid IO at {:X}", addr));
                    0xFF
                }
            }
            0xFF80..=0xFFFE => self.mem_unit.high_ram[addr - 0xFF80],
            0xFFFF => self.mem_unit.interrupt_enable,
            _ => {
                println!("bad general read!");
                0xFF
            }
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
                    //if val & MASKING_BITS[self.mem_unit.rom_bank_bits] as u8 == 0 {
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
                    println!(
                        "rom bank now {} at iter {}",
                        self.mem_unit.rom_bank, self.iteration_count
                    );
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
                    //println!("changing rom bank to {}", val);
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
                    // println!(
                    //     "{}",
                    //     format!(
                    //         "setting to rom bank with val {:#04X} using addr {:#06X}, making it rom_bank {:#04X}",
                    //         val, addr, self.mem_unit.rom_bank
                    //     )
                    // );
                    //println!("rom bank is now {}", self.mem_unit.rom_bank);
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
                    // println!(
                    //     "rom bank now {} at iter {}",
                    //     self.mem_unit.rom_bank, self.iteration_count
                    // );
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
                CartType::Mbc3 | CartType::Mbc5 => {
                    println!("SHOULDN'T BE WRITING HERE?");
                }
                _ => panic!("Bad cart type."),
            },
            0x8000..=0x9FFF => {
                if self.mem_unit.ppu_mode != 3 || source == RequestSource::PPU {
                    if self.mem_unit.vram_bank == 0 {
                        self.mem_unit.vram_0[addr - 0x8000] = val;
                    } else {
                        self.mem_unit.vram_1[addr - 0x8000] = val;
                    }
                } else {
                    // println!(
                    //     "{}",
                    //     format!("bad vram write at {:X} from {:?}", addr, source)
                    // );
                    // self.log
                    //     .write(format!("bad vram write at {:X} from {:?}", addr, source).as_bytes())
                    //     .expect("WRITE FAILURE");
                    //panic!("panicking at vram write!!!");
                }
            }
            0xA000..=0xBFFF => match self.mem_unit.cartridge_type {
                CartType::RomOnly => {}
                CartType::Mbc1 | CartType::Mbc3 => {
                    if !(self.mem_unit.available_ram_banks == 0
                        || !self.mem_unit.ram_enable
                        || self.mem_unit.memory_mode == 1)
                    {
                        self.mem_unit.external_ram
                            [addr - 0xA000 + 0x2000 * self.mem_unit.ram_bank] = val;
                    }
                }
                CartType::Mbc2 => {
                    if self.mem_unit.ram_enable {
                        self.mem_unit.external_ram[(addr - 0xA000) % 0x200] = val;
                    }
                }
                CartType::Mbc5 => {
                    if self.mem_unit.ram_enable {
                        self.mem_unit.external_ram
                            [addr - 0xA000 + 0x2000 * self.mem_unit.ram_bank] = val;
                    }
                }
                _ => panic!("Bad cart type."),
            },
            0xC000..=0xCFFF => self.mem_unit.internal_ram[addr - 0xC000] = val,
            0xD000..=0xDFFF => {
                self.mem_unit.internal_ram[addr - 0xD000 + 0x1000 * self.mem_unit.wram_bank] = val;
            }
            0xE000..=0xFDFF => {
                println!("writing in weird mirror area");
                self.mem_unit.internal_ram[addr - 0xE000] = val
            }
            0xFE00..=0xFE9F => {
                if source == RequestSource::MAU
                    || self.mem_unit.ppu_mode == 0
                    || self.mem_unit.ppu_mode == 1
                {
                    self.mem_unit.oam[addr - 0xFE00] = val
                } else {
                    println!("Bad OAM write!");
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
                self.mem_unit.io_registers[addr - 0xFF00] = p1;
            }
            0xFF01 => {
                println!("{}", format!("out: {:X}", val));
                self.mem_unit.io_registers[0x01] = val;
            }
            0xFF04 => {
                self.mem_unit.io_registers[0x04] = 0;
            }
            0xFF10..=0xFF2F => {
                if addr == 0xFF26 {
                    if source == RequestSource::CPU {
                        let old_power_val = self.mem_unit.io_registers[0x26] >> 7;
                        self.mem_unit.io_registers[0x26] &= 0x7F;
                        if (val >> 7) == 0 {
                            println!("apu power down!");
                            for ind in 0x10..0x30_usize {
                                self.mem_unit.io_registers[ind] = 0;
                            }
                            self.apu.apu_power = false;
                            self.apu.all_sound_enable.store(false, Ordering::Relaxed);
                        } else if old_power_val == 0 {
                            self.apu_power_up();
                        }
                        self.mem_unit.io_registers[0x26] |= (val & 0x80) | 0x70;
                    } else {
                        self.mem_unit.io_registers[0x26] = val;
                    }
                } else if (self.mem_unit.io_registers[0x26] >> 7) == 1 {
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

                    self.mem_unit.io_registers[addr - 0xFF00] = val;
                }
            }
            0xFF30..=0xFF3F => {
                if source == RequestSource::CPU {
                    self.wave_ram_write(addr, val);
                } else {
                    self.mem_unit.io_registers[addr - 0xFF00] = val;
                }
            }
            0xFF41 => {
                if source == RequestSource::PPU {
                    self.mem_unit.io_registers[0x41] = val;
                } else {
                    // println!(
                    //     "{}",
                    //     format!(
                    //         "changing stat bits {:#010b} at LY {}, LYC {} pc: {:#04X}, iter {}",
                    //         val,
                    //         self.mem_unit.io_registers[0x44],
                    //         self.mem_unit.io_registers[0x45],
                    //         self.cpu.pc,
                    //         self.iteration_count
                    //     )
                    // );
                    self.mem_unit.io_registers[0x41] &= 0b0000111;
                    self.mem_unit.io_registers[0x41] |= val & 0b1111000
                }
            }
            0xFF44 => {
                if source == RequestSource::PPU {
                    self.mem_unit.io_registers[0x44] = val;
                }
            }
            0xFF45 => {
                self.mem_unit.io_registers[0x45] = val;
                self.check_lyc_flag();
            }
            0xFF46 => {
                self.mem_unit.io_registers[addr - 0xFF00] = val;
                self.dma_transfer(val as usize);
            }
            0xFF4D => {
                if self.mem_unit.cgb {
                    if source != RequestSource::SPEC {
                        self.mem_unit.io_registers[0x4D] &= 0xFE;
                        self.mem_unit.io_registers[0x4D] |= val & 1;
                    } else {
                        self.mem_unit.io_registers[0x4D] = val;
                    }
                }
            }
            0xFF4F => {
                if self.mem_unit.cgb {
                    // println!(
                    //     "{}",
                    //     format!("vram bank to {} at pc: {:#06X}", val, self.cpu.pc)
                    // );
                    self.mem_unit.vram_bank = val & 1;
                    self.mem_unit.io_registers[0x4F] = 0b11111110 | (val & 1);
                }
            }
            0xFF50 => {
                if self.mem_unit.in_boot_rom {
                    println!("unloaded!");
                    self.unload_boot_rom();
                    self.mem_unit.in_boot_rom = false;
                }
            }
            0xFF52 => {
                self.mem_unit.io_registers[addr - 0xFF00] = val | 0xF;
            }
            0xFF53 => {
                self.mem_unit.io_registers[addr - 0xFF00] = val | 0xE0;
            }
            0xFF54 => {
                self.mem_unit.io_registers[addr - 0xFF00] = val | 0xF;
            }
            0xFF55 => {
                if self.mem_unit.cgb {
                    if self.mem_unit.hdma_active {
                        if (val >> 7) == 0 {
                            println!("stopping HDMA!");
                            self.mem_unit.hdma_active = false;
                            self.mem_unit.io_registers[0x55] |= 0x80;
                        }
                    } else {
                        self.mem_unit.io_registers[0x55] = val;
                    }
                    println!("HDMA TRANSFER");
                    self.hdma_transfer();
                }
            }
            0xFF68 => {
                self.mem_unit.bg_color_inc = (val >> 7) == 1;
                self.mem_unit.io_registers[0x69] = val & 0x3F;
                self.mem_unit.io_registers[0x68] = val;
            }
            0xFF69 => {
                self.mem_unit.bg_color_ram[self.mem_unit.io_registers[0x69] as usize] = val;
                if self.mem_unit.bg_color_inc {
                    self.mem_unit.io_registers[0x69] = (self.mem_unit.io_registers[0x69] + 1) % 64;
                }
            }
            0xFF6A => {
                self.mem_unit.obj_color_inc = (val >> 7) == 1;
                self.mem_unit.io_registers[0x6B] = val & 0x3F;
                self.mem_unit.io_registers[0x6A] = val;
            }
            0xFF6B => {
                self.mem_unit.obj_color_ram[self.mem_unit.io_registers[0x6B] as usize] = val;
                if self.mem_unit.obj_color_inc {
                    self.mem_unit.io_registers[0x6B] = (self.mem_unit.io_registers[0x6B] + 1) % 64;
                }
            }
            0xFF70 => {
                if self.mem_unit.cgb {
                    // println!(
                    //     "{}",
                    //     format!(
                    //         "wram bank to {} ({}) at pc {:#06X}",
                    //         val,
                    //         (val & 0b111),
                    //         self.cpu.pc
                    //     )
                    // );
                    self.mem_unit.wram_bank = (val & 0b111) as usize;
                    if self.mem_unit.wram_bank == 0 {
                        self.mem_unit.wram_bank += 1;
                    }
                    self.mem_unit.io_registers[0x70] = 0b11111000 | self.mem_unit.wram_bank as u8;
                }
            }
            0xFF01..=0xFF7F => self.mem_unit.io_registers[addr - 0xFF00] = val,
            0xFF80..=0xFFFE => self.mem_unit.high_ram[addr - 0xFF80] = val,
            0xFFFF => self.mem_unit.interrupt_enable = val,
            _ => {
                println!("bad general write!")
            }
        }
    }

    fn dma_transfer(&mut self, reg: usize) {
        let start_address = reg << 8;
        //println!("{}", format!("DMA TRANSFER from {:X}", start_address));
        match reg >> 4 {
            // 0x0..=0x3 => {
            //     let end_address = (start_address + 0xA0) as usize;
            //     self.mem_unit.oam
            //         .copy_from_slice(&self.mem_unit.rom[start_address..end_address]);
            // }
            // 0x4..=0x7 => {
            //     let adjusted_start_address = 0x4000 * (self.mem_unit.rom_bank - 1) + reg;
            //     let adjusted_end_address = adjusted_start_address + 0xA0;
            //     self.mem_unit.oam
            //         .copy_from_slice(&self.mem_unit.rom[adjusted_start_address..adjusted_end_address]);
            // }
            0x8..=0x9 => {
                let adjusted_start_address = start_address - 0x8000;
                let adjusted_end_address = adjusted_start_address + 0xA0;
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
                    8192 * (self.mem_unit.ram_bank) + start_address - 0xA000;
                let adjusted_end_address = adjusted_start_address + 0xA0;
                self.mem_unit.oam.copy_from_slice(
                    &self.mem_unit.external_ram[adjusted_start_address..adjusted_end_address],
                );
            }
            0xC => {
                let adjusted_start_address = (start_address - 0xC000) as usize;
                let adjusted_end_address = adjusted_start_address + 0xA0;
                self.mem_unit.oam.copy_from_slice(
                    &self.mem_unit.internal_ram[adjusted_start_address..adjusted_end_address],
                );
            }
            0xD => {
                let adjusted_start_address =
                    (start_address - 0xD000 + 0x1000 * self.mem_unit.wram_bank) as usize;
                let adjusted_end_address = adjusted_start_address + 0xA0;
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
        self.mem_unit.hdma_current_source_addr = (((self.get_memory(HDMA1_ADDR, SOURCE) as usize)
            << 8)
            + self.get_memory(HDMA2_ADDR, SOURCE) as usize)
            & 0xFFF0;
        self.mem_unit.hdma_current_dest_addr = ((((self.get_memory(HDMA3_ADDR, SOURCE) as usize)
            << 8)
            + self.get_memory(HDMA4_ADDR, SOURCE) as usize)
            & 0x1FF0)
            | 0x8000;
        self.mem_unit.hdma_blocks = (hdma5 & 0x7F) + 1;
        let gen_dma = (hdma5 >> 7) == 0;
        // println!(
        //     "{}",
        //     format!(
        //         "HDMA, gen is {} with 0x{:X} blocks and going from {:X} to {:X}",
        //         gen_dma,
        //         self.mem_unit.hdma_blocks,
        //         self.mem_unit.hdma_current_source_addr,
        //         self.mem_unit.hdma_current_dest_addr
        //     )
        // );
        if gen_dma {
            for _ in 0..self.mem_unit.hdma_blocks {
                self.hdma_block_transfer()
            }
        } else {
            self.mem_unit.hdma_active = true;
            self.mem_unit.hdma_primed = true;
            self.mem_unit.io_registers[0x55] &= 0x7F;
        }
    }
    pub fn hdma_block_transfer(&mut self) {
        let high_nibble_source = self.mem_unit.hdma_current_source_addr >> 12;
        let copy_data = match high_nibble_source {
            0x0..=0x3 => {
                &self.mem_unit.rom[self.mem_unit.hdma_current_source_addr
                    ..(self.mem_unit.hdma_current_source_addr + 0x10)]
            }
            0x4..=0x7 => {
                let adjusted_address = self.mem_unit.hdma_current_source_addr
                    + 0x4000 * self.mem_unit.rom_bank
                    - 0x4000;
                &self.mem_unit.rom[adjusted_address..(adjusted_address + 0x10)]
            }
            0xA..=0xB => {
                let adjusted_address = self.mem_unit.hdma_current_source_addr - 0xA000
                    + 0x2000 * self.mem_unit.ram_bank;
                &self.mem_unit.external_ram[adjusted_address..(adjusted_address + 0x10)]
            }
            0xC => {
                let adjusted_address = self.mem_unit.hdma_current_source_addr - 0xC000;
                &self.mem_unit.internal_ram[adjusted_address..(adjusted_address + 0x10)]
            }
            0xD => {
                let adjusted_address = self.mem_unit.hdma_current_source_addr - 0xD000
                    + 0x1000 * self.mem_unit.wram_bank;
                &self.mem_unit.internal_ram[adjusted_address..(adjusted_address + 0x10)]
            }
            _ => {
                panic!("HDMA BAD ADDRESS");
            }
        };
        // println!(
        //     "{}",
        //     format!(
        //         "block number {} transfer from {:X} to {:X}",
        //         self.mem_unit.hdma_blocks, self.mem_unit.hdma_current_source_addr, self.mem_unit.hdma_current_dest_addr
        //     )
        // );
        let adjusted_dest_address = self.mem_unit.hdma_current_dest_addr - 0x8000;
        if self.mem_unit.vram_bank == 0 {
            self.mem_unit.vram_0[adjusted_dest_address..(adjusted_dest_address + 0x10)]
                .copy_from_slice(copy_data);
        }
        if self.mem_unit.vram_bank == 1 {
            self.mem_unit.vram_1[adjusted_dest_address..(adjusted_dest_address + 0x10)]
                .copy_from_slice(copy_data);
        }
        self.mem_unit.hdma_current_source_addr += 0x10;
        self.mem_unit.hdma_current_dest_addr += 0x10;
        let (source_high, source_low) = split_u16(self.mem_unit.hdma_current_source_addr as u16);
        self.write_memory(HDMA1_ADDR, source_high, SOURCE);
        self.write_memory(HDMA2_ADDR, source_low, SOURCE);

        let (dest_high, dest_low) = split_u16(self.mem_unit.hdma_current_dest_addr as u16);
        self.write_memory(HDMA3_ADDR, dest_high, SOURCE);
        self.write_memory(HDMA4_ADDR, dest_low, SOURCE);
        self.mem_unit.hdma_blocks -= 1;
        if self.mem_unit.hdma_blocks == 0 {
            self.mem_unit.hdma_active = false;
            self.mem_unit.io_registers[0x55] = 0xFF;
        }
    }
    pub fn dma_tick(&mut self) {
        if self.mem_unit.dma_cycles > 0 {
            self.mem_unit.dma_cycles -= 4;
        }
        if self.mem_unit.hdma_active {
            if self.mem_unit.hdma_primed && self.mem_unit.ppu_mode == 0 {
                self.hdma_block_transfer();
                self.mem_unit.hdma_primed = false;
            } else if !self.mem_unit.hdma_primed && self.mem_unit.ppu_mode != 0 {
                self.mem_unit.hdma_primed = true;
            }
        }
    }
    pub fn access_vram(&self, addr: impl Into<usize>, bank: u8) -> u8 {
        let addr = addr.into() as usize;
        if bank == 0 {
            self.mem_unit.vram_0[addr - 0x8000]
        } else {
            self.mem_unit.vram_1[addr - 0x8000]
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
        self.mem_unit.io_registers[P1_ADDR - 0xFF00] = 0xCF;
        self.mem_unit.io_registers[TIMA_ADDR - 0xFF00] = 0x00;
        self.mem_unit.io_registers[TMA_ADDR - 0xFF00] = 0x00;
        self.mem_unit.io_registers[TAC_ADDR - 0xFF00] = 0xF8;
        self.mem_unit.io_registers[INT_FLAG_ADDR - 0xFF00] = 0xE1;
        self.mem_unit.io_registers[LCDC_ADDR - 0xFF00] = 0x91;
        self.mem_unit.io_registers[SCX_ADDR - 0xFF00] = 0x00;
        self.mem_unit.io_registers[SCY_ADDR - 0xFF00] = 0x00;
        self.mem_unit.io_registers[LYC_ADDR - 0xFF00] = 0x00;
        self.mem_unit.io_registers[BGP_ADDR - 0xFF00] = 0xFC;
        self.mem_unit.io_registers[WY_ADDR - 0xFF00] = 0x00;
        self.mem_unit.io_registers[WX_ADDR - 0xFF00] = 0x00;
        self.mem_unit.io_registers[KEY1_ADDR - 0xFF00] = 0xFF;
        self.mem_unit.io_registers[HDMA1_ADDR - 0xFF00] = 0xFF;
        self.mem_unit.io_registers[HDMA2_ADDR - 0xFF00] = 0xFF;
        self.mem_unit.io_registers[HDMA3_ADDR - 0xFF00] = 0xFF;
        self.mem_unit.io_registers[HDMA4_ADDR - 0xFF00] = 0xFF;
        self.mem_unit.io_registers[HDMA5_ADDR - 0xFF00] = 0xFF;

        self.mem_unit.io_registers[BGP_ADDR - 0xFF00] = 0xFC;
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
        self.mem_unit.cpu_initialize = true;
        self.memory_initialize_after_boot();
    }
}
impl GameBoyEmulator {
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
            vram: self.mem_unit.vram_0.clone(),
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
            ram_enable: self.mem_unit.ram_enable,
            hold_mem: self.mem_unit.hold_mem.clone(),
            in_boot_rom: self.mem_unit.in_boot_rom,
            dma_cycles: self.mem_unit.dma_cycles,
            cpu: self.cpu,
            ppu: self.ppu.clone(),
            timer: self.timer,
        };
        bincode::serialize_into(save_file, &save_data).unwrap();
    }
    pub fn open_game(&mut self, path: &Path) {
        let open_file = File::open(&path).unwrap();
        let open_data: SaveGame = bincode::deserialize_from(open_file).unwrap();
        self.mem_unit.vram_0 = open_data.vram;
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
        self.mem_unit.ram_enable = open_data.ram_enable;
        self.mem_unit.hold_mem = open_data.hold_mem;
        self.mem_unit.in_boot_rom = open_data.in_boot_rom;
        self.mem_unit.dma_cycles = open_data.dma_cycles;
        self.cpu = open_data.cpu;
        self.ppu = open_data.ppu;
        self.timer = open_data.timer;
    }
}

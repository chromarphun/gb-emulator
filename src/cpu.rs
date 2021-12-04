use crate::{CYCLES_PER_PERIOD, PERIOD_NS};
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const REG_A: usize = 0;
const REG_B: usize = 1;
const REG_C: usize = 2;
const REG_D: usize = 3;
const REG_E: usize = 4;
const REG_H: usize = 5;
const REG_L: usize = 6;
const CARRY_LIMIT_16: u32 = 65535;
const CARRY_LIMIT_8: u16 = 255;
const NANOS_PER_DOT: f64 = 238.4185791015625;
const INTERRUPT_DOTS: i32 = 20;
const HALT_DOTS: u8 = 10;

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

fn get_function_map() -> [fn(&mut CentralProcessingUnit, u8); 256] {
    [
        //0x00
        CentralProcessingUnit::nop,
        CentralProcessingUnit::ld_reg_16,
        CentralProcessingUnit::ld_addr_a,
        CentralProcessingUnit::inc_reg_16,
        CentralProcessingUnit::inc_reg_8,
        CentralProcessingUnit::dec_reg_8,
        CentralProcessingUnit::ld_reg_8,
        CentralProcessingUnit::rlca,
        CentralProcessingUnit::ld_addr_sp,
        CentralProcessingUnit::add_hl_reg_16,
        CentralProcessingUnit::ld_a_addr,
        CentralProcessingUnit::dec_reg_16,
        CentralProcessingUnit::inc_reg_8,
        CentralProcessingUnit::dec_reg_8,
        CentralProcessingUnit::ld_reg_8,
        CentralProcessingUnit::rrca,
        //0x10
        CentralProcessingUnit::stop,
        CentralProcessingUnit::ld_reg_16,
        CentralProcessingUnit::ld_addr_a,
        CentralProcessingUnit::inc_reg_16,
        CentralProcessingUnit::inc_reg_8,
        CentralProcessingUnit::dec_reg_8,
        CentralProcessingUnit::ld_reg_8,
        CentralProcessingUnit::rla,
        CentralProcessingUnit::jr,
        CentralProcessingUnit::add_hl_reg_16,
        CentralProcessingUnit::ld_a_addr,
        CentralProcessingUnit::dec_reg_16,
        CentralProcessingUnit::inc_reg_8,
        CentralProcessingUnit::dec_reg_8,
        CentralProcessingUnit::ld_reg_8,
        CentralProcessingUnit::rra,
        //0x20
        CentralProcessingUnit::jr,
        CentralProcessingUnit::ld_reg_16,
        CentralProcessingUnit::ld_addr_a,
        CentralProcessingUnit::inc_reg_16,
        CentralProcessingUnit::inc_reg_8,
        CentralProcessingUnit::dec_reg_8,
        CentralProcessingUnit::ld_reg_8,
        CentralProcessingUnit::daa,
        CentralProcessingUnit::jr,
        CentralProcessingUnit::add_hl_reg_16,
        CentralProcessingUnit::ld_a_addr,
        CentralProcessingUnit::dec_reg_16,
        CentralProcessingUnit::inc_reg_8,
        CentralProcessingUnit::dec_reg_8,
        CentralProcessingUnit::ld_reg_8,
        CentralProcessingUnit::cpl,
        //0x30
        CentralProcessingUnit::jr,
        CentralProcessingUnit::ld_reg_16,
        CentralProcessingUnit::ld_addr_a,
        CentralProcessingUnit::inc_reg_16,
        CentralProcessingUnit::inc_reg_8,
        CentralProcessingUnit::dec_reg_8,
        CentralProcessingUnit::ld_reg_8,
        CentralProcessingUnit::scf,
        CentralProcessingUnit::jr,
        CentralProcessingUnit::add_hl_reg_16,
        CentralProcessingUnit::ld_a_addr,
        CentralProcessingUnit::dec_reg_16,
        CentralProcessingUnit::inc_reg_8,
        CentralProcessingUnit::dec_reg_8,
        CentralProcessingUnit::ld_reg_8,
        CentralProcessingUnit::ccf,
        //0x40
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_hl_addr,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_hl_addr,
        CentralProcessingUnit::ld_reg_reg,
        //0x50
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_hl_addr,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_hl_addr,
        CentralProcessingUnit::ld_reg_reg,
        //0x60
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_hl_addr,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_hl_addr,
        CentralProcessingUnit::ld_reg_reg,
        //0x70
        CentralProcessingUnit::ld_hl_addr_reg,
        CentralProcessingUnit::ld_hl_addr_reg,
        CentralProcessingUnit::ld_hl_addr_reg,
        CentralProcessingUnit::ld_hl_addr_reg,
        CentralProcessingUnit::ld_hl_addr_reg,
        CentralProcessingUnit::ld_hl_addr_reg,
        CentralProcessingUnit::halt,
        CentralProcessingUnit::ld_hl_addr_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_reg,
        CentralProcessingUnit::ld_reg_hl_addr,
        CentralProcessingUnit::ld_reg_reg,
        //0x80
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        //0x90
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        //0xA0
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        //0xB0
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::arthimetic_a,
        //0xC0
        CentralProcessingUnit::ret,
        CentralProcessingUnit::pop,
        CentralProcessingUnit::jp,
        CentralProcessingUnit::jp,
        CentralProcessingUnit::call,
        CentralProcessingUnit::push,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::rst,
        CentralProcessingUnit::ret,
        CentralProcessingUnit::ret,
        CentralProcessingUnit::jp,
        CentralProcessingUnit::cb,
        CentralProcessingUnit::call,
        CentralProcessingUnit::call,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::rst,
        //0xD0
        CentralProcessingUnit::ret,
        CentralProcessingUnit::pop,
        CentralProcessingUnit::jp,
        CentralProcessingUnit::fail,
        CentralProcessingUnit::call,
        CentralProcessingUnit::push,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::rst,
        CentralProcessingUnit::ret,
        CentralProcessingUnit::ret,
        CentralProcessingUnit::jp,
        CentralProcessingUnit::fail,
        CentralProcessingUnit::call,
        CentralProcessingUnit::fail,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::rst,
        //0xE0
        CentralProcessingUnit::ld_addr_a,
        CentralProcessingUnit::pop,
        CentralProcessingUnit::ld_addr_a,
        CentralProcessingUnit::fail,
        CentralProcessingUnit::fail,
        CentralProcessingUnit::push,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::rst,
        CentralProcessingUnit::add_sp_i8,
        CentralProcessingUnit::jp_hl,
        CentralProcessingUnit::ld_addr_a,
        CentralProcessingUnit::fail,
        CentralProcessingUnit::fail,
        CentralProcessingUnit::fail,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::rst,
        //0xF0
        CentralProcessingUnit::ld_a_addr,
        CentralProcessingUnit::pop,
        CentralProcessingUnit::ld_a_addr,
        CentralProcessingUnit::di,
        CentralProcessingUnit::fail,
        CentralProcessingUnit::push,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::rst,
        CentralProcessingUnit::ld_hl_sp_i8,
        CentralProcessingUnit::ld_sp_hl,
        CentralProcessingUnit::ld_a_addr,
        CentralProcessingUnit::ei,
        CentralProcessingUnit::fail,
        CentralProcessingUnit::fail,
        CentralProcessingUnit::arthimetic_a,
        CentralProcessingUnit::rst,
    ]
}

fn get_cycles_map() -> [i32; 256] {
    [
        04, 12, 08, 08, 04, 04, 08, 04, 20, 08, 08, 08, 04, 04, 08, 04, 04, 12, 08, 08, 04, 04, 08,
        04, 12, 08, 08, 08, 04, 04, 08, 04, 08, 12, 08, 08, 04, 04, 08, 04, 08, 08, 08, 08, 04, 04,
        08, 04, 08, 12, 08, 08, 12, 12, 12, 04, 08, 08, 08, 08, 04, 04, 08, 04, 04, 04, 04, 04, 04,
        04, 08, 04, 04, 04, 04, 04, 04, 04, 08, 04, 04, 04, 04, 04, 04, 04, 08, 04, 04, 04, 04, 04,
        04, 04, 08, 04, 04, 04, 04, 04, 04, 04, 08, 04, 04, 04, 04, 04, 04, 04, 08, 04, 08, 08, 08,
        08, 08, 08, 04, 08, 04, 04, 04, 04, 04, 04, 08, 04, 04, 04, 04, 04, 04, 04, 08, 04, 04, 04,
        04, 04, 04, 04, 08, 04, 04, 04, 04, 04, 04, 04, 08, 04, 04, 04, 04, 04, 04, 04, 08, 04, 04,
        04, 04, 04, 04, 04, 08, 04, 04, 04, 04, 04, 04, 04, 08, 04, 04, 04, 04, 04, 04, 04, 08, 04,
        04, 04, 04, 04, 04, 04, 08, 04, 08, 12, 12, 16, 12, 16, 08, 16, 08, 16, 12, 04, 12, 24, 08,
        16, 08, 12, 12, 00, 12, 16, 08, 16, 08, 16, 12, 00, 12, 00, 08, 16, 12, 12, 08, 00, 00, 16,
        08, 16, 16, 04, 16, 00, 00, 00, 08, 16, 12, 12, 08, 04, 00, 16, 08, 16, 12, 08, 16, 04, 00,
        00, 08, 16,
    ]
}
pub struct CentralProcessingUnit {
    regs: [u8; 7],
    pc: u16,
    sp: u16,
    cycle_modification: i32,
    z_flag: u8,
    n_flag: u8,
    h_flag: u8,
    c_flag: u8,
    reenable_interrupts: bool,
    disable_interrupts: bool,
    function_map: [fn(&mut CentralProcessingUnit, u8); 256],
    cycles_map: [i32; 256],
    rom: Arc<Mutex<Vec<u8>>>,
    external_ram: Arc<Mutex<[u8; 131072]>>,
    internal_ram: Arc<Mutex<[u8; 8192]>>,
    high_ram: [u8; 127],
    rom_bank: Arc<Mutex<usize>>,
    ram_bank: Arc<Mutex<usize>>,
    lcdc: Arc<Mutex<u8>>,
    stat: Arc<Mutex<u8>>,
    vram: Arc<Mutex<[u8; 8192]>>,
    oam: Arc<Mutex<[u8; 160]>>,
    scy: Arc<Mutex<u8>>,
    scx: Arc<Mutex<u8>>,
    ly: Arc<Mutex<u8>>,
    lyc: Arc<Mutex<u8>>,
    wy: Arc<Mutex<u8>>,
    wx: Arc<Mutex<u8>>,
    bgp: Arc<Mutex<u8>>,
    ime: Arc<Mutex<u8>>,
    interrupt_enable: Arc<Mutex<u8>>,
    interrupt_flag: Arc<Mutex<u8>>,
    p1: Arc<Mutex<u8>>,
    div: Arc<Mutex<u8>>,
    tima: Arc<Mutex<u8>>,
    tma: Arc<Mutex<u8>>,
    tac: Arc<Mutex<u8>>,
    obp0: Arc<Mutex<u8>>,
    obp1: Arc<Mutex<u8>>,
    dma_transfer: Arc<Mutex<bool>>,
    dma_register: Arc<Mutex<u8>>,
    change_ime_true: bool,
    change_ime_false: bool,
    debug_var: usize,
    ram_enable: bool,
    in_boot_rom: bool,
    holding_ff01: u8,
    holding_ff02: u8,
    halting: bool,
    cycle_count: Arc<Mutex<i32>>,
    cycle_cond: Arc<Condvar>,
    dma_cond: Arc<Condvar>,
    lcdc_cond: Arc<Condvar>,
    now: Instant,
    repeat: bool,
}

impl CentralProcessingUnit {
    pub fn new(
        rom: Arc<Mutex<Vec<u8>>>,
        external_ram: Arc<Mutex<[u8; 131072]>>,
        internal_ram: Arc<Mutex<[u8; 8192]>>,
        rom_bank: Arc<Mutex<usize>>,
        ram_bank: Arc<Mutex<usize>>,
        lcdc: Arc<Mutex<u8>>,
        stat: Arc<Mutex<u8>>,
        vram: Arc<Mutex<[u8; 8192]>>,
        oam: Arc<Mutex<[u8; 160]>>,
        scy: Arc<Mutex<u8>>,
        scx: Arc<Mutex<u8>>,
        ly: Arc<Mutex<u8>>,
        lyc: Arc<Mutex<u8>>,
        wy: Arc<Mutex<u8>>,
        wx: Arc<Mutex<u8>>,
        bgp: Arc<Mutex<u8>>,
        ime: Arc<Mutex<u8>>,
        p1: Arc<Mutex<u8>>,
        div: Arc<Mutex<u8>>,
        tima: Arc<Mutex<u8>>,
        tma: Arc<Mutex<u8>>,
        tac: Arc<Mutex<u8>>,
        obp0: Arc<Mutex<u8>>,
        obp1: Arc<Mutex<u8>>,
        dma_transfer: Arc<Mutex<bool>>,
        dma_register: Arc<Mutex<u8>>,
        interrupt_enable: Arc<Mutex<u8>>,
        interrupt_flag: Arc<Mutex<u8>>,
        cycle_count: Arc<Mutex<i32>>,
        cycle_cond: Arc<Condvar>,
        dma_cond: Arc<Condvar>,
        lcdc_cond: Arc<Condvar>,
    ) -> CentralProcessingUnit {
        let regs = [0u8; 7];
        let pc: u16 = 0x0;
        let sp: u16 = 0xFFFE;
        let reenable_interrupts: bool = false;
        let disable_interrupts: bool = false;
        let z_flag: u8 = 0;
        let n_flag: u8 = 0;
        let h_flag: u8 = 0;
        let c_flag: u8 = 0;
        let function_map: [fn(&mut CentralProcessingUnit, u8); 256] = get_function_map();
        let cycles_map: [i32; 256] = get_cycles_map();
        let cycle_modification: i32 = 0;
        let change_ime_false = false;
        let change_ime_true = false;
        let debug_var: usize = 0;
        let ram_enable = false;
        let high_ram = [0u8; 127];
        let in_boot_rom = true;
        let holding_ff01 = 0;
        let holding_ff02 = 0;
        let halting = false;
        let now = Instant::now();
        let repeat = false;
        CentralProcessingUnit {
            regs,
            pc,
            sp,
            cycle_modification,
            z_flag,
            n_flag,
            h_flag,
            c_flag,
            reenable_interrupts,
            disable_interrupts,
            function_map,
            cycles_map,
            rom,
            external_ram,
            internal_ram,
            high_ram,
            rom_bank,
            ram_bank,
            lcdc,
            stat,
            vram,
            oam,
            scy,
            scx,
            ly,
            lyc,
            wy,
            wx,
            bgp,
            ime,
            interrupt_enable,
            interrupt_flag,
            p1,
            div,
            tima,
            tma,
            tac,
            obp0,
            obp1,
            change_ime_true,
            change_ime_false,
            dma_register,
            dma_transfer,
            debug_var,
            ram_enable,
            in_boot_rom,
            holding_ff01,
            holding_ff02,
            halting,
            cycle_count,
            cycle_cond,
            dma_cond,
            lcdc_cond,
            now,
            repeat,
        }
    }
    pub fn run(&mut self, path: &str) {
        let mut f = File::open(path).expect("File problem!");
        f.read_to_end(&mut *self.rom.lock().unwrap())
            .expect("Read issue!");

        {
            let mut hold_mem = [0u8; 256];
            hold_mem.copy_from_slice(&self.rom.lock().unwrap()[..256]);
            let boot_mem = [
                0x31, 0xFE, 0xFF, 0xAF, 0x21, 0xFF, 0x9F, 0x32, 0xCB, 0x7C, 0x20, 0xFB, 0x21, 0x26,
                0xFF, 0x0E, 0x11, 0x3E, 0x80, 0x32, 0xE2, 0x0C, 0x3E, 0xF3, 0xE2, 0x32, 0x3E, 0x77,
                0x77, 0x3E, 0xFC, 0xE0, 0x47, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1A, 0xCD, 0x95,
                0x00, 0xCD, 0x96, 0x00, 0x13, 0x7B, 0xFE, 0x34, 0x20, 0xF3, 0x11, 0xD8, 0x00, 0x06,
                0x08, 0x1A, 0x13, 0x22, 0x23, 0x05, 0x20, 0xF9, 0x3E, 0x19, 0xEA, 0x10, 0x99, 0x21,
                0x2F, 0x99, 0x0E, 0x0C, 0x3D, 0x28, 0x08, 0x32, 0x0D, 0x20, 0xF9, 0x2E, 0x0F, 0x18,
                0xF3, 0x67, 0x3E, 0x64, 0x57, 0xE0, 0x42, 0x3E, 0x91, 0xE0, 0x40, 0x04, 0x1E, 0x02,
                0x0E, 0x0C, 0xF0, 0x44, 0xFE, 0x90, 0x20, 0xFA, 0x0D, 0x20, 0xF7, 0x1D, 0x20, 0xF2,
                0x0E, 0x13, 0x24, 0x7C, 0x1E, 0x83, 0xFE, 0x62, 0x28, 0x06, 0x1E, 0xC1, 0xFE, 0x64,
                0x20, 0x06, 0x7B, 0xE2, 0x0C, 0x3E, 0x87, 0xE2, 0xF0, 0x42, 0x90, 0xE0, 0x42, 0x15,
                0x20, 0xD2, 0x05, 0x20, 0x4F, 0x16, 0x20, 0x18, 0xCB, 0x4F, 0x06, 0x04, 0xC5, 0xCB,
                0x11, 0x17, 0xC1, 0xCB, 0x11, 0x17, 0x05, 0x20, 0xF5, 0x22, 0x23, 0x22, 0x23, 0xC9,
                0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C,
                0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6,
                0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC,
                0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E, 0x3C, 0x42, 0xB9, 0xA5, 0xB9, 0xA5, 0x42, 0x3C,
                0x21, 0x04, 0x01, 0x11, 0xA8, 0x00, 0x1A, 0x13, 0xBE, 0x20, 0xFE, 0x23, 0x7D, 0xFE,
                0x34, 0x20, 0xF5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xFB, 0x86, 0x20, 0xFE,
                0x3E, 0x01, 0xE0, 0x50,
            ];
            self.rom.lock().unwrap()[..256].copy_from_slice(&boot_mem);
            self.now = Instant::now();
            while self.pc < 0x100 {
                self.process();
            }
            self.rom.lock().unwrap()[..256].copy_from_slice(&hold_mem);
            println!("Made it past boot!");
        }
        //self.pc = 0x100;
        self.in_boot_rom = false;
        loop {
            self.process();
        }
    }
    fn process(&mut self) {
        if self.change_ime_true {
            self.change_ime_true = false;
            *self.ime.lock().unwrap() = 1;
        }
        if self.reenable_interrupts {
            self.reenable_interrupts = false;
            self.change_ime_true = true;
        }

        let viable_interrupts =
            *self.interrupt_flag.lock().unwrap() & *self.interrupt_enable.lock().unwrap();

        if *self.ime.lock().unwrap() == 1 && viable_interrupts != 0 && !self.halting {
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
            *self.interrupt_flag.lock().unwrap() &= mask;
            *self.ime.lock().unwrap() = 0;
            let (high_pc, low_pc) = split_u16(self.pc);
            self.push_stack(high_pc, low_pc);
            self.pc = addr;
            *self.cycle_count.lock().unwrap() += INTERRUPT_DOTS;
        } else {
            let old_pc = self.pc;

            let command = self.get_memory(self.pc as usize) as usize;
            if !self.in_boot_rom {
                //println!("{}", format!("pc: {:X}, command: {:X}", self.pc, command));
            }
            self.debug_var = 1;
            self.function_map[command](self, command as u8);
            let cycles = if self.cycle_modification != 0 {
                let val = self.cycle_modification;
                self.cycle_modification = 0;
                val
            } else {
                self.cycles_map[command]
            };
            if self.repeat {
                self.repeat = false;
                self.pc = old_pc;
            }

            if self.regs[REG_A] == 0x64 {
                self.debug_var = 1;
            }

            *self.cycle_count.lock().unwrap() += cycles;
            self.cycle_cond.notify_all();
            if *self.cycle_count.lock().unwrap() >= CYCLES_PER_PERIOD {
                spin_sleep::sleep(Duration::new(0, PERIOD_NS).saturating_sub(self.now.elapsed()));
                *self.cycle_count.lock().unwrap() = 0;
                self.now = Instant::now();
            }
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
    #[inline]
    fn push_stack(&mut self, high_val: u8, low_val: u8) {
        self.write_memory((self.sp - 1) as usize, high_val);
        self.write_memory((self.sp - 2) as usize, low_val);
        self.sp -= 2;
    }
    fn pop_stack(&mut self) -> [u8; 2] {
        let val1 = self.get_memory((self.sp) as usize);
        let val2 = self.get_memory((self.sp + 1) as usize);
        self.sp += 2;
        [val1, val2]
    }
    fn write_memory(&mut self, addr: usize, val: u8) {
        self.write_memory_rom_only(addr, val);
    }
    fn get_memory(&mut self, addr: usize) -> u8 {
        self.get_memory_rom_only(addr)
    }
    fn write_memory_rom_only(&mut self, addr: usize, val: u8) {
        match addr {
            0x8000..=0x9FFF => {
                let mutex = self.vram.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    mem_unlocked[addr - 0x8000] = val;
                } else {
                    //println!("VRAM WRITE FAILED");
                }
            }
            0xC000..=0xDFFF => {
                let mutex = self.internal_ram.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    mem_unlocked[addr - 0xC000] = val;
                } else {
                    //println!("INTERNAL RAMWRITE FAILED");
                }
            }
            0xE000..=0xFDFF => {
                let mutex = self.internal_ram.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    mem_unlocked[addr - 0xE000] = val;
                } else {
                    //println!("INTERNAL RAMWRITE FAILED");
                }
            }
            0xFE00..=0xFE9F => {
                let mutex = self.oam.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    mem_unlocked[addr - 0xFE00] = val;
                } else {
                    //println!("OAM WRITE FAILED");
                }
            }
            0xFEA0..=0xFEFF => {
                //println!("FORBIDDEN AREA");
            }
            0xFF00 => {
                *self.p1.lock().unwrap() = val;
            }
            0xFF01 => {
                //println!("{}", val as char);
                self.holding_ff01 = val;
            }
            0xFF02 => {
                self.holding_ff02 = val;
            }
            0xFF04 => {
                *self.div.lock().unwrap() = 0;
            }
            0xFF05 => {
                *self.tima.lock().unwrap() = val;
            }
            0xFF06 => {
                *self.tma.lock().unwrap() = val;
            }
            0xFF07 => {
                *self.tac.lock().unwrap() = val;
            }
            0xFF0F => {
                *self.interrupt_flag.lock().unwrap() = val;
            }
            0xFF10..=0xFF1E | 0xFF30..=0xFF3F | 0xFF20..=0xFF26 => {}
            0xFF40 => {
                let mutex = self.lcdc.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
                self.lcdc_cond.notify_all();
                //println!("{}", format!("LCDC CHANGE TO {:#010b}", val));
            }
            0xFF41 => {
                let mutex = self.stat.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF42 => {
                let mutex = self.scy.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF43 => {
                let mutex = self.scx.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF44 => {
                let mutex = self.ly.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF45 => {
                let mutex = self.lyc.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF46 => {
                *self.dma_transfer.lock().unwrap() = true;
                *self.dma_register.lock().unwrap() = val;
                self.dma_cond.notify_all();
            }
            0xFF47 => {
                let mutex = self.bgp.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF48 => {
                let mutex = self.obp0.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF49 => {
                let mutex = self.obp1.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF4A => {
                let mutex = self.wy.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }

            0xFF4B => {
                let mutex = self.wx.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF80..=0xFFFE => self.high_ram[addr - 0xFF80] = val,
            0xFFFF => *self.interrupt_enable.lock().unwrap() = val,
            _ => {
                // println!(
                //     "{}",
                //     format!("trying to write to 0x{:X} with {:X}!", addr, val)
                // )
            }
        }
    }
    fn write_memory_mbc3(&mut self, addr: usize, val: u8) {
        match addr {
            0x0..=0x1FFF => match addr {
                0x0 => self.ram_enable = false,
                0xA => self.ram_enable = true,
                _ => {}
            },
            0x2000..=0x3FFF => {
                *self.rom_bank.lock().unwrap() = if val == 0 { 1 } else { val as usize };
            }
            0x4000..=0x5FFF => *self.ram_bank.lock().unwrap() = addr,
            0x8000..=0x9FFF => {
                let mutex = self.vram.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    mem_unlocked[addr - 0x8000] = val;
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    let mutex = self.external_ram.try_lock();
                    if let Ok(mut mem_unlocked) = mutex {
                        mem_unlocked[8192 * *self.ram_bank.lock().unwrap() + addr - 0xA000] = val;
                    }
                }
            }
            0xC000..=0xDFFF => {
                let mutex = self.internal_ram.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    mem_unlocked[addr - 0xC000] = val;
                }
            }
            0xFe00..=0xFE9F => {}
            0xFF00 => {
                *self.p1.lock().unwrap() = val;
            }
            0xFF04 => {
                *self.div.lock().unwrap() = val;
            }
            0xFF05 => {
                *self.tima.lock().unwrap() = val;
            }
            0xFF06 => {
                *self.tma.lock().unwrap() = val;
            }
            0xFF07 => {
                *self.tac.lock().unwrap() = val;
            }
            0xFF0F => {
                *self.interrupt_enable.lock().unwrap() = val;
            }
            0xFF40 => {
                let mutex = self.lcdc.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF41 => {
                let mutex = self.stat.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF42 => {
                let mutex = self.scy.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF43 => {
                let mutex = self.scx.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF44 => {
                let mutex = self.ly.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF45 => {
                let mutex = self.lyc.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF46 => {
                *self.dma_transfer.lock().unwrap() = true;
                *self.dma_register.lock().unwrap() = val;
            }
            0xFF47 => {
                let mutex = self.bgp.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF48 => {
                let mutex = self.obp0.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF49 => {
                let mutex = self.obp1.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF4A => {
                let mutex = self.wy.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }

            0xFF4B => {
                let mutex = self.wx.try_lock();
                if let Ok(mut mem_unlocked) = mutex {
                    *mem_unlocked = val;
                }
            }
            0xFF80..=0xFFFE => self.high_ram[addr - 0xFF80] = val,
            0xFFFF => *self.interrupt_enable.lock().unwrap() = val,
            _ => {}
        }
    }
    fn get_memory_rom_only(&self, addr: usize) -> u8 {
        match addr {
            0x0..=0x7FFF => {
                let mutex = self.rom.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    (*mem_unlocked)[addr]
                } else {
                    0xFF
                }
            }

            0x8000..=0x9FFF => {
                let mutex = self.vram.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    mem_unlocked[addr - 0x8000]
                } else {
                    0xFF
                }
            }
            0xC000..=0xDFFF => {
                let mutex = self.internal_ram.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    mem_unlocked[addr - 0xC000]
                } else {
                    0xFF
                }
            }
            0xE000..=0xFDFF => {
                let mutex = self.internal_ram.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    mem_unlocked[addr - 0xE000]
                } else {
                    0xFF
                }
            }
            0xFE00..=0xFE9F => {
                let mutex = self.oam.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    mem_unlocked[addr - 0xFE00]
                } else {
                    0xFF
                }
            }
            0xFF00 => *self.p1.lock().unwrap(),
            0xFF01 => self.holding_ff01,
            0xFF02 => self.holding_ff02,
            0xFF04 => *self.div.lock().unwrap(),
            0xFF05 => *self.tima.lock().unwrap(),
            0xFF06 => *self.tma.lock().unwrap(),
            0xFF07 => *self.tac.lock().unwrap(),
            0xFF0F => *self.interrupt_flag.lock().unwrap(),
            0xFF40 => {
                let mutex = self.lcdc.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF41 => {
                let mutex = self.stat.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF42 => {
                let mutex = self.scy.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF43 => {
                let mutex = self.scx.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF44 => {
                let mutex = self.ly.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF45 => {
                let mutex = self.lyc.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF47 => {
                let mutex = self.bgp.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF48 => {
                let mutex = self.obp0.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF49 => {
                let mutex = self.obp1.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF4A => {
                let mutex = self.wy.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }

            0xFF4B => {
                let mutex = self.wx.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF80..=0xFFFE => self.high_ram[addr - 0xFF80],
            0xFFFF => *self.interrupt_enable.lock().unwrap(),
            _ => {
                println!("{}", format!("trying to write to 0x{:X}!", addr));
                0xFF
            }
        }
    }
    fn get_memory_mbc3(&self, addr: usize) -> u8 {
        match addr {
            0x0..=0x3FFF => {
                let mutex = self.rom.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    (*mem_unlocked)[addr]
                } else {
                    0xFF
                }
            }
            0x4000..=0x7FFF => {
                let mutex = self.rom.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    mem_unlocked[16384 * (*self.rom_bank.lock().unwrap()) - 0x4000 + addr]
                } else {
                    0xFF
                }
            }
            0x8000..=0x9FFF => {
                let mutex = self.vram.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    mem_unlocked[addr - 0x8000]
                } else {
                    0xFF
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    if *self.ram_bank.lock().unwrap() <= 3 {
                        let mutex = self.vram.try_lock();
                        if let Ok(mem_unlocked) = mutex {
                            mem_unlocked[8192 * *self.ram_bank.lock().unwrap() + addr - 0xA000]
                        } else {
                            0xFF
                        }
                    } else {
                        0 // Timer
                    }
                } else {
                    0xFF
                }
            }
            0xC000..=0xDFFF => {
                let mutex = self.internal_ram.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    mem_unlocked[addr - 0xC000]
                } else {
                    0xFF
                }
            }
            0xFE00..=0xFE9F => 0xFF,
            0xFF00 => *self.p1.lock().unwrap(),
            0xFF04 => *self.div.lock().unwrap(),
            0xFF05 => *self.tima.lock().unwrap(),
            0xFF06 => *self.tma.lock().unwrap(),
            0xFF07 => *self.tac.lock().unwrap(),
            0xFF0F => *self.interrupt_flag.lock().unwrap(),
            0xFF40 => {
                let mutex = self.lcdc.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF41 => {
                let mutex = self.stat.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF42 => {
                let mutex = self.scy.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF43 => {
                let mutex = self.scx.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF44 => {
                let mutex = self.ly.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF45 => {
                let mutex = self.lyc.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF47 => {
                let mutex = self.bgp.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF48 => {
                let mutex = self.obp0.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF49 => {
                let mutex = self.obp1.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF4A => {
                let mutex = self.wy.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }

            0xFF4B => {
                let mutex = self.wx.try_lock();
                if let Ok(mem_unlocked) = mutex {
                    *mem_unlocked
                } else {
                    0xFF
                }
            }
            0xFF80..=0xFFFE => self.high_ram[addr - 0xFF80],
            0xFFFF => *self.interrupt_enable.lock().unwrap(),
            _ => 0xFF,
        }
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
    // fn sub_set_flags(&mut self, val1: u16, val2: u16, z: bool, h: bool, c: bool) {
    //     if z {
    //         self.z_flag = if (val1 - val2) == 0 { 1 } else { 0 };
    //     }
    //     if h {
    //         self.h_flag = if (((val1 & 0xF) - (val2 & 0xF)) & 0x10) == 0x10 {
    //             1
    //         } else {
    //             0
    //         };
    //     }
    //     if c {
    //         self.c_flag = if val1 < val2 { 1 } else { 0 };
    //     }
    // }
    fn fail(&mut self, command: u8) {
        panic!(
            "{}",
            format!("Unrecognized command {:X} at ld_reg_16!", command)
        );
    }
    fn nop(&mut self, _command: u8) {
        self.pc += 1;
    }
    fn stop(&mut self, _command: u8) {}
    fn ld_reg_16(&mut self, command: u8) {
        let low_byte = self.get_memory((self.pc + 1) as usize);
        let high_byte = self.get_memory((self.pc + 2) as usize);
        match command {
            0x01 => {
                //LD BC
                self.regs[REG_B] = high_byte;
                self.regs[REG_C] = low_byte;
            }
            0x11 => {
                //LD DE
                self.regs[REG_D] = high_byte;
                self.regs[REG_E] = low_byte;
            }
            0x21 => {
                //LD HL
                self.regs[REG_H] = high_byte;
                self.regs[REG_L] = low_byte;
            }
            0x31 => {
                //LD SP XX
                self.sp = combine_bytes(high_byte, low_byte);
            }
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at ld_reg_16!", command)
            ),
        };
        self.pc += 3;
    }
    fn ld_addr_a(&mut self, command: u8) {
        let adding_1 = self.get_memory((self.pc + 1) as usize);
        let adding_2 = self.get_memory((self.pc + 2) as usize);
        let addr = match command {
            0x02 => combine_bytes(self.regs[REG_B], self.regs[REG_C]), //LD (BC) A

            0x12 => combine_bytes(self.regs[REG_D], self.regs[REG_E]), //LD (DE) A

            0x22 => {
                //LD (HL+) A
                let mut hl: u16 = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
                let addr = hl;
                hl = hl.wrapping_add(1);
                let (new_h, new_l) = split_u16(hl);
                self.regs[REG_L] = new_l;
                self.regs[REG_H] = new_h;
                addr
            }
            0x32 => {
                //LD (HL-) A
                let mut hl: u16 = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
                let addr = hl;
                hl = hl.wrapping_sub(1);
                let (new_h, new_l) = split_u16(hl);
                self.regs[REG_L] = new_l;
                self.regs[REG_H] = new_h;
                addr
            }
            0xE0 => {
                //LD (FF00 + XX) A
                let adding = adding_1 as u16;
                self.pc += 1;
                0xFF00 + adding
            }
            0xE2 => {
                // LD (FF00 + C) A
                0xFF00 + self.regs[REG_C] as u16
            }
            0xEA => {
                //LD (XX) A
                self.pc += 2;
                combine_bytes(adding_2, adding_1)
            }
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at ld_reg_addr_a!", command)
            ),
        };
        self.write_memory(addr as usize, self.regs[REG_A]);
        self.pc += 1;
    }
    fn inc_reg_16(&mut self, command: u8) {
        if command == 0x33 {
            //INC SP
            self.sp = self.sp.wrapping_add(1);
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
            let mut comb = combine_bytes(self.regs[r_high], self.regs[r_low]);
            comb = comb.wrapping_add(1);
            let (comb_high, comb_low) = split_u16(comb);
            self.regs[r_high] = comb_high;
            self.regs[r_low] = comb_low;
        }
        self.pc += 1;
    }
    fn inc_reg_8(&mut self, command: u8) {
        let val = match command {
            0x04 => {
                //INC B
                self.regs[REG_B] = self.regs[REG_B].wrapping_add(1);
                self.regs[REG_B]
            }
            0x14 => {
                //INC D
                self.regs[REG_D] = self.regs[REG_D].wrapping_add(1);
                self.regs[REG_D]
            }
            0x24 => {
                //INC H
                self.regs[REG_H] = self.regs[REG_H].wrapping_add(1);
                self.regs[REG_H]
            }
            0x34 => {
                //INC (HL)
                let addr: usize = combine_bytes(self.regs[REG_H], self.regs[REG_L]) as usize;
                let mut val = self.get_memory(addr);
                val = val.wrapping_add(1);
                self.write_memory(addr, val);
                val
            }
            0x0C => {
                //INC C
                self.regs[REG_C] = self.regs[REG_C].wrapping_add(1);
                self.regs[REG_C]
            }
            0x1C => {
                //INC E
                self.regs[REG_E] = self.regs[REG_E].wrapping_add(1);
                self.regs[REG_E]
            }
            0x2C => {
                //INC L
                self.regs[REG_L] = self.regs[REG_L].wrapping_add(1);
                self.regs[REG_L]
            }
            0x3C => {
                //INC A
                self.regs[REG_A] = self.regs[REG_A].wrapping_add(1);
                self.regs[REG_A]
            }
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at inc_reg_8!", command)
            ),
        };
        self.z_flag = if val == 0 { 1 } else { 0 };
        self.h_flag = if (val & 0xF) == 0 { 1 } else { 0 };
        self.n_flag = 0;
        self.pc += 1;
    }
    fn dec_reg_8(&mut self, command: u8) {
        let val = match command {
            0x05 => {
                //DEC B
                self.regs[REG_B] = self.regs[REG_B].wrapping_sub(1);
                self.regs[REG_B]
            }
            0x15 => {
                //DEC D
                self.regs[REG_D] = self.regs[REG_D].wrapping_sub(1);
                self.regs[REG_D]
            }
            0x25 => {
                //DEC H
                self.regs[REG_H] = self.regs[REG_H].wrapping_sub(1);
                self.regs[REG_H]
            }
            0x35 => {
                //DEC (HL)
                let addr: usize = combine_bytes(self.regs[REG_H], self.regs[REG_L]) as usize;
                let mut val = self.get_memory(addr);
                val = val.wrapping_sub(1);
                self.write_memory(addr, val);
                val
            }
            0x0D => {
                //DEC C
                self.regs[REG_C] = self.regs[REG_C].wrapping_sub(1);
                self.regs[REG_C]
            }
            0x1D => {
                //DEC E
                self.regs[REG_E] = self.regs[REG_E].wrapping_sub(1);
                self.regs[REG_E]
            }
            0x2D => {
                //DEC L
                self.regs[REG_L] = self.regs[REG_L].wrapping_sub(1);
                self.regs[REG_L]
            }
            0x3D => {
                //DEC A
                self.regs[REG_A] = self.regs[REG_A].wrapping_sub(1);
                self.regs[REG_A]
            }
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at dec_reg_8!", command)
            ),
        };
        self.z_flag = if val == 0 { 1 } else { 0 };
        self.h_flag = if (val & 0xF) == 0xF { 1 } else { 0 };
        self.n_flag = 1;
        self.pc = self.pc.wrapping_add(1);
    }
    fn ld_reg_8(&mut self, command: u8) {
        let to_load = self.get_memory((self.pc + 1) as usize);

        match command {
            0x06 => {
                //LD B XX
                self.regs[REG_B] = to_load;
            }
            0x16 => {
                //LD D XX
                self.regs[REG_D] = to_load;
            }
            0x26 => {
                //LD H XX
                self.regs[REG_H] = to_load;
            }
            0x36 => {
                //LD (HL) XX
                let addr: usize = combine_bytes(self.regs[REG_H], self.regs[REG_L]) as usize;
                self.write_memory(addr, to_load);
            }
            0x0E => {
                //LD C
                self.regs[REG_C] = to_load;
            }
            0x1E => {
                //LD E
                self.regs[REG_E] = to_load;
            }
            0x2E => {
                //LD L
                self.regs[REG_L] = to_load;
            }
            0x3E => {
                //LD A
                self.regs[REG_A] = to_load;
            }
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at ld_reg_8!", command)
            ),
        };
        self.pc += 2;
    }
    fn rlca(&mut self, _command: u8) {
        let bit = self.regs[REG_A] >> 7;
        self.regs[REG_A] <<= 1;
        self.regs[REG_A] += bit;
        self.z_flag = 0;
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = bit;
        self.pc += 1;
    }
    fn rla(&mut self, _command: u8) {
        let last_bit = self.regs[REG_A] >> 7;
        self.regs[REG_A] <<= 1;
        self.regs[REG_A] += self.c_flag;
        self.z_flag = 0;
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = last_bit;
        self.pc += 1;
    }
    fn daa(&mut self, _command: u8) {
        // if self.n_flag == 0 {
        //     // after an addition, adjust if (half-)carry occurred or if result is out of bounds
        //     if self.c_flag == 1 || self.regs[REG_A] > 0x99 {
        //         self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x60);
        //         self.c_flag = 1;
        //     }
        //     if self.h_flag == 1 || (self.regs[REG_A] & 0x0F) > 0x09 {
        //         self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x6);
        //     }
        // } else {
        //     // after a subtraction, only adjust if (half-)carry occurred
        //     if self.c_flag == 1 {
        //         self.regs[REG_A] = self.regs[REG_A].wrapping_sub(0x60);
        //     }
        //     if self.h_flag == 1 {
        //         self.regs[REG_A] = self.regs[REG_A].wrapping_sub(0x6);
        //     }
        // }
        // self.pc += 1;
        // self.z_flag = if self.regs[REG_A] == 0 { 1 } else { 0 };
        // self.h_flag = 0;
        let mut correction = 0;
        if self.h_flag == 1 || ((self.n_flag == 0) && ((self.regs[REG_A] & 0xF) > 0x9)) {
            correction |= 0x6;
        }
        if self.c_flag == 1 || ((self.n_flag == 0) && (self.regs[REG_A] > 0x99)) {
            correction |= 0x60;
            self.c_flag = 1;
        }
        self.regs[REG_A] = if self.n_flag == 0 {
            self.regs[REG_A].wrapping_add(correction)
        } else {
            self.regs[REG_A].wrapping_sub(correction)
        };
        self.z_flag = if self.regs[REG_A] == 0 { 1 } else { 0 };
        self.h_flag = 0;
        self.pc += 1;
    }
    fn scf(&mut self, _command: u8) {
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = 1;
        self.pc += 1;
    }
    fn ld_addr_sp(&mut self, _command: u8) {
        let addr_low = self.get_memory((self.pc + 1) as usize);
        let addr_high = self.get_memory((self.pc + 2) as usize);
        let addr = combine_bytes(addr_high, addr_low) as usize;
        let (high_sp, low_sp) = split_u16(self.sp);
        self.write_memory(addr, low_sp);
        self.write_memory(addr + 1, high_sp);
        self.pc += 3;
    }
    fn jr(&mut self, command: u8) {
        let add = self.get_memory((self.pc + 1) as usize);
        self.pc += 2;
        let condition = match command {
            0x18 => true,             //JR XX
            0x20 => self.z_flag == 0, //JR NZ XX
            0x28 => self.z_flag == 1, //JR Z XX
            0x30 => self.c_flag == 0, //JR NC XX
            0x38 => self.c_flag == 1, //JR C XX
            _ => panic!("{}", format!("Unrecognized command {:X} at jr!", command)),
        };
        if condition {
            self.cycle_modification = 12;
            self.pc = (self.pc as i32 + (add as i8) as i32) as u16;
            //self.pc = self.pc.wrapping_add((add as i8) as u16);
        }
    }
    fn add_hl_reg_16(&mut self, command: u8) {
        let mut hl = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
        let reg16 = match command {
            0x09 => combine_bytes(self.regs[REG_B], self.regs[REG_C]), //ADD HL, BC

            0x19 => combine_bytes(self.regs[REG_D], self.regs[REG_E]), //ADD HL, DE

            0x29 => combine_bytes(self.regs[REG_H], self.regs[REG_L]), //ADD HL, HL

            0x39 => self.sp, //ADD HL, SP
            _ => panic!("{}", format!("Unrecognized command {:X} at jr!", command)),
        };
        self.add_set_flags_16(&(hl as u32), &(reg16 as u32), false, true, true);
        hl = hl.wrapping_add(reg16);
        let (h_new, l_new) = split_u16(hl);
        self.regs[REG_H] = h_new;
        self.regs[REG_L] = l_new;
        self.n_flag = 0;
        self.pc += 1;
    }
    fn ld_a_addr(&mut self, command: u8) {
        let addr_low = self.get_memory((self.pc + 1) as usize);
        let addr_high = self.get_memory((self.pc + 2) as usize);
        let addr = match command {
            0x0A => combine_bytes(self.regs[REG_B], self.regs[REG_C]), //LD A (BC)

            0x1A => combine_bytes(self.regs[REG_D], self.regs[REG_E]), //LD A (DE)

            0x2A => {
                //LD A (HL +)
                let hl_old: u16 = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
                let hl_new = hl_old.wrapping_add(1);
                let (h_new, l_new) = split_u16(hl_new);
                self.regs[REG_L] = l_new;
                self.regs[REG_H] = h_new;
                hl_old
            }
            0x3A => {
                //LD A (HL -)
                let hl_old: u16 = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
                let hl_new = hl_old.wrapping_sub(1);
                let (h_new, l_new) = split_u16(hl_new);
                self.regs[REG_L] = l_new;
                self.regs[REG_H] = h_new;
                hl_old
            }
            0xF0 => {
                //LD A (FF00 + XX)
                self.pc += 1;
                0xFF00 + (addr_low as u16)
            }
            0xF2 => 0xFF00 + (self.regs[REG_C] as u16), //LD A (FF00 + C)

            0xFA => {
                //LD A (XX)
                let addr = combine_bytes(addr_high, addr_low);
                self.pc += 2;
                addr
            }
            _ => panic!(
                "{}",
                format!("Unrecognized command {:X} at ld_a_reg_addr!", command)
            ),
        };
        let new_val = self.get_memory(addr as usize);
        self.regs[REG_A] = new_val;
        self.pc += 1;
    }
    fn dec_reg_16(&mut self, command: u8) {
        if command == 0x3B {
            //DEC SP
            self.sp = self.sp.wrapping_sub(1);
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
            let mut comb = combine_bytes(self.regs[r_high], self.regs[r_low]);
            comb = comb.wrapping_sub(1);
            let (comb_high, comb_low) = split_u16(comb);
            self.regs[r_high] = comb_high;
            self.regs[r_low] = comb_low;
        }
        self.pc += 1;
    }
    fn rrca(&mut self, _command: u8) {
        let bit = self.regs[REG_A] & 1;
        self.regs[REG_A] >>= 1;
        self.regs[REG_A] += bit << 7;

        self.z_flag = 0;
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = bit;
        self.pc += 1;
    }
    fn rra(&mut self, _command: u8) {
        let bit = self.regs[REG_A] & 1;
        self.regs[REG_A] >>= 1;
        self.regs[REG_A] += self.c_flag << 7;
        self.z_flag = 0;
        self.n_flag = 0;
        self.h_flag = 0;
        self.c_flag = bit;
        self.pc += 1;
    }
    fn cpl(&mut self, _command: u8) {
        self.n_flag = 1;
        self.h_flag = 1;
        self.regs[REG_A] = !self.regs[REG_A];
        self.pc += 1;
    }
    fn ccf(&mut self, _command: u8) {
        self.c_flag = if self.c_flag == 1 { 0 } else { 1 };
        self.n_flag = 0;
        self.h_flag = 0;
        self.pc += 1;
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
        self.regs[reg_1] = self.regs[reg_2];
        self.pc += 1;
    }
    fn ld_reg_hl_addr(&mut self, command: u8) {
        let (command_high, command_low) = split_byte(command);
        let addr = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
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
        let new_val = self.get_memory(addr as usize);
        self.regs[reg] = new_val;
        self.pc += 1;
    }
    fn ld_hl_addr_reg(&mut self, command: u8) {
        let addr = combine_bytes(self.regs[REG_H], self.regs[REG_L]) as usize;
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
        self.pc += 1;
        self.write_memory(addr, self.regs[reg]);
    }
    fn halt(&mut self, _command: u8) {
        self.halting = true;
        if *self.interrupt_flag.lock().unwrap() & *self.interrupt_enable.lock().unwrap() != 0 {
            self.pc += 1;
            self.halting = false;
            if *self.ime.lock().unwrap() == 0 {
                self.repeat = true;
            }
        }
    }
    fn arthimetic_a(&mut self, command: u8) {
        let additional_val = self.get_memory((self.pc + 1) as usize);
        let (command_high, command_low) = split_byte(command);
        let op_val = if command_high <= 0xB {
            match command_low % 8 {
                0x0 => self.regs[REG_B],
                0x1 => self.regs[REG_C],
                0x2 => self.regs[REG_D],
                0x3 => self.regs[REG_E],
                0x4 => self.regs[REG_H],
                0x5 => self.regs[REG_L],
                0x6 => {
                    let addr = combine_bytes(self.regs[REG_H], self.regs[REG_L]) as usize;
                    self.get_memory(addr)
                }
                0x7 => self.regs[REG_A],
                _ => panic!(
                    "{}",
                    format!(
                        "Unrecognized subcommand {:X} at arthimetic!",
                        command_low % 8
                    )
                ),
            }
        } else {
            self.pc += 1;
            additional_val
        };
        let command_high_mod = (command_high - 0x8) % 4;
        let (additional, second_half) = if command_low >= 0x8 {
            if command_high_mod != 0x3 {
                (self.c_flag, true)
            } else {
                (0, true)
            }
        } else {
            (0, false)
        };
        let zero_val = if !second_half && command_high_mod == 0x3 {
            // OR A
            self.regs[REG_A] |= op_val;
            self.n_flag = 0;
            self.h_flag = 0;
            self.c_flag = 0;
            self.regs[REG_A]
        } else {
            match command_high_mod {
                0x0 => {
                    //ADD/ADC A
                    self.h_flag = if ((self.regs[REG_A] & 0xF) + (op_val & 0xF) + additional) & 0x10
                        == 0x10
                    {
                        1
                    } else {
                        0
                    };
                    self.c_flag = if self.regs[REG_A] as u16 + op_val as u16 + additional as u16
                        > CARRY_LIMIT_8
                    {
                        1
                    } else {
                        0
                    };
                    self.regs[REG_A] = self.regs[REG_A]
                        .wrapping_add(op_val)
                        .wrapping_add(additional);
                    self.n_flag = 0;
                    self.regs[REG_A]
                }
                0x1 | 0x3 => {
                    //SUB/SBC/CP
                    self.h_flag = if ((self.regs[REG_A] & 0xF)
                        .wrapping_sub(op_val & 0xF)
                        .wrapping_sub(additional))
                        & 0x10
                        == 0x10
                    {
                        1
                    } else {
                        0
                    };
                    self.c_flag = if (op_val as u16 + additional as u16) > self.regs[REG_A] as u16 {
                        1
                    } else {
                        0
                    };

                    let new_a = self.regs[REG_A]
                        .wrapping_sub(op_val)
                        .wrapping_sub(additional);
                    if command_high_mod == 0x1 {
                        self.regs[REG_A] = new_a;
                    }
                    self.n_flag = 1;
                    new_a
                }
                0x2 => {
                    if second_half {
                        //XOR A
                        self.regs[REG_A] ^= op_val;
                        self.h_flag = 0;
                    } else {
                        //AND A
                        self.regs[REG_A] &= op_val;
                        self.h_flag = 1;
                    }
                    self.n_flag = 0;
                    self.c_flag = 0;
                    self.regs[REG_A]
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
        self.z_flag = if zero_val == 0 { 1 } else { 0 };
        self.pc += 1;
    }
    fn ret(&mut self, command: u8) {
        self.pc += 1;
        let condition = match command {
            0xC0 => {
                //RET NZ
                if self.z_flag == 0 {
                    self.cycle_modification = 20;
                    true
                } else {
                    false
                }
            }
            0xD0 => {
                //RET NC
                if self.c_flag == 0 {
                    self.cycle_modification = 20;
                    true
                } else {
                    false
                }
            }
            0xC8 => {
                //RET Z
                if self.z_flag == 1 {
                    self.cycle_modification = 20;
                    true
                } else {
                    false
                }
            }
            0xD8 => {
                //RET C
                if self.c_flag == 1 {
                    self.cycle_modification = 20;
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

                *self.ime.lock().unwrap() = 1;
                true
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at ret!", command)),
        };
        if condition {
            let [addr_low, addr_high] = self.pop_stack();
            let addr = combine_bytes(addr_high, addr_low);
            self.pc = addr;
        }
    }
    fn pop(&mut self, command: u8) {
        let [first_val, second_val] = self.pop_stack();
        self.pc += 1;
        match command {
            0xC1 => {
                self.regs[REG_C] = first_val;
                self.regs[REG_B] = second_val;
            }
            0xD1 => {
                self.regs[REG_E] = first_val;
                self.regs[REG_D] = second_val;
            }
            0xE1 => {
                self.regs[REG_L] = first_val;
                self.regs[REG_H] = second_val;
            }
            0xF1 => {
                self.write_f(first_val);
                self.regs[REG_A] = second_val;
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at pop!", command)),
        }
    }
    fn push(&mut self, command: u8) {
        self.pc += 1;
        let (high_val, low_val) = match command {
            0xC5 => (self.regs[REG_B], self.regs[REG_C]),
            0xD5 => (self.regs[REG_D], self.regs[REG_E]),
            0xE5 => (self.regs[REG_H], self.regs[REG_L]),
            0xF5 => (self.regs[REG_A], self.get_f()),
            _ => panic!("{}", format!("Unrecognized command {:X} at push!", command)),
        };
        self.push_stack(high_val, low_val);
    }
    fn call(&mut self, command: u8) {
        let addr_low = self.get_memory((self.pc + 1) as usize);
        let addr_high = self.get_memory((self.pc + 2) as usize);
        let addr = combine_bytes(addr_high, addr_low);
        self.pc += 3;
        let (pc_high, pc_low) = split_u16(self.pc);
        match command {
            0xC4 => {
                //CALL NZ
                if self.z_flag == 0 {
                    self.push_stack(pc_high, pc_low);
                    self.pc = addr;
                    self.cycle_modification = 24;
                }
            }
            0xD4 => {
                //CALL NC
                if self.c_flag == 0 {
                    self.push_stack(pc_high, pc_low);
                    self.pc = addr;
                    self.cycle_modification = 24;
                }
            }
            0xCC => {
                //CALL Z
                if self.z_flag == 1 {
                    self.push_stack(pc_high, pc_low);
                    self.pc = addr;
                    self.cycle_modification = 24;
                }
            }
            0xDC => {
                //CALL C
                if self.c_flag == 1 {
                    self.push_stack(pc_high, pc_low);
                    self.pc = addr;
                    self.cycle_modification = 24;
                }
            }
            0xCD => {
                //CALL
                self.push_stack(pc_high, pc_low);
                self.pc = addr;
                self.cycle_modification = 24;
            }
            _ => panic!("{}", format!("Unrecognized command {:X} at call!", command)),
        }
    }
    fn rst(&mut self, command: u8) {
        let (high_command, low_command) = split_byte(command);
        self.pc += 1;
        let (high_pc, low_pc) = split_u16(self.pc);
        self.push_stack(high_pc, low_pc);
        self.pc = if low_command == 0xF {
            16 * (high_command as u16 - 0xC) + 8
        } else {
            16 * (high_command as u16 - 0xC)
        };
    }
    fn jp(&mut self, command: u8) {
        let low_byte = self.get_memory((self.pc + 1) as usize);
        let high_byte = self.get_memory((self.pc + 2) as usize);
        let addr = combine_bytes(high_byte, low_byte);
        let condition = match command {
            0xC2 => self.z_flag == 0, //JP NZ
            0xD2 => self.c_flag == 0, //JP NC
            0xC3 => true,             //JP
            0xCA => self.z_flag == 1, //JP Z
            0xDA => self.c_flag == 1, //JP C
            _ => panic!("{}", format!("Unrecognized command {:X} at jp!", command)),
        };
        self.pc = if condition {
            self.cycle_modification = 16;
            addr
        } else {
            self.pc + 3
        };
    }
    fn jp_hl(&mut self, _command: u8) {
        self.pc = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
    }
    fn add_sp_i8(&mut self, _command: u8) {
        let val = self.get_memory((self.pc + 1) as usize) as i8;
        let new_sp = self.sp.wrapping_add(val as u16);
        self.h_flag = if val >= 0 {
            if (self.sp & 0xF) as i8 + (val & 0xF) > 0xF {
                1
            } else {
                0
            }
        } else {
            if new_sp & 0xF <= self.sp & 0xF {
                1
            } else {
                0
            }
        };

        self.c_flag = if val >= 0 {
            if (self.sp & 0xFF) as i16 + val as i16 > 0xFF {
                1
            } else {
                0
            }
        } else {
            if new_sp & 0xFF <= (self.sp & 0xFF) {
                1
            } else {
                0
            }
        };
        self.sp = new_sp;

        // if (val as i8) < 0 {
        //     let minus_val: u8 = ((val as i8) * -1).try_into().unwrap();
        //     self.h_flag = if (((self.sp & 0xF) as u8 + (minus_val & 0xF)) & 0x10) == 0x10 {
        //         1
        //     } else {
        //         0
        //     };
        //     self.c_flag = if self.sp < minus_val as u16 { 1 } else { 0 };
        // } else {
        //     self.add_set_flags_16(&(val as u32), &(self.sp as u32), false, true, true);
        // };

        self.z_flag = 0;
        self.n_flag = 0;
        self.pc += 2;
    }
    fn ld_hl_sp_i8(&mut self, _command: u8) {
        let val = self.get_memory((self.pc + 1) as usize) as i8;
        let new_hl = self.sp.wrapping_add(val as u16);
        self.h_flag = if val >= 0 {
            if (self.sp & 0xF) as i8 + (val & 0xF) > 0xF {
                1
            } else {
                0
            }
        } else {
            if new_hl & 0xF <= self.sp & 0xF {
                1
            } else {
                0
            }
        };

        self.c_flag = if val >= 0 {
            if (self.sp & 0xFF) as i16 + val as i16 > 0xFF {
                1
            } else {
                0
            }
        } else {
            if new_hl & 0xFF <= (self.sp & 0xFF) {
                1
            } else {
                0
            }
        };
        let (hl_val_high, hl_val_low) = split_u16(new_hl);
        self.regs[REG_H] = hl_val_high;
        self.regs[REG_L] = hl_val_low;
        self.z_flag = 0;
        self.n_flag = 0;
        self.pc += 2;
    }
    fn ld_sp_hl(&mut self, _command: u8) {
        self.sp = combine_bytes(self.regs[REG_H], self.regs[REG_L]);
        self.pc += 1;
    }
    fn ei(&mut self, _command: u8) {
        self.reenable_interrupts = true;
        self.pc += 1;
    }
    fn di(&mut self, _command: u8) {
        *self.ime.lock().unwrap() = 0;
        self.pc += 1;
    }
    fn cb(&mut self, _command: u8) {
        let mut addr_val_ref = 0;
        let mut mem = false;
        let cb_command = self.get_memory((self.pc + 1) as usize);
        let (cb_command_high, cb_command_low) = split_byte(cb_command);
        let cb_command_low_second_half = cb_command_low >= 0x8;
        let bit_num = if cb_command_low_second_half {
            (cb_command_high % 4) * 2 + 1
        } else {
            (cb_command_high % 4) * 2
        };

        let reg = match cb_command_low % 8 {
            0x0 => &mut self.regs[REG_B],
            0x1 => &mut self.regs[REG_C],
            0x2 => &mut self.regs[REG_D],
            0x3 => &mut self.regs[REG_E],
            0x4 => &mut self.regs[REG_H],
            0x5 => &mut self.regs[REG_L],
            0x6 => {
                addr_val_ref =
                    self.get_memory(combine_bytes(self.regs[REG_H], self.regs[REG_L]) as usize);
                mem = true;
                &mut addr_val_ref
            }

            0x7 => &mut self.regs[REG_A],
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

                self.c_flag = moved_bit;
                self.h_flag = 0;
                self.n_flag = 0;
                self.z_flag = if *reg == 0 { 1 } else { 0 };
            }
            1 => {
                let moved_bit = if cb_command_low_second_half {
                    //rr
                    let bit_0 = *reg & 1;
                    *reg >>= 1;
                    *reg += self.c_flag << 7;
                    bit_0
                } else {
                    //rl
                    let bit_7 = (*reg >> 7) & 1;
                    *reg <<= 1;
                    *reg += self.c_flag;
                    bit_7
                };

                self.c_flag = moved_bit;
                self.h_flag = 0;
                self.n_flag = 0;
                self.z_flag = if *reg == 0 { 1 } else { 0 };
            }
            2 => {
                let moved_bit = if cb_command_low_second_half {
                    //sra
                    let bit_7 = (*reg >> 7) & 1;
                    let bit_0 = *reg & 1;
                    *reg >>= 1;
                    *reg += bit_7 << 7;
                    bit_0
                } else {
                    //sla
                    let bit_7 = (*reg >> 7) & 1;
                    *reg <<= 1;
                    bit_7
                };
                self.c_flag = moved_bit;
                self.h_flag = 0;
                self.n_flag = 0;
                self.z_flag = if *reg == 0 { 1 } else { 0 };
            }
            0x3 => {
                if cb_command_low_second_half {
                    //srl
                    let bit_0 = *reg & 1;
                    self.c_flag = bit_0;
                    *reg >>= 1;
                    self.h_flag = 0;
                    self.n_flag = 0;
                } else {
                    //swap
                    let (high_nib, low_nib) = split_byte(*reg);
                    *reg = (low_nib << 4) + high_nib;
                    self.c_flag = 0;
                    self.h_flag = 0;
                    self.n_flag = 0;
                }
                self.z_flag = if *reg == 0 { 1 } else { 0 };
            }
            4..=7 => {
                self.z_flag = 1 - ((*reg >> bit_num) & 1);
                self.n_flag = 0;
                self.h_flag = 1;
            }
            0x8..=0xB => {
                *reg &= 255 - 2u8.pow(bit_num as u32);
            }
            0xC..=0xF => {
                *reg |= 1 << bit_num;
            }
            _ => panic!(
                "{}",
                format!("Unrecognized subcommand {:X} at CB!", cb_command)
            ),
        };
        if mem {
            self.write_memory(
                combine_bytes(self.regs[REG_H], self.regs[REG_L]) as usize,
                addr_val_ref,
            );
        }
        self.pc += 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn get_blank_cpu() -> CentralProcessingUnit {
        let cycle_count = Arc::new(Mutex::new(0i32));
        let cycle_cond = Arc::new(Condvar::new());
        let dma_cond = Arc::new(Condvar::new());
        let interrupt_cond = Arc::new(Condvar::new());
        let rom = Arc::new(Mutex::new(Vec::<u8>::new()));
        let external_ram = Arc::new(Mutex::new([0u8; 131072]));
        let internal_ram = Arc::new(Mutex::new([0u8; 8192]));
        let rom_bank = Arc::new(Mutex::new(0usize));
        let ram_bank = Arc::new(Mutex::new(0usize));
        let lcdc = Arc::new(Mutex::new(0u8));
        let stat = Arc::new(Mutex::new(0u8));
        let vram = Arc::new(Mutex::new([0u8; 8192]));
        let oam = Arc::new(Mutex::new([0u8; 160]));
        let scy = Arc::new(Mutex::new(0u8));
        let scx = Arc::new(Mutex::new(0u8));
        let ly = Arc::new(Mutex::new(0u8));
        let lyc = Arc::new(Mutex::new(0u8));
        let wy = Arc::new(Mutex::new(0u8));
        let wx = Arc::new(Mutex::new(7u8));
        let bgp = Arc::new(Mutex::new(0u8));
        let ime = Arc::new(Mutex::new(0u8));
        let interrupt_enable = Arc::new(Mutex::new(0u8));
        let interrupt_flag = Arc::new(Mutex::new(0u8));
        let p1 = Arc::new(Mutex::new(0u8));
        let div = Arc::new(Mutex::new(0u8));
        let tima = Arc::new(Mutex::new(0u8));
        let tma = Arc::new(Mutex::new(0u8));
        let tac = Arc::new(Mutex::new(0u8));
        let obp0 = Arc::new(Mutex::new(0u8));
        let obp1 = Arc::new(Mutex::new(0u8));
        let dma_transfer = Arc::new(Mutex::new(false));
        let dma_register = Arc::new(Mutex::new(0u8));
        CentralProcessingUnit::new(
            rom,
            external_ram,
            internal_ram,
            rom_bank,
            ram_bank,
            lcdc,
            stat,
            vram,
            oam,
            scy,
            scx,
            ly,
            lyc,
            wy,
            wx,
            bgp,
            ime,
            p1,
            div,
            tima,
            tma,
            tac,
            obp0,
            obp1,
            dma_transfer,
            dma_register,
            interrupt_enable,
            interrupt_flag,
            cycle_count,
            cycle_cond,
            dma_cond,
            interrupt_cond,
        )
    }
    #[test]
    fn initial_test() {
        let mut cpu_instance = get_blank_cpu();
        cpu_instance.z_flag = 1;
        cpu_instance.c_flag = 0;
        cpu_instance.h_flag = 1;
        cpu_instance.n_flag = 1;
        cpu_instance.pc = 0;
        cpu_instance.regs[REG_B] = 0b01000001;
        cpu_instance.regs[REG_C] = 0xFF;
        cpu_instance.regs[REG_H] = 0xFF;
        cpu_instance.regs[REG_L] = 0xFE;
        cpu_instance.high_ram[126] = 0b11001100;
        cpu_instance.sp = 0xFFFE;
        (*cpu_instance.rom.lock().unwrap()).extend([0xCB, 0x16, 0x00, 0xFE, 0x00, 0b11001100]);

        for _ in 0..1 {
            cpu_instance.process();
        }
        //assert_eq!(cpu_instance.regs[REG_A], 0b1000);
        assert_eq!(cpu_instance.high_ram[126], 0b10011000);
        assert_eq!(cpu_instance.h_flag, 0);
        assert_eq!(cpu_instance.c_flag, 1);
        assert_eq!(cpu_instance.n_flag, 0);
    }
}

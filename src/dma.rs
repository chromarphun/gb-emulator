use crate::cycle_count_mod;
use std::sync::{Arc, Condvar, Mutex};

const DMA_DOTS: i32 = 640;

pub struct DirectMemoryAccess {
    dma_register: Arc<Mutex<u8>>,
    dma_transfer: Arc<Mutex<bool>>,
    vram: Arc<Mutex<[u8; 8192]>>,
    oam: Arc<Mutex<[u8; 160]>>,
    rom: Arc<Mutex<Vec<u8>>>,
    external_ram: Arc<Mutex<[u8; 131072]>>,
    internal_ram: Arc<Mutex<[u8; 8192]>>,
    rom_bank: Arc<Mutex<usize>>,
    ram_bank: Arc<Mutex<usize>>,
    cycle_count: Arc<Mutex<i32>>,
    cycle_cond: Arc<Condvar>,
    dma_cond: Arc<Condvar>,
}

impl DirectMemoryAccess {
    pub fn new(
        dma_register: Arc<Mutex<u8>>,
        dma_transfer: Arc<Mutex<bool>>,
        vram: Arc<Mutex<[u8; 8192]>>,
        oam: Arc<Mutex<[u8; 160]>>,
        rom: Arc<Mutex<Vec<u8>>>,
        external_ram: Arc<Mutex<[u8; 131072]>>,
        internal_ram: Arc<Mutex<[u8; 8192]>>,
        rom_bank: Arc<Mutex<usize>>,
        ram_bank: Arc<Mutex<usize>>,
        cycle_count: Arc<Mutex<i32>>,
        cycle_cond: Arc<Condvar>,
        dma_cond: Arc<Condvar>,
    ) -> DirectMemoryAccess {
        DirectMemoryAccess {
            dma_register,
            dma_transfer,
            vram,
            oam,
            rom,
            external_ram,
            internal_ram,
            rom_bank,
            ram_bank,
            cycle_count,
            cycle_cond,
            dma_cond,
        }
    }
    pub fn run(&mut self) {
        let mut start_cycle_count = *self.cycle_count.lock().unwrap();
        loop {
            let mut dma_transfer = self.dma_transfer.lock().unwrap();
            while !*dma_transfer {
                dma_transfer = self.dma_cond.wait(dma_transfer).unwrap();
            }
            *dma_transfer = false;
            std::mem::drop(dma_transfer);
            {
                start_cycle_count = *self.cycle_count.lock().unwrap();
                *self.dma_transfer.lock().unwrap() = false;
                let vram = self.vram.lock().unwrap();
                let mut oam = self.oam.lock().unwrap();
                let rom = self.rom.lock().unwrap();
                let external_ram = self.external_ram.lock().unwrap();
                let internal_ram = self.internal_ram.lock().unwrap();
                let rom_bank = *self.rom_bank.lock().unwrap();
                let ram_bank = *self.ram_bank.lock().unwrap();
                let reg = *self.dma_register.lock().unwrap() as usize;
                let start_address = reg << 8;

                match reg >> 4 {
                    0x0..=0x3 => {
                        let end_address = (start_address + 0xA0) as usize;
                        oam.copy_from_slice(&rom[start_address..end_address]);
                    }
                    0x4..=0x7 => {
                        let adjusted_start_address = 16384 * rom_bank - 0x4000 + reg;
                        let adjusted_end_address = adjusted_start_address + 0xA0;
                        oam.copy_from_slice(&rom[adjusted_start_address..adjusted_end_address]);
                    }
                    0x8..=0x9 => {
                        let adjusted_start_address = start_address - 0x8000;
                        let adjusted_end_address = adjusted_start_address + 0xA0;
                        oam.copy_from_slice(&vram[adjusted_start_address..adjusted_end_address]);
                    }
                    0xA..=0xB => {
                        let adjusted_start_address = 8192 * ram_bank + start_address - 0xA000;
                        let adjusted_end_address = adjusted_start_address + 0xA0;
                        oam.copy_from_slice(
                            &external_ram[adjusted_start_address..adjusted_end_address],
                        );
                    }
                    0xC..=0xD => {
                        let adjusted_start_address = (start_address - 0xC000) as usize;
                        let adjusted_end_address = adjusted_start_address + 0xA0;
                        oam.copy_from_slice(
                            &internal_ram[adjusted_start_address..adjusted_end_address],
                        );
                    }
                    _ => {}
                }
                let mut current_cycle_count = self.cycle_count.lock().unwrap();
                while cycle_count_mod(*current_cycle_count - start_cycle_count) <= DMA_DOTS {
                    current_cycle_count = self.cycle_cond.wait(current_cycle_count).unwrap();
                }
            }
        }
    }
}

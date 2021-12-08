use std::sync::{Arc, Condvar, Mutex};

use crate::ADVANCE_CYCLES;

const DMA_DOTS: u32 = 640;

pub struct DirectMemoryAccess {
    dma_register: Arc<Mutex<u8>>,
    dma_transfer: Arc<Mutex<bool>>,
    vram: Arc<Mutex<[u8; 8192]>>,
    oam: Arc<Mutex<[u8; 160]>>,
    rom: Arc<Mutex<Vec<u8>>>,
    external_ram: Arc<Mutex<Vec<u8>>>,
    internal_ram: Arc<Mutex<[u8; 8192]>>,
    rom_bank: Arc<Mutex<usize>>,
    ram_bank: Arc<Mutex<usize>>,
    cycle_count: u32,

    starting: bool,
}

impl DirectMemoryAccess {
    pub fn new(
        dma_register: Arc<Mutex<u8>>,
        dma_transfer: Arc<Mutex<bool>>,
        vram: Arc<Mutex<[u8; 8192]>>,
        oam: Arc<Mutex<[u8; 160]>>,
        rom: Arc<Mutex<Vec<u8>>>,
        external_ram: Arc<Mutex<Vec<u8>>>,
        internal_ram: Arc<Mutex<[u8; 8192]>>,
        rom_bank: Arc<Mutex<usize>>,
        ram_bank: Arc<Mutex<usize>>,
    ) -> DirectMemoryAccess {
        let starting = true;
        let cycle_count = 0;
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
            starting,
        }
    }

    pub fn advance(&mut self) {
        let mut dma_transfer = self.dma_transfer.lock().unwrap();
        if *dma_transfer {
            if self.starting {
                let mut oam = self.oam.lock().unwrap();
                let reg = *self.dma_register.lock().unwrap() as usize;
                //println!("{}", format!("DMA TRANSFER {:X}", reg));
                let start_address = reg << 8;

                match reg >> 4 {
                    0x0..=0x3 => {
                        let end_address = (start_address + 0xA0) as usize;
                        oam.copy_from_slice(&self.rom.lock().unwrap()[start_address..end_address]);
                    }
                    0x4..=0x7 => {
                        let adjusted_start_address =
                            16384 * (*self.rom_bank.lock().unwrap()) - 0x4000 + reg;
                        let adjusted_end_address = adjusted_start_address + 0xA0;
                        oam.copy_from_slice(
                            &self.rom.lock().unwrap()[adjusted_start_address..adjusted_end_address],
                        );
                    }
                    0x8..=0x9 => {
                        let adjusted_start_address = start_address - 0x8000;
                        let adjusted_end_address = adjusted_start_address + 0xA0;
                        oam.copy_from_slice(
                            &self.vram.lock().unwrap()
                                [adjusted_start_address..adjusted_end_address],
                        );
                    }
                    0xA..=0xB => {
                        let adjusted_start_address =
                            8192 * (*self.ram_bank.lock().unwrap()) + start_address - 0xA000;
                        let adjusted_end_address = adjusted_start_address + 0xA0;
                        oam.copy_from_slice(
                            &self.external_ram.lock().unwrap()
                                [adjusted_start_address..adjusted_end_address],
                        );
                    }
                    0xC..=0xD => {
                        let adjusted_start_address = (start_address - 0xC000) as usize;
                        let adjusted_end_address = adjusted_start_address + 0xA0;
                        oam.copy_from_slice(
                            &self.internal_ram.lock().unwrap()
                                [adjusted_start_address..adjusted_end_address],
                        );
                    }
                    _ => println!("DMA FAILURE"),
                }
                self.starting = false;
                self.cycle_count += ADVANCE_CYCLES;
            } else {
                self.cycle_count += ADVANCE_CYCLES;
                if self.cycle_count == DMA_DOTS {
                    self.cycle_count = 0;
                    *dma_transfer = false;
                    self.starting = true;
                }
            }
        }
    }
}

pub const WINDOW_WIDTH: usize = 160;
pub const WINDOW_HEIGHT: usize = 144;

//CPU Specific Constants
pub const NUM_REG: usize = 7;
pub const REG_A: usize = 0;
pub const REG_B: usize = 1;
pub const REG_C: usize = 2;
pub const REG_D: usize = 3;
pub const REG_E: usize = 4;
pub const REG_H: usize = 5;
pub const REG_L: usize = 6;
pub const CARRY_LIMIT_16: u32 = 65535;
pub const CARRY_LIMIT_8: u16 = 255;
pub const INTERRUPT_DOTS: u32 = 20;

//Timing Constants
pub const PERIODS_PER_SECOND: u32 = 64;
pub const PERIOD_NS: u32 = 1_000_000_000 / PERIODS_PER_SECOND;
pub const CYCLES_PER_SECOND: u32 = 4_194_304;
pub const CYCLES_PER_PERIOD: u32 = CYCLES_PER_SECOND / PERIODS_PER_SECOND;
pub const ADVANCES_PER_PERIOD: u32 = CYCLES_PER_PERIOD / ADVANCE_CYCLES;
pub const ADVANCE_CYCLES: u32 = 4;
pub const SAMPLES_PER_SECOND: u32 = 44100;
pub const CYCLES_PER_SAMPLE: u32 = CYCLES_PER_SECOND / SAMPLES_PER_SECOND;

//Address Constants
pub const INT_FLAG_ADDR: usize = 0xFF0F;
pub const INT_ENABLE_ADDR: usize = 0xFFFF;
pub const NR10_ADDR: usize = 0xFF10;
pub const NR11_ADDR: usize = 0xFF11;
pub const NR12_ADDR: usize = 0xFF12;
pub const NR13_ADDR: usize = 0xFF13;
pub const NR14_ADDR: usize = 0xFF14;
pub const NR21_ADDR: usize = 0xFF16;
pub const NR22_ADDR: usize = 0xFF17;
pub const NR23_ADDR: usize = 0xFF18;
pub const NR24_ADDR: usize = 0xFF19;
pub const NR30_ADDR: usize = 0xFF1A;
pub const NR31_ADDR: usize = 0xFF1B;
pub const NR32_ADDR: usize = 0xFF1C;
pub const NR33_ADDR: usize = 0xFF1D;
pub const NR34_ADDR: usize = 0xFF1E;
pub const NR41_ADDR: usize = 0xFF20;
pub const NR42_ADDR: usize = 0xFF21;
pub const NR43_ADDR: usize = 0xFF22;
pub const NR44_ADDR: usize = 0xFF23;
pub const NR50_ADDR: usize = 0xFF24;
pub const NR51_ADDR: usize = 0xFF25;
pub const NR52_ADDR: usize = 0xFF26;
pub const P1_ADDR: usize = 0xFF00;
pub const LY_ADDR: usize = 0xFF44;
pub const LCDC_ADDR: usize = 0xFF40;
pub const STAT_ADDR: usize = 0xFF41;
pub const SCY_ADDR: usize = 0xFF42;
pub const SCX_ADDR: usize = 0xFF43;
pub const LYC_ADDR: usize = 0xFF45;
pub const BGP_ADDR: usize = 0xFF47;
pub const OBP0_ADDR: usize = 0xFF48;
pub const OBP1_ADDR: usize = 0xFF49;
pub const WY_ADDR: usize = 0xFF4A;
pub const WX_ADDR: usize = 0xFF4B;
pub const OAM_START_ADDR: usize = 0xFE00;
pub const VRAM_START_ADDR: usize = 0x8000;
pub const DIV_ADDR: usize = 0xFF04;
pub const TIMA_ADDR: usize = 0xFF05;
pub const TMA_ADDR: usize = 0xFF06;
pub const TAC_ADDR: usize = 0xFF07;
pub const CART_TYPE_ADDR: usize = 0x147;
pub const ROM_BANK_ADDR: usize = 0x148;
pub const RAM_BANK_ADDR: usize = 0x149;

//Memory Specific Constants
pub const VRAM_SIZE: usize = 0x2000;
pub const IRAM_SIZE: usize = 0x8000;
pub const OAM_SIZE: usize = 160;
pub const IO_SIZE: usize = 0x80;
pub const HRAM_SIZE: usize = 127;

//PPU Specific Constants
pub const TILES_PER_ROW: usize = 32;
pub const BG_MAP_SIZE_PX: usize = 256;
pub const TILE_WIDTH: usize = 8;
pub const BG_TILE_HEIGHT: usize = 8;
pub const BYTES_PER_TILE: usize = 16;
pub const BYTES_PER_TILE_ROW: usize = 2;
pub const VRAM_BLOCK_SIZE: usize = 128;
pub const OAM_SCAN_DOTS: u32 = 80;
pub const DRAWING_DOTS: u32 = 172;
pub const HBLANK_DOTS: u32 = 204;
pub const ROW_DOTS: u32 = 456;
pub const BYTES_PER_OAM_ENTRY: usize = 4;
pub const OAM_Y_INDEX: usize = 0;
pub const OAM_X_INDEX: usize = 1;
pub const OAM_TILE_INDEX: usize = 2;
pub const OAM_ATTRIBUTE_INDEX: usize = 3;

//APU Specific Constants
pub const DUTY_CONVERSION: [f32; 4] = [0.125, 0.25, 0.5, 0.75];
pub const VOLUME_SHIFT_CONVERSION: [u8; 4] = [4, 0, 1, 2];
pub const MAX_FREQ_VAL: u32 = 2048;
pub const MAX_8_LENGTH: u8 = 64;
pub const MAX_16_LENGTH: u16 = 256;
pub const CYCLE_COUNT_128HZ: u32 = 32768;
pub const CYCLE_COUNT_256HZ: u32 = 16384;
pub const CYCLE_COUNT_512HZ: u32 = 65536;
pub const CYCLE_COUNT_16384HZ: u32 = 256;

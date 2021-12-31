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
pub const HDMA1_ADDR: usize = 0xFF51;
pub const HDMA2_ADDR: usize = 0xFF52;
pub const HDMA3_ADDR: usize = 0xFF53;
pub const HDMA4_ADDR: usize = 0xFF54;
pub const HDMA5_ADDR: usize = 0xFF55;
pub const KEY1_ADDR: usize = 0xFF4D;
pub const SVBK_ADDR: usize = 0xFF70;
pub const VBK_ADDR: usize = 0xFF4F;

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
pub const DMG_COLOR_MAP: [[u8; 4]; 4] = [
    [155, 188, 15, 255],
    [139, 172, 15, 255],
    [48, 98, 48, 255],
    [15, 56, 15, 255],
];
pub const BG_LCDC_LOW_PRIORITY: u8 = 220;
pub const BG_COLOR_0_PRIORITY: u8 = 215;
pub const OAM_LOW_PRIORITY: u8 = 210;
pub const BG_COLOR_1_3_PRIORITY: u8 = 205;
pub const BG_HIGH_PRIORITY: u8 = 0;

//APU Specific Constants
pub const DUTY_CONVERSION: [f32; 4] = [0.125, 0.25, 0.5, 0.75];
pub const VOLUME_SHIFT_CONVERSION: [u8; 4] = [4, 0, 1, 2];
pub const MAX_FREQ_VAL: u32 = 2048;
pub const MAX_8_LENGTH: u16 = 64;
pub const MAX_16_LENGTH: u16 = 256;
pub const CYCLE_COUNT_64HZ: u32 = 65536;
pub const CYCLE_COUNT_128HZ: u32 = 32768;
pub const CYCLE_COUNT_256HZ: u32 = 16384;
pub const CYCLE_COUNT_512HZ: u32 = 8192;
pub const CH1_IND: usize = 0;
pub const CH2_IND: usize = 1;
pub const CH3_IND: usize = 2;
pub const CH4_IND: usize = 3;

pub const CYCLE_COUNT_16384HZ: u32 = 256;
pub const SAMPLE_BUFFER_SIZE: u16 = 128;

//Memory Specific Constants
pub const VRAM_SIZE: usize = 0x2000;
pub const IRAM_SIZE: usize = 0x8000;
pub const OAM_SIZE: usize = 160;
pub const IO_SIZE: usize = 0x80;
pub const HRAM_SIZE: usize = 127;
pub const NON_BLOCK_INVALID_IO: [usize; 24] = [
    0x03, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0x13, 0x15, 0x18, 0x1B, 0x1D, 0x1F, 0x20, 0x27, 0x28,
    0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F,
];
pub const NON_BLOCK_CGB_VALID_IO: [usize; 9] =
    [0x51, 0x52, 0x53, 0x54, 0x55, 0x4D, 0x4F, 0x6C, 0x70];

pub const FUNCTION_NAMES: [&str; 256] = [
    //0x0
    "NOP",
    "LD BC, u16",
    "LD (BC) A",
    "INC BC",
    "INC B",
    "DEC B",
    "LD B, u8",
    "RLCA",
    "LD (u16), SP",
    "ADD HL, BC",
    "LD A, (BC)",
    "DEC BC",
    "INC C",
    "DEC C",
    "LD C, u8",
    "RRCA",
    //0x1
    "STOP",
    "LD DE, u16",
    "LD (DE), A",
    "INC DE",
    "INC D",
    "DEC D",
    "LD D, u8",
    "RLA",
    "JR i8",
    "ADD HL, DE",
    "LD A, (DE)",
    "DEC DE",
    "INC E",
    "DEC E",
    "LD E, u8",
    "RRA",
    //0x2
    "JR NZ, i8",
    "LD HL, u16",
    "LD (HL+), A",
    "INC HL",
    "INC H",
    "DEC H",
    "LD H, u8",
    "DAA",
    "JR Z, i8",
    "ADD HL, HL",
    "LD A, (HL+)",
    "DEC HL",
    "INC L",
    "DEC L",
    "LD L, u8",
    "CPL",
    //0x3
    "JR NC, i8",
    "LD SP, u16",
    "LD (HL-), A",
    "INC SP",
    "INC (HL)",
    "DEC (HL)",
    "LD (HL), u8",
    "SCF",
    "JR C, i8",
    "ADD HL, SP",
    "LD A, (HL-)",
    "DEC SP",
    "INC A",
    "DEC A",
    "LD A, u8",
    "CCF",
    //0x4
    "LD B, B",
    "LD B, C",
    "LD B, D",
    "LD B, E",
    "LD B, H",
    "LD B, L",
    "LD B, (HL)",
    "LD B, A",
    "LD C, B",
    "LD C, C",
    "LD C, D",
    "LD C, E",
    "LD C, H",
    "LD C, L",
    "LD C, (HL)",
    "LD C, A",
    //0x5
    "LD D, B",
    "LD D, C",
    "LD D, D",
    "LD D, E",
    "LD D, H",
    "LD D, L",
    "LD D, (HL)",
    "LD D, A",
    "LD E, B",
    "LD E, C",
    "LD E, D",
    "LD E, E",
    "LD E, H",
    "LD E, L",
    "LD E, (HL)",
    "LD E, A",
    //0x6
    "LD H, B",
    "LD H, C",
    "LD H, D",
    "LD H, E",
    "LD H, H",
    "LD H, L",
    "LD H, (HL)",
    "LD H, A",
    "LD L, B",
    "LD L, C",
    "LD L, D",
    "LD L, E",
    "LD L, H",
    "LD L, L",
    "LD L, (HL)",
    "LD L, A",
    //0x7
    "LD (HL), B",
    "LD (HL), C",
    "LD (HL), D",
    "LD (HL), E",
    "LD (HL), H",
    "LD (HL), L",
    "HALT",
    "LD (HL), A",
    "LD A, B",
    "LD A, C",
    "LD A, D",
    "LD A, E",
    "LD A, H",
    "LD A, L",
    "LD A, (HL)",
    "LD A, A",
    //0x8
    "ADD A, B",
    "ADD A, C",
    "ADD A, D",
    "ADD A, E",
    "ADD A, H",
    "ADD A, L",
    "ADD A, (HL)",
    "ADD A, A",
    "ADC A, B",
    "ADC A, C",
    "ADC A, D",
    "ADC A, E",
    "ADC A, H",
    "ADC A, L",
    "ADC A, (HL)",
    "ADC A, A",
    //0x9
    "SUB A, B",
    "SUB A, C",
    "SUB A, D",
    "SUB A, E",
    "SUB A, H",
    "SUB A, L",
    "SUB A, (HL)",
    "SUB A, A",
    "SBC A, B",
    "SBC A, C",
    "SBC A, D",
    "SBC A, E",
    "SBC A, H",
    "SBC A, L",
    "SBC A, (HL)",
    "SBC A, A",
    //0xA
    "AND A, B",
    "AND A, C",
    "AND A, D",
    "AND A, E",
    "AND A, H",
    "AND A, L",
    "AND A, (HL)",
    "AND A, A",
    "XOR A, B",
    "XOR A, C",
    "XOR A, D",
    "XOR A, E",
    "XOR A, H",
    "XOR A, L",
    "XOR A, (HL)",
    "XOR A, A",
    //0xB
    "OR A, B",
    "OR A, C",
    "OR A, D",
    "OR A, E",
    "OR A, H",
    "OR A, L",
    "OR A, (HL)",
    "OR A, A",
    "CP A, B",
    "CP A, C",
    "CP A, D",
    "CP A, E",
    "CP A, H",
    "CP A, L",
    "CP A, (HL)",
    "CP A, A",
    //0xC
    "RET NZ",
    "POP BC",
    "JP NZ, u16",
    "JP u16",
    "CALL NZ, u16",
    "PUSH BC",
    "ADD A, u8",
    "RST 0x00",
    "RET Z",
    "RET",
    "JP Z, u16",
    "CB COM",
    "CALL Z, u16",
    "CALL",
    "ADC A, u8",
    "RST 0x00",
    //0xD
    "RET NC",
    "POP DE",
    "JP NC, u16",
    "FAIL",
    "CALL NC, u16",
    "PUSH DE",
    "SUB A, u8",
    "RST 0x10",
    "RET C",
    "RETI",
    "JP C, u16",
    "FAIL",
    "CALL C, u16",
    "FAIL",
    "SBC A, u8",
    "RST 0x18",
    //0xE
    "LD (0xFF00 + u8), A",
    "POP HL",
    "LD (0xFF00 + C), A",
    "FAIL",
    "FAIL",
    "PUSH HL",
    "AND A, u8",
    "RST 0x20",
    "ADD SP, i8",
    "JP HL",
    "LD (u16), A",
    "FAIL",
    "FAIL",
    "FAIL",
    "XOR A, u8",
    "RST 0x28",
    //0xF
    "LD A, (0xFF00 + u8)",
    "POP AF",
    "LD A, (0xFF00 + C)",
    "DI",
    "FAIL",
    "PUSH AF",
    "OR A, u8",
    "RST 0x30",
    "LD HL, SP + i8",
    "LD SP, HL",
    "LD A, (u16)",
    "EI",
    "FAIL",
    "FAIL",
    "CP A, u8",
    "RST 0x38",
];

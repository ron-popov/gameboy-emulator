// General
// pub const SCREEN_WIDTH: usize = 160;
// pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 256;
pub const GBEMULATOR_ASCII_ART: &str = "\n   _____ ____                       _       _             \n  / ____|  _ \\                     | |     | |            \n | |  __| |_) | ___ _ __ ___  _   _| | __ _| |_ ___  _ __ \n | | |_ |  _ < / _ \\ \'_ ` _ \\| | | | |/ _` | __/ _ \\| \'__|\n | |__| | |_) |  __/ | | | | | |_| | | (_| | || (_) | |   \n  \\_____|____/ \\___|_| |_| |_|\\__,_|_|\\__,_|\\__\\___/|_|   \n                                                          \n                                                          \n";

// PPU Debug flags
pub const PPU_DISABLE: bool = false;
pub const PPU_DUMP_SPRITES: bool = true;

// RAM STUFF
pub const RAM_ECHO_RANGE_START: u16 = 0xE000;
pub const RAM_SPRITE_ATTRIBUTE_TABLE_RANGE_START: u16 = 0xFE00;
pub const RAM_IO_PORTS_RANGE_START: u16 = 0xFF00;
pub const RAM_EMPTY_RANGE_START: u16 = 0xFF4C;
pub const RAM_INTERNAL_RANGE_START: u16 = 0xFF80;

pub const DMG_BOOT_ROM: [u8; 0x100] = [
    0x31, 0xfe, 0xff, 0xaf, 0x21, 0xff, 0x9f, 0x32, 0xcb, 0x7c, 0x20, 0xfb,
    0x21, 0x26, 0xff, 0x0e, 0x11, 0x3e, 0x80, 0x32, 0xe2, 0x0c, 0x3e, 0xf3,
    0xe2, 0x32, 0x3e, 0x77, 0x77, 0x3e, 0xfc, 0xe0, 0x47, 0x11, 0x04, 0x01,
    0x21, 0x10, 0x80, 0x1a, 0xcd, 0x95, 0x00, 0xcd, 0x96, 0x00, 0x13, 0x7b,
    0xfe, 0x34, 0x20, 0xf3, 0x11, 0xd8, 0x00, 0x06, 0x08, 0x1a, 0x13, 0x22,
    0x23, 0x05, 0x20, 0xf9, 0x3e, 0x19, 0xea, 0x10, 0x99, 0x21, 0x2f, 0x99,
    0x0e, 0x0c, 0x3d, 0x28, 0x08, 0x32, 0x0d, 0x20, 0xf9, 0x2e, 0x0f, 0x18,
    0xf3, 0x67, 0x3e, 0x64, 0x57, 0xe0, 0x42, 0x3e, 0x91, 0xe0, 0x40, 0x04,
    0x1e, 0x02, 0x0e, 0x0c, 0xf0, 0x44, 0xfe, 0x90, 0x20, 0xfa, 0x0d, 0x20,
    0xf7, 0x1d, 0x20, 0xf2, 0x0e, 0x13, 0x24, 0x7c, 0x1e, 0x83, 0xfe, 0x62,
    0x28, 0x06, 0x1e, 0xc1, 0xfe, 0x64, 0x20, 0x06, 0x7b, 0xe2, 0x0c, 0x3e,
    0x87, 0xe2, 0xf0, 0x42, 0x90, 0xe0, 0x42, 0x15, 0x20, 0xd2, 0x05, 0x20,
    0x4f, 0x16, 0x20, 0x18, 0xcb, 0x4f, 0x06, 0x04, 0xc5, 0xcb, 0x11, 0x17,
    0xc1, 0xcb, 0x11, 0x17, 0x05, 0x20, 0xf5, 0x22, 0x23, 0x22, 0x23, 0xc9,
    0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00, 0x83,
    0x00, 0x0c, 0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e,
    0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63,
    0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e,
    0x3c, 0x42, 0xb9, 0xa5, 0xb9, 0xa5, 0x42, 0x3c, 0x21, 0x04, 0x01, 0x11,
    0xa8, 0x00, 0x1a, 0x13, 0xbe, 0x20, 0xfe, 0x23, 0x7d, 0xfe, 0x34, 0x20,
    0xf5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xfb, 0x86, 0x20, 0xfe,
    0x3e, 0x01, 0xe0, 0x50
  ];
  

// Flag masks
pub const FLAG_ZERO_MASK: u8 = 0b10000000;
pub const FLAG_SUB_MASK: u8 = 0b01000000;
pub const FLAG_HALF_CARRY_MASK: u8 = 0b00100000;
pub const FLAG_CARRY_MASK: u8 = 0b00010000;

// External ram size
pub const CARTRIDGE_RAM_SIZE_NONE: u8 = 0x00;
// pub const CARTRIDGE_RAM_SIZE_2KB: u8 = 0x01;
// pub const CARTRIDGE_RAM_SIZE_8KB: u8 = 0x02;
// pub const CARTRIDGE_RAM_SIZE_32KB: u8 = 0x03;
// pub const CARTRIDGE_RAM_SIZE_128KB: u8 = 0x04;
// pub const CARTRIDGE_RAM_SIZE_64KB: u8 = 0x05;

pub const RAM_SIZE: usize = 0xFFFF;
pub const CARTRIDGE_ROM_SIZE_DEFAULT: usize = 32 * 1024;

// Rom size
pub const CARTRIDGE_ROM_SIZE_NO_BANKS: u8 = 0x00;
// pub const CARTRIDGE_ROM_SIZE_4_BANKS: u8 = 0x01;
// pub const CARTRIDGE_ROM_SIZE_8_BANKS: u8 = 0x02;
// pub const CARTRIDGE_ROM_SIZE_16_BANKS: u8 = 0x03;
// pub const CARTRIDGE_ROM_SIZE_32_BANKS: u8 = 0x04;
// pub const CARTRIDGE_ROM_SIZE_64_BANKS: u8 = 0x05;
// pub const CARTRIDGE_ROM_SIZE_128_BANKS: u8 = 0x06;
// pub const CARTRIDGE_ROM_SIZE_256_BANKS: u8 = 0x07;
// pub const CARTRIDGE_ROM_SIZE_512_BANKS: u8 = 0x08;
// pub const CARTRIDGE_ROM_SIZE_72_BANKS: u8 = 0x52;
// pub const CARTRIDGE_ROM_SIZE_80_BANKS: u8 = 0x53;
// pub const CARTRIDGE_ROM_SIZE_96_BANKS: u8 = 0x54;

// PPU Stuff
// --- PPU ADDR Ranges ---
pub const PPU_JOYPAD_INPUT_ADDR:  [u16; 0x01] = 
  [0xFF00];

pub const PPU_SERIAL_ADDR:        [u16; 0x02] = 
  [0xFF01, 0xFF02];

pub const PPU_TIMER_DIVIDER_ADDR: [u16; 0x04] = 
  [0xFF04, 0xFF05, 0xFF06, 0xFF07];

pub const PPU_AUDIO_ADDR:         [u16; 0x17] = 
  [0xFF10, 0xFF11, 0xFF12, 0xFF13, 0xFF14, 0xFF15, 0xFF16, 0xFF17, 0xFF18, 0xFF19, 0xFF1A, 0xFF1B, 0xFF1C, 0xFF1D, 0xFF1E, 0xFF1F, 0xFF20, 0xFF21, 0xFF22, 0xFF23, 0xFF24, 0xFF25, 0xFF26];

pub const PPU_WAVE_ADDR:          [u16; 0x10] = 
  [0xFF30, 0xFF31, 0xFF32, 0xFF33, 0xFF34, 0xFF35, 0xFF36, 0xFF37, 0xFF38, 0xFF39, 0xFF3A, 0xFF3B, 0xFF3C, 0xFF3D, 0xFF3E, 0xFF3F];

pub const PPU_LCD_ADDR:           [u16; 0x0C] = 
  [0xFF40, 0xFF41, 0xFF42, 0xFF43, 0xFF44, 0xFF45, 0xFF46, 0xFF47, 0xFF48, 0xFF49, 0xFF4A, 0xFF4B];
// TODO: .......

// --- PPU ADDR and each bit meaning ---
pub const PPU_ADDR_LCD_CONTROL: u16                                 = 0xFF40;
pub const PPU_LCD_CONTROL_BIT_ENABLE: u8                            = 7;
pub const PPU_LCD_CONTROL_BIT_WINDOW_TILE_MAP_AREA: u8              = 6;
pub const PPU_LCD_CONTROL_BIT_WINDOW_ENABLE: u8                     = 5;
pub const PPU_LCD_CONTROL_BIT_BG_AND_WINDOW_TILE_DATA_AREA: u8       = 4;
pub const PPU_LCD_CONTROL_BIT_BG_TILE_MAP_AREA: u8                  = 3;
pub const PPU_LCD_CONTROL_BIT_OBJ_SIZE: u8                          = 2;
pub const PPU_LCD_CONTROL_BIT_OBJ_ENABLE: u8                        = 1;
pub const PPU_LCD_CONTROL_BIT_BG_AND_WINDOW_PRIORITY: u8            = 0;

pub const PPU_OAM_ADDR: u16                                         = 0xFE00;

// Colors
pub const COLOR_WHITE: u32 = 0x00ffffff;
pub const COLOR_LIGHT_GREY: u32 = 0x00aaaaaa;
pub const COLOR_DARK_GREY: u32 = 0x00555555;
pub const COLOR_BLACK: u32 = 0x00000000;

pub const SPRITE_DUMP_PATH: &str = "sprites_dump.bmp";

// Cartridge Type
pub const CARTRIDGE_TYPE_ROM_ONLY: u8 = 0x00;

pub fn bit_check(value: u8, bit: u8) -> bool {
  let bit_mask: u8 = 0x01 << bit;
  return value & bit_mask == bit_mask;
}

pub fn bit_enable(value: u8, bit: u8) -> u8 {
  let bit_mask: u8 = 0x01 << bit;
  return value | bit_mask;
}

pub fn bit_disable(value: u8, bit: u8) -> u8 {
  let bit_mask: u8 = !(0x01 << bit);
  return value & bit_mask;
}

pub fn bit_set(value: u8, bit_index: u8, bit: bool) -> u8 {
  if bit {
    return bit_enable(value, bit_index);
  } else {
    return bit_disable(value, bit_index);
  }
}

pub fn get_empty_screen_buffer() -> Vec<u32> {
  [COLOR_WHITE; SCREEN_HEIGHT * SCREEN_WIDTH].to_vec()
}
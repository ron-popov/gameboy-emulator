// General
pub const PLACEHOLDER_VALUE_U8: u8 = 0x23;
pub const OPCODES_JSON_PATH: &str = "src/opcodes.json";

// Flag masks
pub const FLAG_ZERO_MASK: u8 = 0b10000000;
pub const FLAG_SUB_MASK: u8 = 0b01000000;
pub const FLAG_HALF_CARRY_MASK: u8 = 0b00100000;
pub const FLAG_CARRY_MASK: u8 = 0b00010000;

// External ram size
pub const CARTRIDGE_RAM_SIZE_NONE: u8 = 0x00;
pub const CARTRIDGE_RAM_SIZE_2KB: u8 = 0x01;
pub const CARTRIDGE_RAM_SIZE_8KB: u8 = 0x02;
pub const CARTRIDGE_RAM_SIZE_32KB: u8 = 0x03;
pub const CARTRIDGE_RAM_SIZE_128KB: u8 = 0x04;
pub const CARTRIDGE_RAM_SIZE_64KB: u8 = 0x05;

pub const RAM_SIZE: usize = 32 * 1024;

// Rom size
pub const CARTRIDGE_ROM_SIZE_NO_BANKS: u8 = 0x00;
pub const CARTRIDGE_ROM_SIZE_4_BANKS: u8 = 0x01;
pub const CARTRIDGE_ROM_SIZE_8_BANKS: u8 = 0x02;
pub const CARTRIDGE_ROM_SIZE_16_BANKS: u8 = 0x03;
pub const CARTRIDGE_ROM_SIZE_32_BANKS: u8 = 0x04;
pub const CARTRIDGE_ROM_SIZE_64_BANKS: u8 = 0x05;
pub const CARTRIDGE_ROM_SIZE_128_BANKS: u8 = 0x06;
pub const CARTRIDGE_ROM_SIZE_256_BANKS: u8 = 0x07;
pub const CARTRIDGE_ROM_SIZE_512_BANKS: u8 = 0x08;
pub const CARTRIDGE_ROM_SIZE_72_BANKS: u8 = 0x52;
pub const CARTRIDGE_ROM_SIZE_80_BANKS: u8 = 0x53;
pub const CARTRIDGE_ROM_SIZE_96_BANKS: u8 = 0x54;

// Cartridge Type
pub const CARTRIDGE_TYPE_ROM_ONLY: u8 = 0x00;
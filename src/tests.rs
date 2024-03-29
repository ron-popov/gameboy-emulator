#[cfg(test)]
mod rom_parser_tests {
    use crate::rom_parser::Rom;
    use std::io::Read;
    use std::fs::File;

    #[test]
    fn create_rom_file() {
        let rom_file: File = File::open("roms/bully.gb").expect("Failed opening rom file");

        let rom_content: Vec<u8> = rom_file.bytes().map(|value| {
            value.expect("Failed reading rom file")
        }).collect();

        let _:Rom = Rom::create_from_bytes(rom_content);
    }

    #[test]
    fn validate_rom_values() {
        let rom_file: File = File::open("roms/bully.gb").expect("Failed opening rom file");

        let rom_content: Vec<u8> = rom_file.bytes().map(|value| {
            value.expect("Failed reading rom file")
        }).collect();

        let test_rom:Rom = Rom::create_from_bytes(rom_content);

        assert_eq!(test_rom.title, "BULLYGB");
        assert_eq!(test_rom.manufacturer_code, [0,0,0,0]);
        assert_eq!(test_rom.cgb_flag, 0x80);
    }
}


#[cfg(test)]
mod cpu_tests {
    use crate::cpu::{CPU, get_opcodes};
    use crate::rom_parser::Rom;
    use crate::ram_memory::{RamMemory};
    
    use std::fs::File;
    use std::io::Read;
    use simplelog::*;

    #[test]
    fn test_ram_memory() {
        let rom_file: File = File::open("roms/bully.gb").expect("Failed opening rom file");

        let rom_content: Vec<u8> = rom_file.bytes().map(|value| {
            value.expect("Failed reading rom file")
        }).collect();

        let rom: Rom = Rom::create_from_bytes(rom_content);
        let ram: RamMemory = RamMemory::init_from_rom(&rom);

        assert_eq!(ram.get_addr(0x0147), 0x00); // Cartridge Type
        assert_eq!(ram.get_addr(0x0143), 0x80); // CGB Flag
        assert_eq!(ram.get_addr(0x0134), 0x42); // First char of title (uppercase B)
    }

    #[test]
    fn test_opcodes_json() {
        let opcodes = get_opcodes();
        assert_eq!(opcodes["unprefixed"]["0x00"]["mnemonic"], "NOP");
    }

    #[test]
    fn test_cpu_jp_ld_cp() {
        let test_rom = Rom::create_test_rom();
        let ram_memory = &mut RamMemory::init_from_rom(&test_rom);

        // Jump instruction setup
        //      Jump to 0x0200
        ram_memory.set_addr(0x0100, 0xC3);
        ram_memory.set_addr(0x0101, 0x00);
        ram_memory.set_addr(0x0102, 0x02);

        // Load instruction setup
        //      Load 0x11 to a register
        ram_memory.set_addr(0x0200, 0x3E);
        ram_memory.set_addr(0x0201, 0x11);
        
        // Compare instruction setup at 0x0200
        //      Compare A register with 0x0F
        ram_memory.set_addr(0x0202, 0xFE);
        ram_memory.set_addr(0x0203, 0x0F);

        let mut cpu: CPU = CPU::init_with_ram_ppu(ram_memory);

        // Check Jump
        cpu.execute_instruction();
        assert_eq!(cpu.get_program_counter(), 0x0200);

        // Check Load
        cpu.execute_instruction();
        assert_eq!(cpu.get_register("A".to_string()), 0x11);

        // Check Compare
        cpu.execute_instruction();
        assert_eq!(cpu.get_sub_flag(), true);
        assert_eq!(cpu.get_half_carry_flag(), true);

        // Check Compare
    }

}
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
    use crate::cpu::CPU;
    use crate::rom_parser::Rom;
    use crate::ram_memory::RamMemory;

    use std::fs::File;
    use std::io::Read;

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
        let rom_file: File = File::open("roms/bully.gb").expect("Failed opening rom file");

        let rom_content: Vec<u8> = rom_file.bytes().map(|value| {
            value.expect("Failed reading rom file")
        }).collect();

        let rom: Rom = Rom::create_from_bytes(rom_content);
        let mut ram: RamMemory = RamMemory::init_from_rom(&rom);

        let cpu: CPU = CPU::init_from_rom(&rom, &mut ram);

        assert_eq!(cpu.opcodes["unprefixed"]["0x00"]["mnemonic"], "NOP");
    }
}
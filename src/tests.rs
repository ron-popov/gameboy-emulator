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
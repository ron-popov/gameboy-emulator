use crate::consts::*;

#[readonly::make]
pub struct Rom {
    pub title: String,
    pub manufacturer_code: [u8; 4],
    pub cgb_flag: u8,
    pub new_license_code: [u8; 2],
    pub sgb_flag: u8,
    pub cartridge_type: u8,
    rom_size: u8,
    ram_size: u8,
    pub destination_code: u8,
    pub old_license_code: u8,
    pub mask_rom_version_number: u8,
    pub header_checksum: u8,
    pub global_checksum: u16,
    pub data: Vec<u8>
}

impl Rom {
    pub fn create_from_bytes(rom_content: Vec<u8>) -> Rom {
        let mut title:String = "".to_string();
        for c in &rom_content[0x134..0x0143+1] {
            if *c == 0x00 {
                break;
            }
            title.push(*c as char);
        }

        let new_licnse_code: [u8; 2] = 
            [*rom_content.get(0x144).expect("Invalid rom structure (first new_licnse_code)"),
             *rom_content.get(0x145).expect("Invalid rom structure (second new_licnse_code)")];

        let global_checksum: u16 = 
            (*rom_content.get(0x14E).expect("Invalid rom structure (first global_checksum)") as u16) << 8 |
            (*rom_content.get(0x14F).expect("Invalid rom structure (second global_checksum)") as u16);

        // Validate cartridge type
        let cartridge_type: u8 = *rom_content.get(0x147).expect("Invalid rom structure (cartridge_type)");
        if cartridge_type != CARTRIDGE_TYPE_ROM_ONLY {
            panic!("Unsupported cartridge type (0x{:02X})", cartridge_type);
        }

        // Validate rom size
        let rom_size: u8 = *rom_content.get(0x148).expect("Invalid rom structure (rom_size)");
        if rom_size != CARTRIDGE_ROM_SIZE_NO_BANKS {
            panic!("Unsupported rom size (0x{:02X})", rom_size);
        }

        // Validate rom size
        let ram_size: u8 = *rom_content.get(0x149).expect("Invalid rom structure (ram_size)");
        if ram_size != CARTRIDGE_RAM_SIZE_NONE {
            panic!("Unsupported ram size (0x{:02X})", ram_size);
        }

        Rom {
            title: title.clone(),
            manufacturer_code: rom_content[0x13F..0x142+1].try_into().expect("Invalid rom structure (manufacturer_code)"),
            cgb_flag: *rom_content.get(0x143).expect("Invalid rom structure (cgb_flag)"),
            new_license_code: new_licnse_code,
            sgb_flag: *rom_content.get(0x146).expect("Invalid rom structure (sgb_flag)"),
            cartridge_type: cartridge_type,
            rom_size: rom_size,
            ram_size: ram_size,
            destination_code: *rom_content.get(0x14A).expect("Invalid rom structure (destination_code)"),
            old_license_code: *rom_content.get(0x14B).expect("Invalid rom structure (old_license_code)"),
            mask_rom_version_number: *rom_content.get(0x14C).expect("Invalid rom structure (mask_rom_version_number)"),
            header_checksum: *rom_content.get(0x14D).expect("Invalid rom structure (header_checksum)"),
            global_checksum: global_checksum,
            data: rom_content.clone()
        }
    }

    // pub fn create_test_rom() -> Rom {
    //     Rom {
    //         title: "TEST".to_string(),
    //         manufacturer_code: [0x13, 0x37, 0x23, 0x07],
    //         cgb_flag: 0xFF,
    //         new_license_code: [0xAB, 0xCD],
    //         sgb_flag: 0xFF,
    //         cartridge_type: 0xFF,
    //         rom_size: 0x00,
    //         ram_size: 0x00,
    //         destination_code: 0x80,
    //         old_license_code: 0x00,
    //         mask_rom_version_number: 0x00,
    //         header_checksum: 0x00,
    //         global_checksum: 0x0000,
    //         data: Vec::new()
    //     }
    // }

}
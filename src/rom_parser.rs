// mod consts;
// use consts::*;

use crate::consts::PLACEHOLDER_VALUE_U8;

pub struct Rom {
    title: String,
    manufacturer_code: u64,
    cgb_flag: u8,
    new_license_code: u16,
    sgb_flag: u8,
    cartridge_type: u8,
    rom_size: u8,
    ram_size: u8,
    destination_code: u8,
    old_license_code: u8,
    mask_rom_version_number: u8,
    header_checksum: u8,
    global_checksum: u16,
    data: Vec<u8>
}

impl Rom {
    pub fn create_from_bytes(rom_content: Vec<u8>) -> Rom {
        let mut title:String = "".to_string();
        for c in &rom_content[0x134..0x0143] {
            if *c == 0x00 {
                break;
            }
            title.push(*c as char);
        }

        let new_licnse_code: u16 = 
            (*rom_content.get(0x144).expect("Invalid rom structure (no first new_licnse_code)") as u16) << 8 |
            (*rom_content.get(0x145).expect("Invalid rom structure (no second new_licnse_code)") as u16);

        let global_checksum: u16 = 
            (*rom_content.get(0x14E).expect("Invalid rom structure (no first global_checksum)") as u16) << 8 |
            (*rom_content.get(0x14F).expect("Invalid rom structure (no second global_checksum)") as u16);

        Rom {
            title: title.clone(),
            manufacturer_code: PLACEHOLDER_VALUE_U8 as u64,
            cgb_flag: *rom_content.get(0x143).expect("Invalid rom structure (no cgb_flag)"),
            new_license_code: new_licnse_code,
            sgb_flag: *rom_content.get(0x146).expect("Invalid rom structure (no sgb_flag)"),
            cartridge_type: *rom_content.get(0x147).expect("Invalid rom structure (no cartridge_type)"),
            rom_size: *rom_content.get(0x148).expect("Invalid rom structure (no rom_size)"),
            ram_size: *rom_content.get(0x149).expect("Invalid rom structure (no ram_size)"),
            destination_code: *rom_content.get(0x14A).expect("Invalid rom structure (no destination_code)"),
            old_license_code: *rom_content.get(0x14B).expect("Invalid rom structure (no old_license_code)"),
            mask_rom_version_number: *rom_content.get(0x14C).expect("Invalid rom structure (no mask_rom_version_number)"),
            header_checksum: *rom_content.get(0x14D).expect("Invalid rom structure (no header_checksum)"),
            global_checksum: global_checksum,
            data: rom_content.clone()
        }
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }
}
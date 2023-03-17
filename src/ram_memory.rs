use crate::rom_parser::Rom;
use crate::consts::*;

pub struct RamMemory {
    memory: Vec<u8>
}

impl RamMemory {
    pub fn init_from_rom(rom: &Rom) -> RamMemory {
        let mut memory: Vec<u8> = Vec::new();

        for b in &rom.data {
            memory.push(*b);
        }

        while memory.len() < CARTRIDGE_ROM_SIZE_DEFAULT {
            memory.push(0x00);
        }

        RamMemory { 
            memory
        }
    }

    pub fn get_addr(&self, addr: u16) -> u8 {
        if addr as usize >= self.memory.len() {
            panic!("Requested invalid memory addr ({:04X})", addr);
        }

        *self.memory.get(addr as usize).unwrap()
    }

    pub fn set_addr(&mut self, addr: u16, value: u8) {
        if addr as usize >= self.memory.len() {
            panic!("Requested invalid memory addr ({:04X})", addr);
        }

        self.memory[addr as usize] = value;
    }
}
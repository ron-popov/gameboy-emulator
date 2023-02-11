use crate::consts::*;
use crate::ram_memory::RamMemory;
use crate::rom_parser::Rom;

pub struct CPU<'cpu> {
    ram_memory: &'cpu mut RamMemory,
    rom: &'cpu Rom
}

impl<'cpu_impl> CPU<'_> {
    pub fn init_from_rom(rom: &'cpu_impl Rom, ram_memory: &'cpu_impl mut RamMemory) -> CPU<'cpu_impl> {
        CPU {
            ram_memory: ram_memory,
            rom: rom
        }
    }

    fn get_addr(&self, addr: u16) -> u8 {
        self.ram_memory.get_addr(addr)
    }

    fn set_addr(&mut self, addr: u16, value: u8) {
        self.ram_memory.set_addr(addr, value);
    }
}
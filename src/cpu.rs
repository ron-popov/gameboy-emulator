use crate::consts::*;
use crate::ram_memory::RamMemory;
use crate::rom_parser::Rom;
use crate::opcodes::OPCODES_JSON;

use serde_json::{Result, Value};

#[readonly::make]
pub struct CPU<'cpu> {
    ram_memory: &'cpu mut RamMemory,
    rom: &'cpu Rom,
    a_reg: u8,
    b_reg: u8,
    c_reg: u8,
    d_reg: u8,
    e_reg: u8,
    f_reg: u8,
    h_reg: u8,
    l_reg: u8,
    sp_reg: u16,
    pc_reg: u16,
    pub opcodes: Value
}

impl<'cpu_impl> CPU<'_> {
    pub fn init_from_rom(rom: &'cpu_impl Rom, ram_memory: &'cpu_impl mut RamMemory) -> CPU<'cpu_impl> {
        let opcodes: Value = serde_json::from_str(OPCODES_JSON).expect("Failed parsing opcodes json data");

        CPU {
            ram_memory: ram_memory,
            rom: rom,
            a_reg: 0,
            b_reg: 0,
            c_reg: 0,
            d_reg: 0,
            e_reg: 0,
            f_reg: 0,
            h_reg: 0,
            l_reg: 0,
            pc_reg: 0x0100,
            sp_reg: 0xFFFE,
            opcodes: opcodes
        }
    }

    pub fn execute_instruction(&mut self) {
        let mut opcode = self.get_addr(self.pc_reg);
        let mut opcode_data: Value = Value::Null;

        
        if opcode == 0xCB {
            opcode = self.get_addr(self.pc_reg + 1);

            trace!("Executing instruction 0x{:02X} with 0xCB prefix", opcode);
            opcode_data = self.opcodes["cbprefixed"][format!("0x{:02X}", opcode)].clone();
        } else {

            trace!("Executing instruction 0x{:02X}", opcode);
            opcode_data = self.opcodes["unprefixed"][format!("0x{:02X}", opcode)].clone();
        }

        assert_ne!(opcode_data, Value::Null);
        trace!("{}", opcode_data);
        panic!("bla");
    }

    fn get_addr(&self, addr: u16) -> u8 {
        self.ram_memory.get_addr(addr)
    }

    fn set_addr(&mut self, addr: u16, value: u8) {
        self.ram_memory.set_addr(addr, value);
    }
}
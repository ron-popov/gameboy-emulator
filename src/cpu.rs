use crate::consts::*;
use crate::ram_memory::RamMemory;
use crate::rom_parser::Rom;
use crate::opcodes::OPCODES_JSON;
use crate::param::{Param, ParamType, ParamValue};

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

        trace!("{}", opcode_data);

        if opcode_data == Value::Null {
            panic!("Opcode data for instruction 0x{:02X} is null", opcode);
        }

        if opcode_data["mnemonic"] == Value::Null {
            panic!("Opcode 0x{:02X} doesn't have a name", opcode);
        } else if !opcode_data["mnemonic"].is_string() {
            panic!("Opcode 0x{:02X} name is not a string (WTF)", opcode);
        }
        
        let opcode_name: &str = opcode_data["mnemonic"].as_str().unwrap();
        let params: Vec<Param> = self.get_params(&opcode_data);

        match opcode_name {
            "NOP" => {
                // Nothing to do
            },
            "DI" => {
                //TODO: Disable instrupts
            },
            "JP" => {
                if params.len() == 1 {

                }
            }
            _ => {
                panic!("Unknown opcode name ({})", opcode_data["mnemonic"]);
            }
        }

        self.pc_reg += opcode_data["bytes"].as_u64().unwrap() as u16;

    }

    fn get_addr(&self, addr: u16) -> u8 {
        self.ram_memory.get_addr(addr)
    }

    fn set_addr(&mut self, addr: u16, value: u8) {
        self.ram_memory.set_addr(addr, value);
    }

    fn get_params(&self, opcode_data: &Value) -> Vec<Param> {
        if !opcode_data["operands"].is_array() {
            panic!("Operands value is not array");
        }

        let mut return_value = Vec::<Param>::new();

        for operand in opcode_data["operands"].as_array().unwrap() {
            let is_immediate: bool = operand["immediate"] != Value::Null && operand["immediate"].as_bool().unwrap();
            let mut bytes_count: usize = 0;
            if operand["bytes"] != Value::Null {
                if !operand["bytes"].is_u64() {
                    panic!("Invalid operand bytes type");
                }

                bytes_count = operand["bytes"].as_u64().unwrap() as usize;
            }
            
            return_value.push(
                Param::new(operand["name"].as_str().unwrap().to_string(), is_immediate, bytes_count)
            );
        }

        return_value
    }

    fn is_bool_param_true(&self, param: Param) {

    }
}
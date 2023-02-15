use crate::consts::*;
use crate::ram_memory::RamMemory;
use crate::rom_parser::Rom;
use crate::opcodes::OPCODES_JSON;
use crate::param::{Param, ParamValue};

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
        let opcode_data: Value;
        let mut should_inc_pc = true;
        
        if opcode == 0xCB {
            opcode = self.get_addr(self.pc_reg + 1);

            debug!("Executing instruction 0x{:02X} from addr {:04X} with 0xCB prefix", opcode, self.pc_reg);
            opcode_data = self.opcodes["cbprefixed"][format!("0x{:02X}", opcode)].clone();
        } else {
            debug!("Executing instruction 0x{:02X} from addr {:04X}", opcode, self.pc_reg);
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

        trace!("Params for this opcode are {:?}", params);

        match opcode_name {
            "NOP" => {
                // Nothing to do \:
            },
            "DI" => {
                info!("TODO: Disable instrupts");
            },
            "JP" => {
                if params.len() == 1 {
                    let target_addr: u16 = params.get(0).unwrap().get_double();
                    trace!("Jumping to addr 0x{:04X}", target_addr);
                    
                    should_inc_pc = false;
                    self.pc_reg = target_addr;
                }
            }
            _ => {
                panic!("Unknown opcode name ({})", opcode_data["mnemonic"]);
            }
        }

        if should_inc_pc {
            self.pc_reg += opcode_data["bytes"].as_u64().unwrap() as u16;
        }

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
            
            let mut param = Param::new(operand["name"].as_str().unwrap().to_string(), is_immediate, bytes_count);
            
            let value: ParamValue = match param.get_name().as_str() {
                "a16" => {
                    let mut value: u16= self.get_addr(self.pc_reg + 1) as u16;
                    value += (self.get_addr(self.pc_reg + 2) as u16) << 8;
                    ParamValue::Double(value)
                },
                "d16" => {
                    let mut value: u16= self.get_addr(self.pc_reg + 1) as u16;
                    value += (self.get_addr(self.pc_reg + 2) as u16) << 8;
                    ParamValue::Double(value)
                },
                _ => panic!("Unknown parameter type")
            };

            param.set_param_value(value);

            return_value.push(
                param
            );
        }

        return_value
    }

    fn is_bool_param_true(&self, param: Param) {

    }
}
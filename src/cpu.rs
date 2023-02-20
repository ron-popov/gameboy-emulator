use crate::consts::*;
use crate::ram_memory::RamMemory;
use crate::rom_parser::Rom;
use crate::opcodes::OPCODES_JSON;
use crate::param::{Param, MemValue};

use serde_json::Value;

pub fn get_opcodes() -> Value {
    serde_json::from_str(OPCODES_JSON).expect("Failed parsing opcodes json data")
}

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
    opcodes: Value
}

impl<'cpu_impl> CPU<'_> {
    pub fn init_from_rom(rom: &'cpu_impl Rom, ram_memory: &'cpu_impl mut RamMemory) -> CPU<'cpu_impl> {
        let opcodes = get_opcodes();

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
        let mut set_zero_flag: MemValue = MemValue::Null;
        let mut set_carry_flag: MemValue = MemValue::Null;
        let mut set_sub_flag: MemValue = MemValue::Null;
        let mut set_half_carry_flag: MemValue = MemValue::Null;
        
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
            "NOP" => { // NOTHING
                // Nothing to do \:
            },
            "DI" => { // DISABLE INTERRUPTS
                info!("TODO: Disable instrupts");
            },
            "JP" => { // JUMP
                if params.len() == 1 {
                    let target_addr: u16 = params.get(0).unwrap().get_double();
                    trace!("Jumping to addr 0x{:04X}", target_addr);
                    
                    should_inc_pc = false;
                    self.pc_reg = target_addr;
                } else {
                    panic!("Invalid param count for JP")
                }
            },
            "CP" => { // COMPARE
                if params.len() != 1 {
                    panic!("Invalid params count for CP");
                }

                set_sub_flag = MemValue::Bool(true);

                let param = params.get(0).unwrap().get_byte();
                match self.a_reg.checked_sub(param) {
                    Some(sub_result) => {
                        // Valid sub result
                        set_carry_flag = MemValue::Bool(false);
                        
                        if sub_result == 0 {
                            set_zero_flag = MemValue::Bool(true);
                        } else {
                            set_zero_flag = MemValue::Bool(false);
                        }
                    }, 
                    None => {
                        //Underflow happened
                        set_carry_flag = MemValue::Bool(true);
                    }
                }

                set_half_carry_flag = MemValue::Bool((((self.a_reg & 0xf).wrapping_sub(param & 0xf)) & 0x10) != 0);
            },
            "JR" => { // RELATIVE JUMP, SOMETIMES CONDITIONAL
                if params.len() == 1 {
                    let jump_addr_param = params.get(0).unwrap();
                    self.pc_reg = self.pc_reg.wrapping_add_signed(jump_addr_param.get_signed_byte() as i16);
                    should_inc_pc = false;
                } else if params.len() == 2 {
                    let condition_param = params.get(0).unwrap();

                    if condition_param.get_bool() {
                        let jump_addr_param = params.get(1).unwrap();
                        self.pc_reg = self.pc_reg.wrapping_add_signed(jump_addr_param.get_signed_byte() as i16);
                        should_inc_pc = false;
                    }

                } else {
                    panic!("Invalid param count for JR")
                }
            },
            "LD" => { // LOAD
                match params.len() {
                    2 => {
                        let target_param = params.get(0).unwrap();
                        let read_param = params.get(1).unwrap();
    
                        let mut write_value: MemValue = MemValue::Null;

                        match read_param.get_value() {
                            MemValue::Register(reg_name) => {
                                match reg_name.len() {
                                    1 => {
                                        let reg_value = self.get_register(reg_name);
                                        if read_param.is_immediate() {
                                            write_value = MemValue::Byte(reg_value);
                                        } else {
                                            unimplemented!("Load from single register not immediate value")
                                        }
                                    },
                                    2 => {
                                        let mut chars = reg_name.chars();
                                        let reg_value = (self.get_register(chars.next().unwrap().to_string()) as u16) << 8 + 
                                            self.get_register(chars.next().unwrap().to_string()) as u16;
                                        if read_param.is_immediate() {
                                            write_value = MemValue::Double(reg_value);
                                        } else {
                                            write_value = MemValue::Byte(self.get_addr(reg_value));
                                        }
                                        
                                    },
                                    _ => panic!("Invalid register name length")
                                }
                            },
                            MemValue::Byte(_) => write_value = read_param.get_value(),
                            _ => panic!("Tried running LD from unknown param type ({:?})", read_param)
                        }

                        match target_param.get_value() {
                            MemValue::Register(reg_name) => {
                                match reg_name.len() {
                                    1 => {
                                        match write_value {
                                            MemValue::Byte(value) => {
                                                self.set_register(reg_name, value)
                                            },
                                            _ => panic!("Invalid type to load to a single register")
                                        }
                                    },
                                    2 => {
                                        unimplemented!("Load to a double register")
                                    },
                                    _ => panic!("Invalid register name length")
                                }
                            },
                            _ => panic!("Tried writing to unknown param type ({:?})", target_param)
                        }
                    },
                    _ => {
                        unimplemented!("Load with {} params", params.len())
                    }
                }
            }
            _ => {
                unimplemented!("Opcode name ({})", opcode_data["mnemonic"]);
            }

        }

        if should_inc_pc {
            self.pc_reg += opcode_data["bytes"].as_u64().unwrap() as u16;
        }

        match set_zero_flag {
            MemValue::Bool(value) => {
                self.set_zero_flag(value);
            },
            _ => ()
        }

        match set_sub_flag {
            MemValue::Bool(value) => {
                self.set_sub_flag(value);
            },
            _ => ()
        }

        match set_half_carry_flag {
            MemValue::Bool(value) => {
                self.set_half_carry_flag(value);
            },
            _ => ()
        }

        match set_carry_flag {
            MemValue::Bool(value) => {
                self.set_carry_flag(value);
            },
            _ => ()
        }

    }

    // Memory stuff
    fn get_addr(&self, addr: u16) -> u8 {
        self.ram_memory.get_addr(addr)
    }

    pub fn get_register(&self, reg: String) -> u8 {
        match reg.to_lowercase().as_str() {
            "a" => self.a_reg,
            "b" => self.b_reg,
            "c" => self.c_reg,
            "d" => self.d_reg,
            "e" => self.e_reg,
            "f" => self.f_reg,
            "h" => self.h_reg,
            "l" => self.l_reg,
            _ => panic!("Requested value of unknown register ({})", reg)
        }
    }

    fn set_register(&mut self, reg: String, value: u8) {
        match reg.to_lowercase().as_str() {
            "a" => self.a_reg = value,
            "b" => self.b_reg = value,
            "c" => self.c_reg = value,
            "d" => self.d_reg = value,
            "e" => self.e_reg = value,
            "f" => self.f_reg = value,
            "h" => self.h_reg = value,
            "l" => self.l_reg = value,
            _ => panic!("Requested writing to unknown register ({})", reg)
        }
    }

    pub fn is_register(&self, reg: String) -> bool {
        ["a", "b", "c", "d", "e", "f", "h", "l"].contains(&reg.to_lowercase().as_str())
    }

    fn set_addr(&mut self, addr: u16, value: u8) {
        self.ram_memory.set_addr(addr, value);
    }

    pub fn get_program_counter(&self) -> u16 {
        self.pc_reg
    }

    // Params stuff
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
            
            let param_name = operand["name"].as_str().unwrap().to_string();
            let mut param = Param::new(param_name.clone(), is_immediate, bytes_count);
            
            let value: MemValue = match param.get_name().as_str() {
                "a16" => {
                    let mut value: u16= self.get_addr(self.pc_reg + 1) as u16;
                    value += (self.get_addr(self.pc_reg + 2) as u16) << 8;
                    MemValue::Double(value)
                },
                "d16" => {
                    let mut value: u16= self.get_addr(self.pc_reg + 1) as u16;
                    value += (self.get_addr(self.pc_reg + 2) as u16) << 8;
                    MemValue::Double(value)
                },
                "d8" => {
                    MemValue::Byte(self.get_addr(self.pc_reg + 1))
                },
                "NZ" => {
                    MemValue::Bool(!self.get_zero_flag())
                },
                "r8" => {
                    MemValue::SignedByte(self.get_addr(self.pc_reg + 1) as i8)
                },
                "a8" => {
                    MemValue::Byte(self.get_addr(self.pc_reg + 1))
                },
                _ => {
                    let chars = param_name.clone();
                    for c in chars.chars() {
                        if !self.is_register(c.to_string()) {
                            panic!("Unknown param name, should be register but isn't ({})", c);
                        }
                    }

                    MemValue::Register(param_name)
                }                
            };

            param.set_param_value(value);

            return_value.push(
                param
            );
        }
        
        return_value
    }

    // Flags stuff
    fn set_flag(&mut self, value: bool, mask: u8) {
        if value {
            self.f_reg = self.f_reg | mask
        } else {
            self.f_reg = self.f_reg & (mask ^ 0b11111111)
        }
    }

    fn get_flag(&self, mask: u8) -> bool {
        self.f_reg & mask != 0
    }

    fn set_zero_flag(&mut self, value: bool) {
        self.set_flag(value, FLAG_ZERO_MASK)
    }

    pub fn get_zero_flag(&self) -> bool {
        self.get_flag(FLAG_ZERO_MASK)
    }

    fn set_sub_flag(&mut self, value: bool) {
        self.set_flag(value, FLAG_SUB_MASK)
    }

    pub fn get_sub_flag(&self) -> bool {
        self.get_flag(FLAG_SUB_MASK)
    }

    fn set_half_carry_flag(&mut self, value: bool) {
        self.set_flag(value, FLAG_HALF_CARRY_MASK)
    }

    pub fn get_half_carry_flag(&self) -> bool {
        self.get_flag(FLAG_HALF_CARRY_MASK)
    }

    fn set_carry_flag(&mut self, value: bool) {
        self.set_flag(value, FLAG_CARRY_MASK)
    }

    pub fn get_carry_flag(&self) -> bool {
        self.get_flag(FLAG_CARRY_MASK)
    }

}
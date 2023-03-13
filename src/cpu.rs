use crate::consts::*;
use crate::ppu::PPU;
use crate::ram_memory::RamMemory;
use crate::opcodes::OPCODES_JSON;
use crate::param::{Param, MemValue};

use serde_json::Value;
use std::rc::Rc;
use std::cell::RefCell;

pub fn get_opcodes() -> Value {
    serde_json::from_str(OPCODES_JSON).expect("Failed parsing opcodes json data")
}

#[readonly::make]
pub struct CPU {
    ram_memory_ref: Rc<RefCell<RamMemory>>,
    ppu_ref: Rc<RefCell<PPU>>,
    internal_ram_memory: Vec<u8>,
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

impl CPU {
    pub fn init_with_ram_ppu(ram_memory_ref: Rc<RefCell<RamMemory>>, ppu_ref: Rc<RefCell<PPU>>) -> CPU {
        let opcodes = get_opcodes();

        CPU {
            ram_memory_ref: ram_memory_ref,
            ppu_ref: ppu_ref,
            internal_ram_memory: vec![0; INTERNAL_RAM_MEMORY_SIZE as usize],
            a_reg: 0,
            b_reg: 0,
            c_reg: 0,
            d_reg: 0,
            e_reg: 0,
            f_reg: 0,
            h_reg: 0,
            l_reg: 0,
            pc_reg: 0x0000,
            sp_reg: 0xFFFE,
            opcodes: opcodes
        }
    }

    pub fn execute_instruction(&mut self) {
        let mut opcode = self.get_addr(self.pc_reg);
        let opcode_data: Value;
        let mut should_inc_pc = true;
        let mut set_zero_flag: Option<bool> = Option::None;
        let mut set_carry_flag: Option<bool> = Option::None;
        let mut set_sub_flag: Option<bool> = Option::None;
        let mut set_half_carry_flag: Option<bool> = Option::None;
        
        if opcode == 0xCB {
            opcode = self.get_addr(self.pc_reg + 1);

            opcode_data = self.opcodes["cbprefixed"][format!("0x{:02X}", opcode)].clone();
        } else {
            opcode_data = self.opcodes["unprefixed"][format!("0x{:02X}", opcode)].clone();
        }
            
        // Just some checks
        assert_ne!(opcode_data["mnemonic"], Value::Null, "Opcode 0x{:02X} doesn't have a name", opcode);
        assert!(opcode_data["mnemonic"].is_string(), "Opcode 0x{:02X} name is not a string (WTF)", opcode);

        if opcode_data == Value::Null {
            panic!("Opcode data for instruction 0x{:02X} is null", opcode);
        }
        
        // Parsing
        let opcode_name: &str = opcode_data["mnemonic"].as_str().unwrap();
        trace!("{}", opcode_data);
        let params: Vec<Param> = self.get_params(&opcode_data);

        // Prints
        let mut param_data: String = "".to_string();
        for param in &params {
            if !param.is_immediate() {
                param_data += format!("({}) ", param.get_name()).as_str();
            } else {
                param_data += format!("{} ", param.get_name()).as_str();
            }
        }
        debug!("0x{:04X} -> {} {}", self.pc_reg, opcode_name, param_data);

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
                assert_eq!(params.len(), 1, "Invalid params count for CP");

                set_sub_flag = Some(true);

                let param = params.get(0).unwrap().get_byte();
                match self.a_reg.checked_sub(param) {
                    Some(sub_result) => {
                        // Valid sub result
                        set_carry_flag = Some(false);
                        
                        if sub_result == 0 {
                            set_zero_flag = Some(true);
                        } else {
                            set_zero_flag = Some(false);
                        }
                    }, 
                    None => {
                        //Underflow happened
                        set_carry_flag = Some(true);
                    }
                }

                set_half_carry_flag = Some((((self.a_reg & 0xf).wrapping_sub(param & 0xf)) & 0x10) != 0);
            },
            "JR" => { // RELATIVE JUMP, SOMETIMES CONDITIONAL
                // match params.len() {
                //     1 => {
                //         let jump_addr_param = params.get(0).unwrap();
                //         self.pc_reg = self.pc_reg.wrapping_add_signed(jump_addr_param.get_signed_byte() as i16);
                //         should_inc_pc = false;
                //     },
                //     2 => {

                //     },
                //     _ => panic!("Unknown number of params to JR ({})", params.len())
                // };
                if params.len() == 1 {
                    let jump_addr_param = params.get(0).unwrap();
                    self.pc_reg = self.pc_reg.wrapping_add_signed(jump_addr_param.get_signed_byte() as i16);
                    should_inc_pc = false;
                } else if params.len() == 2 {
                    let condition_param = params.get(0).unwrap();

                    if self.get_condition_value(condition_param.get_name()) {
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
    
                        let write_value: MemValue;

                        match read_param.get_value() {
                            MemValue::Name(reg_name) => {
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
                                        let reg_name = chars.next().unwrap().to_string() + &chars.next().unwrap().to_string();
                                        let reg_value = self.get_double_register(&reg_name);
                                        if read_param.is_immediate() {
                                            write_value = MemValue::Double(reg_value);
                                        } else {
                                            write_value = MemValue::Byte(self.get_addr(reg_value));
                                        }

                                        // Implement LDD and LDI
                                        if read_param.is_decrement() {
                                            self.set_double_register(
                                                &reg_name,
                                                reg_value - 1);
                                        } else if read_param.is_increment() {
                                            if read_param.is_decrement() {
                                            self.set_double_register(
                                                &reg_name,
                                                reg_value + 1);
                                            }
                                        }
                                        
                                    },
                                    _ => panic!("Invalid register name length")
                                }
                            },
                            MemValue::Byte(_) => write_value = read_param.get_value(),
                            MemValue::Double(_) => write_value = read_param.get_value(),
                            _ => panic!("Tried running LD from unknown param type ({:?})", read_param)
                        }

                        match target_param.get_value() {
                            MemValue::Name(reg_name) => {
                                match reg_name.len() {
                                    1 => {
                                        // TODO: Refactor this shit, maybe with a macro?
                                        match write_value {
                                            MemValue::Byte(value) => {
                                                self.set_register(reg_name, value)
                                            },
                                            _ => panic!("Invalid type to load to a single register")
                                        }
                                    },
                                    2 => {
                                        match write_value {
                                            MemValue::Double(value) => {
                                                self.set_double_register(&reg_name, value)
                                            },
                                            MemValue::Byte(value) => {
                                                assert_eq!(target_param.is_immediate(), false);
                                                let target_addr: u16 = self.get_double_register(&reg_name);

                                                self.set_addr(target_addr, value);
                                            }
                                            _ => panic!("Invalid type to load to a double register ({:?})", write_value)
                                        }
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
            },
            "LDH" => { // LOAD
                assert_eq!(params.len(), 2, "Invalid param count to LDH");

                let to_param = params.get(0).unwrap();
                let from_param = params.get(1).unwrap();

                let from_value: u8;
                match from_param.get_value() {
                    MemValue::Byte(value) => {
                        assert_eq!(from_param.is_immediate(), false, "LDH from immediate byte value");
                        let from_addr: u16 = 0xFF00 + value as u16;
                        from_value = self.get_addr(from_addr);
                    },
                    MemValue::Name(name) => {
                        assert_eq!(from_param.is_immediate(), true, "LDH from not immediate register");
                        from_value = self.get_register(name);
                    },
                    _ => panic!("LDH from unknown type ({:?})", from_param.get_value())
                }
            },
            "XOR" => { // XOR
                assert_eq!(params.len(), 1, "Invalid param count to XOR");

                let from_param = params.get(0).unwrap();

                let xor_value: u8;
                match from_param.get_value() {
                    MemValue::Byte(value) => {
                        xor_value = value;
                    },
                    MemValue::Name(name) => {
                        xor_value = self.get_register(name);
                    },
                    MemValue::Double(addr) => {
                        assert_eq!(from_param.is_immediate(), true, "Tried running XOR with Double immediate value???");
                        xor_value = self.get_addr(addr)
                    }
                    _ => panic!("Unknown type to run XOR with ({:?})", from_param.get_value())
                };

                self.a_reg = self.a_reg ^ xor_value;

                set_zero_flag = Some(self.a_reg == 0);
                set_carry_flag = Some(false);
                set_sub_flag = Some(false);
                set_half_carry_flag = Some(false);
            },
            "BIT" => {
                assert_eq!(params.len(), 2, "Invalid param count to BIT");

                let bit_index = params.get(0).unwrap().get_name().parse::<u8>().unwrap();
                let reg_name: String = params.get(1).unwrap().get_name();
                let reg_value;

                set_sub_flag = Some(false);
                set_half_carry_flag = Some(true);


                match reg_name.len() {
                    1 => {
                        reg_value = self.get_register(reg_name) ;
                    },
                    2 => {
                        assert_eq!(params.get(1).unwrap().is_immediate(), false);
                        reg_value = self.get_addr(self.get_double_register(&reg_name));
                    },
                    _ => {
                        panic!("Tried running BIT on invalid register")
                    }
                };

                set_zero_flag = Some(((reg_value >> bit_index) % 2) == 0);
            },
            "RST" => {
                assert_eq!(params.len(), 1, "Invalid param count to RST");

                self.stack_push_double(self.pc_reg);

                let new_addr_str = params.get(0).unwrap().get_name().replace("H", "");
                let new_addr_parse_result = u16::from_str_radix(new_addr_str.as_str(),16);
                match new_addr_parse_result {
                    Ok(new_addr) => {
                        self.pc_reg = new_addr;
                    },
                    Err(e) => {
                        panic!("Failed getting RST addr to jump ({})", e);
                    }
                }

                should_inc_pc = false;
            },
            _ => {
                unimplemented!("Opcode name ({})", opcode_data["mnemonic"]);
            }

        }

        if should_inc_pc {
            self.pc_reg += opcode_data["bytes"].as_u64().unwrap() as u16;
        }

        self.verify_flag(opcode_data["flags"]["Z"].as_str().unwrap(), set_zero_flag, "Zero");
        match set_zero_flag {
            Some(value) => {
                self.set_zero_flag(value);
            },
            None => ()
        }

        self.verify_flag(opcode_data["flags"]["N"].as_str().unwrap(), set_sub_flag, "Sub");
        match set_sub_flag {
            Some(value) => {
                self.set_sub_flag(value);
            },
            None => ()
        }

        self.verify_flag(opcode_data["flags"]["H"].as_str().unwrap(), set_half_carry_flag, "Half Carry");
        match set_half_carry_flag {
            Some(value) => {
                self.set_half_carry_flag(value);
            },
            None => ()
        }

        self.verify_flag(opcode_data["flags"]["C"].as_str().unwrap(), set_carry_flag, "Carry");
        match set_carry_flag {
            Some(value) => {
                self.set_carry_flag(value);
            },
            None => ()
        }

    }
    
    // Register stuff
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

    fn get_double_register(&self, reg: &String) -> u16 {
        if reg.to_uppercase() == "SP" {
            return self.pc_reg;
        }

        let first_reg: String = reg[0..1].to_string();
        let second_reg: String = reg[1..2].to_string();

        let mut value: u16 = self.get_register(first_reg) as u16;
        value += (self.get_register(second_reg) as u16) << 8;

        return value;
    }

    fn set_double_register(&mut self, reg: &String, value: u16) {
        assert_eq!(reg.len(), 2, "Double register name is not of len 2");

        match reg.to_uppercase().as_str() {
            "SP" => self.sp_reg = value,
            _ => {
                let first_reg: String = reg[0..1].to_string();
                let second_reg: String = reg[1..2].to_string();

                let msb: u8 = (value >> 8) as u8;
                let lsb: u8 = (value & 0xFF) as u8;

                self.set_register(first_reg, msb);
                self.set_register(second_reg, lsb);
            }
        }
    }

    // Stack stuff
    fn stack_push(&mut self, value: u8) {
         self.sp_reg -= 1;
         self.set_addr(self.sp_reg, value);
    }

    fn stack_pop(&mut self) -> u8 {
        let ret_value = self.get_addr(self.sp_reg);
        self.sp_reg += 1;
        return ret_value;
    }

    fn stack_push_double(&mut self, value: u16) {
        let lsb = (value & 0xff) as u8;
        let msb = (value >> 8) as u8;
        
        self.stack_push(msb);
        self.stack_push(lsb);
    }

    fn stack_pop_double(&mut self) -> u16 {
        let lsb = self.stack_pop() as u16;
        let msb = self.stack_pop() as u16;
    
        return lsb + (msb << 8);
    }

    // Memory stuff
    fn get_addr(&self, addr: u16) -> u8 {
        if addr < RAM_SIZE as u16 { // 0x0000 -> 0x8000
            return self.ram_memory_ref.borrow_mut().get_addr(addr);
        } else if (addr >= RAM_SIZE as u16 && addr < RAM_IO_PORTS_RANGE_START) { // 0x8000 -> 0xFF00
            unimplemented!("Requested unimplemented memory addr (0x{:04X})", addr);
        } else if (addr >= RAM_IO_PORTS_RANGE_START && addr < RAM_EMPTY_RANGE_START) {
            return self.ppu_ref.borrow().get_addr(addr);
        } else if (addr >= RAM_EMPTY_RANGE_START && addr < RAM_INTERNAL_RANGE_START) {
            panic!("Requested addr at a memory addr that should not be used (0x{:04X})", addr);
        } else if (addr >= RAM_INTERNAL_RANGE_START) {
            let internal_memory_addr = addr - RAM_INTERNAL_RANGE_START;
            return self.internal_ram_memory[internal_memory_addr as usize];
        } else { // DAFUK
            // return 0xFF;
            panic!("Dafuk? (0x{:04X})", addr);
        }
    }

    fn set_addr(&mut self, addr: u16, value: u8) {
        if addr < RAM_SIZE as u16 { // 0x0000 -> 0x8000
            self.ram_memory_ref.borrow_mut().set_addr(addr, value);
        } else if (addr >= RAM_SIZE as u16 && addr < RAM_IO_PORTS_RANGE_START) { // 0x8000 -> 0xFF00
            unimplemented!("Requested write to unimplemented memory addr (0x{:04X})", addr);
        } else if (addr >= RAM_IO_PORTS_RANGE_START && addr < RAM_EMPTY_RANGE_START) {
            return self.ppu_ref.borrow_mut().set_addr(addr, value);
        } else if (addr >= RAM_EMPTY_RANGE_START && addr < RAM_INTERNAL_RANGE_START) {
            panic!("Requested write to addr at a memory addr that should not be used (0x{:04X})", addr);
        } else if (addr >= RAM_INTERNAL_RANGE_START) {
            let internal_memory_addr = addr - RAM_INTERNAL_RANGE_START;
            self.internal_ram_memory[internal_memory_addr as usize] = value;
        } else { // DAFUK
            // return 0xFF;
            panic!("Dafuk? (0x{:04X})", addr);
        }
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
            let mut param = Param::new(operand.clone());
            
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
                "r8" => {
                    MemValue::SignedByte(self.get_addr(self.pc_reg + 1) as i8)
                },
                "a8" => {
                    MemValue::Byte(self.get_addr(self.pc_reg + 1))
                },
                _ => {
                    // let chars = param.get_name().clone();
                    // for c in chars.chars() {
                    //     if !c.is_alphabetic() {
                    //         panic!("Invalid char ({})", c)
                    //     }
                    // }

                    MemValue::Name(param.get_name())
                }                
            };

            param.set_param_value(value);

            return_value.push(
                param
            );
        }
        
        return_value
    }

    fn get_condition_value(&self, cond: String) -> bool {
        match cond.to_uppercase().as_str() {
            "Z" => self.get_zero_flag(),
            "NZ" => !self.get_zero_flag(),
            "C" => self.get_carry_flag(),
            "NC" => !self.get_carry_flag(),
            _ => panic!("Unknown condition ({})", cond)
        }
    }

    // Flags stuff
    fn set_flag(&mut self, value: bool, mask: u8) {
        if value {
            self.f_reg = self.f_reg | mask
        } else {
            self.f_reg = self.f_reg & (mask ^ 0b11111111)
        }
    }

    fn verify_flag(&self, doc: &str, value: Option<bool>, name: &str) {
        trace!("Checking Flag {}, with doc \"{}\" and value \"{:?}\"", name, doc, value);
        if doc == "-" { assert!(value == Option::None, "{} Flag should be empty", name);}
        if doc != "-" { assert!(value != Option::None, "{} Flag cannot be empty", name); }
        if doc == "1" { assert!(value == Option::Some(true), "{} Flag has to be true", name); }
        if doc == "0" { assert!(value == Option::Some(false), "{} Flag has to be false", name); }
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


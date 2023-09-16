use crate::consts::*;
use crate::ppu::PPU;
use crate::ram_memory::RamMemory;
use crate::opcodes::OPCODES_JSON;
use crate::param::{Param, MemValue};

use serde_json::Value;
use core::panic;
use std::rc::Rc;
use std::cell::RefCell;

pub fn get_opcodes() -> Value {
    serde_json::from_str(OPCODES_JSON).expect("Failed parsing opcodes json data")
}

#[readonly::make]
pub struct CPU {
    ram_memory_ref: Rc<RefCell<RamMemory>>,
    ppu_ref: Rc<RefCell<PPU>>,
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
    instruction_counter: usize,
    opcodes: Value
}

impl CPU {
    pub fn init_with_ram_ppu(ram_memory_ref: Rc<RefCell<RamMemory>>, ppu_ref: Rc<RefCell<PPU>>, boot_rom_enabled: bool) -> CPU {
        let opcodes = get_opcodes();

        let initial_pc: u16;
        if boot_rom_enabled {
            initial_pc = 0x0000;
        } else {
            initial_pc = 0x0100;
        }
        
        CPU {
            ram_memory_ref: ram_memory_ref,
            ppu_ref: ppu_ref,
            a_reg: 0,
            b_reg: 0,
            c_reg: 0,
            d_reg: 0,
            e_reg: 0,
            f_reg: 0,
            h_reg: 0,
            l_reg: 0,
            // pc_reg: 0x000C,
            pc_reg: initial_pc,
            sp_reg: 0xFFFE,
            instruction_counter: 0,
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
        let is_opcode_cbprefixed: bool;

        self.instruction_counter += 1;

        // Blargg test rom stuff
        if self.pc_reg == 0xC005 {
            info!("Entering infinite loop");
            loop {}
        }

        if opcode == 0xCB {
            is_opcode_cbprefixed = true;
            opcode = self.get_addr(self.pc_reg + 1);

            opcode_data = self.opcodes["cbprefixed"][format!("0x{:02X}", opcode)].clone();
        } else {
            is_opcode_cbprefixed = false;
            opcode_data = self.opcodes["unprefixed"][format!("0x{:02X}", opcode)].clone();
        }
            
        // Just some checks
        assert_ne!(opcode_data["mnemonic"], Value::Null, "Opcode 0x{:02X} doesn't have a name", opcode);
        assert!(opcode_data["mnemonic"].is_string(), "Opcode 0x{:02X} name is not a string (WTF)", opcode);

        if opcode_data == Value::Null {
            panic!("Opcode data for instruction 0x{:02X} is null", opcode);
        }
        
        // Parsing
        let params: Vec<Param> = self.get_params(&opcode_data);
        
        // Debug Prints
        let opcode_name: &str = opcode_data["mnemonic"].as_str().unwrap();
        let mut param_data: String = "".to_string();
        for param in &params {
            param_data += &param.get_printable();
            param_data += ", ";
        }
        trace!("");
        trace!("");
        debug!("CPU State, A:0x{:02X}, B:0x{:02X}, , C:0x{:02X}", self.a_reg, self.b_reg, self.c_reg);
        debug!("0x{:04X} -> {} {}", self.pc_reg, opcode_name, param_data);
        trace!("Instruction #{}", self.instruction_counter);

        trace!("OPCODE 0x{:02X}, IS_CB_PREFIXED: {}", opcode, is_opcode_cbprefixed);

        // Trace Prints
        for s in Self::pretty_opcode_data(&opcode_data) {
            trace!("{}", s);
        }

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

                    // Blargg test rom stuff
                    if target_addr == 0xC3C3 {
                        self.dump_memory();
                        panic!("JP: Blargg rom test fail");
                    }

                    trace!("Jumping to addr 0x{:04X}", target_addr);
                    
                    should_inc_pc = false;
                    self.pc_reg = target_addr;
                } else {
                    panic!("JP: Invalid param count")
                }
            },
            "CP" => { // COMPARE
                assert_eq!(params.len(), 1, "CP: Invalid params count");

                set_sub_flag = Some(true);

                

                
                let param = params.get(0).unwrap();
                let value = match param.get_value() {
                    MemValue::Byte(b) => b,
                    MemValue::Name(reg) => {
                        if param.is_immediate() {
                            self.get_register(&reg)
                        } else {
                            let addr = self.get_double_register(&reg);
                            self.get_addr(addr)
                        }
                    },
                    _ => panic!("CP: Invalid source MemValue type")
                };

                let (sub_result, did_underflow) = u8::overflowing_sub(self.a_reg, value);
                set_carry_flag = Some(did_underflow);
                set_zero_flag = Some(sub_result == 0);

                self.a_reg = sub_result;

                set_half_carry_flag = Some((((self.a_reg & 0xf).wrapping_sub(value & 0xf)) & 0x10) != 0);
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
                    // should_inc_pc = false;
                } else if params.len() == 2 {
                    let condition_param = params.get(0).unwrap();

                    if self.get_condition_value(condition_param.get_name()) {
                        let jump_addr_param = params.get(1).unwrap();
                        self.pc_reg = self.pc_reg.wrapping_add_signed(jump_addr_param.get_signed_byte() as i16);
                        // should_inc_pc = false;
                    }

                } else {
                    panic!("JR: Invalid param count")
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
                                        let reg_value = self.get_register(&reg_name);
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

                                    },
                                    _ => panic!("Invalid register name length")
                                }
                            },
                            MemValue::Byte(_) => write_value = read_param.get_value(),
                            MemValue::Double(double_value) => {
                                if read_param.is_immediate() {
                                    write_value = read_param.get_value();
                                } else {
                                    write_value = MemValue::Byte(self.get_addr(double_value));
                                }
                            }
                            _ => panic!("Tried running LD from unknown param type ({:?})", read_param)
                        }

                        match target_param.get_value() {
                            MemValue::Name(reg_name) => {
                                match reg_name.len() {
                                    1 => {
                                        // TODO: Refactor this shit, maybe with a macro?
                                        match write_value {
                                            MemValue::Byte(value) => {
                                                self.set_register(&reg_name, value)
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
                                    }
                                    _ => panic!("Invalid register name length")
                                }

                                // For LDI and LDD (or LD (HL-) and LD (HL+))
                                if target_param.is_decrement() {
                                    self.set_double_register(
                                        &reg_name,
                                        self.get_double_register(&reg_name) - 1);
                                } else if target_param.is_increment() {
                                    self.set_double_register(
                                        &reg_name,
                                        self.get_double_register(&reg_name) + 1);
                                }

                                if read_param.is_decrement() {
                                    self.set_double_register(
                                        &read_param.get_name(),
                                        self.get_double_register(&read_param.get_name()) - 1);
                                } else if read_param.is_increment() {
                                    self.set_double_register(
                                        &read_param.get_name(),
                                        self.get_double_register(&read_param.get_name()) + 1);
                                }

                                trace!("HL: {:?}", self.get_double_register(&"HL".to_string()));
                                trace!("LD: From {:?} to {:?}", read_param, target_param);

                            },
                            MemValue::Double(addr) => {
                                match write_value {
                                    MemValue::Byte(value) => {
                                        self.set_addr(addr, value);
                                    },
                                    MemValue::Double(value) => {
                                        let msb = Self::msb(value);
                                        let lsb = Self::lsb(value);
                                        
                                        self.set_addr(addr, lsb);
                                        self.set_addr(addr+1, msb);
                                    }
                                    _ => panic!("Tried writing non byte value to a memory addr ({:?})", write_value)
                                }
                            }
                            _ => panic!("Tried writing to unknown param type ({:?})", target_param)
                        }
                    },
                    _ => {
                        unimplemented!("Load with {} params", params.len())
                    }
                }
            },
            "LDH" => { // LOAD
                // unimplemented!("LDH: I think it is broken?");
                assert_eq!(params.len(), 2, "Invalid param count to LDH");

                let to_param = params.get(0).unwrap();
                let from_param = params.get(1).unwrap();


                let from_value: u8;
                match from_param.get_value() {
                    MemValue::Byte(value) => {
                        assert_eq!(from_param.is_immediate(), false, "LDH: from immediate byte value");
                        let from_addr: u16 = 0xFF00 + value as u16;
                        from_value = self.get_addr(from_addr);
                    },
                    MemValue::Name(reg_name) => {
                        assert_eq!(from_param.is_immediate(), true, "LDH: from not immediate register");
                        from_value = self.get_register(&reg_name);
                    },
                    _ => panic!("LDH: from unknown type ({:?})", from_param.get_value())
                }

                match to_param.get_value() {
                    MemValue::Byte(value) => {
                        assert_eq!(to_param.is_immediate(), false, "LDH: to immediate byte value");

                        let to_addr: u16 = 0xFF00 + value as u16;
                        self.set_addr(to_addr, from_value);
                    },
                    MemValue::Name(reg_name) => {
                        assert_eq!(to_param.is_immediate(), true, "LDH: to not immediate register");

                        self.set_register(&reg_name, from_value);
                    },
                    _ => panic!("LDH: to unknown type ({:?})", to_param.get_value())
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
                        let reg_name = from_param.get_name();
                        trace!("XOR: From reg \"{}\"", reg_name);
                        if from_param.is_immediate() {
                            xor_value = self.get_register(&name);
                        } else {
                            xor_value = self.get_addr(self.get_double_register(&reg_name));
                        }
                    },
                    MemValue::Double(addr) => {
                        assert_eq!(from_param.is_immediate(), true, "Tried running XOR with Double immediate value???");
                        xor_value = self.get_addr(addr)
                    }
                    _ => panic!("XOR: Unknown type ({:?})", from_param.get_value())
                };

                self.a_reg = self.a_reg ^ xor_value;

                set_zero_flag = Some(self.a_reg == 0);
                set_carry_flag = Some(false);
                set_sub_flag = Some(false);
                set_half_carry_flag = Some(false);
            },
            "BIT" => { // Check if certain bit in byte is set
                assert_eq!(params.len(), 2, "BIT: Invalid param count");

                let bit_index = params.get(0).unwrap().get_name().parse::<u8>().unwrap();
                let reg_name: String = params.get(1).unwrap().get_name();
                let reg_value;

                set_sub_flag = Some(false);
                set_half_carry_flag = Some(true);


                match reg_name.len() {
                    1 => {
                        reg_value = self.get_register(&reg_name) ;
                    },
                    2 => {
                        assert_eq!(params.get(1).unwrap().is_immediate(), false);
                        reg_value = self.get_addr(self.get_double_register(&reg_name));
                    },
                    _ => {
                        panic!("BIT: Tried running operation on invalid register")
                    }
                };

                set_zero_flag = Some(((reg_value >> bit_index) % 2) == 0);
            },
            "RST" => { // Push PC to stack and jump to one of hardcoded values
                assert_eq!(params.len(), 1, "RST: Invalid param count");

                self.stack_push_double(self.pc_reg);

                let new_addr_str = params.get(0).unwrap().get_name().replace("H", "");
                let new_addr_parse_result = u16::from_str_radix(new_addr_str.as_str(),16);
                match new_addr_parse_result {
                    Ok(new_addr) => {
                        self.pc_reg = new_addr;
                    },
                    Err(e) => {
                        panic!("RST: Failed getting addr to jump ({})", e);
                    }
                }

                should_inc_pc = false;
            },
            "INC" => { // Increment value
                assert_eq!(params.len(), 1, "INC: Invalid param count");
                let param = params.get(0).unwrap();
                let reg_name = param.get_name();

                set_sub_flag = Some(false);

                if param.is_immediate() {
                    match reg_name.len() {
                        1 => {
                            let value = self.get_register(&reg_name);
                            let new_value = u8::overflowing_add(value, 1).0;

                            set_half_carry_flag = Some(((u8::overflowing_add(value & 0x0f, 1).0) & 0x10) == 0x10);
                            set_zero_flag = Some(new_value == 0);

                            self.set_register(&reg_name, new_value);
                        },
                        2 => {
                            let value = self.get_double_register(&reg_name);
                            let new_value = u16::overflowing_add(value, 1).0;

                            set_sub_flag = Option::None;

                            self.set_double_register(&reg_name, new_value);
                        },
                        _ => panic!("INC: Invalid reg_name")
                    }
                } else {
                    match reg_name.len() {
                        2 => {
                            let value = self.get_double_register(&reg_name);
                            let new_value = u16::overflowing_add(value, 1).0;

                            set_half_carry_flag = Some(((u16::overflowing_add(value & 0x0f, 1).0) & 0x10) == 0x10);
                            set_zero_flag = Some(new_value == 0);

                            self.set_double_register(&reg_name, new_value);
                        },
                        _ => panic!("INC: Invalid reg_name")
                    }
                }
            },
            "DEC" => { // Decrement value
                assert_eq!(params.len(), 1, "DEC: Invalid param count");
                let param = params.get(0).unwrap();
                let reg_name = param.get_name();

                set_sub_flag = Some(true);

                if param.is_immediate() {
                    match reg_name.len() {
                        1 => {
                            let value = self.get_register(&reg_name);
                            let new_value = u8::overflowing_sub(value, 1).0;

                            set_half_carry_flag = Some(((u8::overflowing_sub(value & 0x0f, 1).0) & 0x10) == 0x10);
                            set_zero_flag = Some(new_value == 0);

                            self.set_register(&reg_name, new_value);
                        },
                        2 => {
                            let value = self.get_double_register(&reg_name);
                            let new_value = u16::overflowing_sub(value, 1).0;

                            set_sub_flag = Option::None;

                            self.set_double_register(&reg_name, new_value);
                        },
                        _ => panic!("DEC: Invalid reg_name")
                    }
                } else {
                    match reg_name.len() {
                        2 => {
                            let value = self.get_double_register(&reg_name);
                            let new_value = u16::overflowing_sub(value, 1).0;

                            set_half_carry_flag = Some(((u16::overflowing_sub(value & 0x0f, 1).0) & 0x10) == 0x10);
                            set_zero_flag = Some(new_value == 0);

                            self.set_double_register(&reg_name, new_value);
                        },
                        _ => panic!("DEC: Invalid reg_name")
                    }
                }
            },
            "CALL" => { // JUMP to addr and push current pc to stack
                
                match params.len() {
                    1 => {
                        let target_addr = params.get(0).unwrap().get_double();
                        self.stack_push_double(
                            self.pc_reg + opcode_data["bytes"].as_u64().unwrap() as u16);

                        self.pc_reg = target_addr;
                        should_inc_pc = false;
                    },
                    2 => {
                        let condition = params.get(0).unwrap().get_name();
                        let target_addr = params.get(1).unwrap().get_double();

                        let should_jump: bool = match condition.as_str() {
                            "NZ"    => !self.get_zero_flag(),
                            "Z"     => self.get_zero_flag(),
                            "NC"    => !self.get_carry_flag(),
                            "C"     => self.get_carry_flag(),
                            _ => panic!("Jump: Unknown condition : {}", condition)
                        };

                        if should_jump {
                            self.stack_push_double(
                                self.pc_reg + opcode_data["bytes"].as_u64().unwrap() as u16);
    
                            self.pc_reg = target_addr;
                            should_inc_pc = false;
                        }
                    },
                    _ => panic!("CALL: Invalid param count")
                }
            },
            "PUSH" => { // Push value to stack
                assert_eq!(params.len(), 1);

                let reg_name = params.get(0).unwrap().get_name();
                self.stack_push_double(self.get_double_register(&reg_name));
            },
            "POP" => { // Pop value from the stack to the corresponding register
                assert_eq!(params.len(), 1);

                let reg_name = params.get(0).unwrap().get_name();
                let popped_value = self.stack_pop_double();
                self.set_double_register(&reg_name, popped_value);

                
            },
            "RL" => { // Rotate left through the carry flag
                assert_eq!(params.len(), 1);
                let param = params.get(0).unwrap();
                let reg_name = param.get_name();

                set_half_carry_flag = Some(false);
                set_sub_flag = Some(false);

                if param.is_immediate() { // Register
                    let old_value = self.get_register(&reg_name);
                    set_carry_flag = Some((old_value & 0b10000000) == 0b10000000);

                    let mut new_value = old_value << 1;
                    if self.get_carry_flag() {
                        new_value += 1;
                    }

                    set_zero_flag = Some(new_value == 0);
                    self.set_register(&reg_name, new_value);
                } else { // (HL)
                    let addr = self.get_double_register(&reg_name);
                    let old_value = self.get_addr(addr);
                    set_carry_flag = Some((old_value & 0b10000000) == 0b10000000);

                    let mut new_value = old_value << 1;
                    if self.get_carry_flag() {
                        new_value += 1;
                    }

                    set_zero_flag = Some(new_value == 0);
                    self.set_addr(addr, new_value);
                }
            },
            "RLA" => { // Rotate left A register through the carry flag
                assert_eq!(params.len(), 0);

                let old_value = self.a_reg;

                set_carry_flag = Some((old_value & 0b10000000) == 0b10000000);
                set_half_carry_flag = Some(false);
                set_sub_flag = Some(false);
                set_zero_flag = Some(false);

                let mut new_value = old_value << 1;
                if self.get_carry_flag() {
                    new_value += 1;
                }

                self.a_reg = new_value;
            },
            "RET" => { // Return, maybe conditional
                match params.len() {
                    0 => { // Just return
                        let addr = self.stack_pop_double();
                        self.pc_reg = addr;
                        should_inc_pc = false;
                    },
                    1 => { // Conditional return
                        unimplemented!("RET: Conditional return")
                    },
                    _ => panic!("RET: Inavlid param count")
                }
            },
            "SUB" => {
                assert_eq!(params.len(), 1, "SUB: Too much params");
                let param = params.get(0).unwrap();

                let value: u8 = match param.get_value() {
                    MemValue::Name(reg_name) => {
                        if param.is_immediate() {
                            self.get_register(&reg_name)
                        } else {
                            let addr = self.get_double_register(&reg_name);
                            self.get_addr(addr)
                        }
                    },
                    MemValue::Byte(param_value) => {
                        param_value
                    },
                    _ => panic!("SUB: Invalid param type")
                };

                let (sub_result, did_underflow) = u8::overflowing_sub(self.a_reg, value);

                self.a_reg = sub_result;

                set_sub_flag = Some(true);
                set_zero_flag = Some(sub_result == 0);
                set_carry_flag = Some(did_underflow);
                set_half_carry_flag = Some((((self.a_reg & 0xf).wrapping_sub(value & 0xf)) & 0x10) != 0);
            },
            "OR" => {
                assert_eq!(params.len(), 1, "OR: Too much params");
                let param = params.get(0).unwrap();

                let value: u8 = match param.get_value() {
                    MemValue::Name(reg_name) => {
                        if param.is_immediate() {
                            self.get_register(&reg_name)
                        } else {
                            let addr = self.get_double_register(&reg_name);
                            self.get_addr(addr)
                        }
                    },
                    MemValue::Byte(param_value) => {
                        param_value
                    },
                    _ => panic!("SUB: Invalid param type")
                };

                let a_reg_value: u8 = self.get_register(&"A".to_string());
                let or_result = value | a_reg_value;

                self.set_register(&"A".to_string(), or_result);

                set_sub_flag = Some(false);
                set_zero_flag = Some(or_result == 0);
                set_carry_flag = Some(false);
                set_half_carry_flag = Some(false);

            },
            "AND" => {
                assert_eq!(params.len(), 1, "AND: Too much params");
                let param = params.get(0).unwrap();

                let value: u8 = match param.get_value() {
                    MemValue::Name(reg_name) => {
                        if param.is_immediate() {
                            self.get_register(&reg_name)
                        } else {
                            let addr = self.get_double_register(&reg_name);
                            self.get_addr(addr)
                        }
                    },
                    MemValue::Byte(param_value) => {
                        param_value
                    },
                    _ => panic!("SUB: Invalid param type")
                };

                let a_reg_value: u8 = self.get_register(&"A".to_string());
                let and_result = value & a_reg_value;

                self.set_register(&"A".to_string(), and_result);

                set_sub_flag = Some(false);
                set_zero_flag = Some(and_result == 0);
                set_carry_flag = Some(false);
                set_half_carry_flag = Some(true);
            },
            "ADD" => {
                assert_eq!(params.len(), 2, "ADD: Invalid amount of params");

                // Parse source value
                let from_param = params.get(1).unwrap();
                let from_value: u8 = match from_param.get_value() {
                    MemValue::Name(reg_name) => {
                        if from_param.is_immediate() {
                            self.get_register(&reg_name)
                        } else {
                            let addr = self.get_double_register(&reg_name);
                            self.get_addr(addr)
                        }
                    },
                    MemValue::Byte(param_value) => {
                        param_value
                    },
                    _ => panic!("ADD: Invalid param type")
                };

                // Parse destination value
                let dest_reg = params.get(0).unwrap().get_name();
                match dest_reg.len() {
                    1 => { // A register
                        assert_eq!(dest_reg, "A", "ADD: Attemp to add to a register of length 1 which isn't A!");

                        // trace!("ADD: Before {}", self.a_reg);
                        let (add_result, did_overflow) = u8::overflowing_add(self.a_reg, from_value);
                        self.a_reg = add_result;
                        // trace!("ADD: After {}", self.a_reg);
                        // trace!("ADD: Should be {}", add_result);

                        set_sub_flag = Some(false);
                        set_zero_flag = Some(add_result == 0);
                        set_carry_flag = Some(did_overflow);
                        set_half_carry_flag = Some((((self.a_reg & 0xf).wrapping_add(from_value & 0xf)) & 0x10) != 0);
                    },
                    2 => { // SP or HL
                        // let (add_result, did_underflow) = u16::overflowing_add(self.get_double_register(&dest_reg), from_value);

                        // set_sub_flag = Some(true);
                        // set_zero_flag = Some(add_result == 0);
                        // set_carry_flag = Some(did_underflow);
                        // set_half_carry_flag = Some((((self.a_reg & 0xf).wrapping_add(from_value & 0xf)) & 0x100) != 0);

                        unimplemented!("ADD: 16-bit arithmetics");
                    },
                    _ => panic!("Attemp to add to a register of unrecognized len")
                }
            },
            "SRL" => {
                assert_eq!(params.len(), 1, "SRL: Invalid amount of params");
                let param = params.get(0).unwrap();

                let reg_name = param.get_name();
                trace!("SRL: Opearting on \"{}\"", reg_name);

                match reg_name.len() {
                    2 => { // (HL)
                        let addr = self.get_double_register(&reg_name);
                        let value = self.get_addr(addr);

                        set_carry_flag = Some(Self::lsb(value.into()) == 1);
                        let new_value = value >> 1;
                        set_zero_flag = Some(new_value == 0);
                        self.set_addr(addr, new_value);
                    },
                    1 => { // All other registers
                        let value = self.get_register(&reg_name);

                        set_carry_flag = Some(Self::lsb(value.into()) == 1);
                        let new_value = value >> 1;
                        set_zero_flag = Some(new_value == 0);
                        self.set_register(&reg_name, new_value);
                    },
                    _ => panic!("SRL: Invalid register name")
                }

                set_sub_flag = Some(false);
                set_half_carry_flag = Some(false);
            },
            "RR" => { // Rotate right through the carry flag
                assert_eq!(params.len(), 1);
                let param = params.get(0).unwrap();
                let reg_name = param.get_name();

                set_half_carry_flag = Some(false);
                set_sub_flag = Some(false);

                if param.is_immediate() { // Register
                    let old_value = self.get_register(&reg_name);
                    set_carry_flag = Some(Self::lsb(old_value.into()) == 1);

                    let mut new_value = old_value >> 1;
                    if self.get_carry_flag() {
                        new_value += 0b1000000;
                    }

                    set_zero_flag = Some(new_value == 0);
                    self.set_register(&reg_name, new_value);
                } else { // (HL)
                    let addr = self.get_double_register(&reg_name);
                    let old_value = self.get_addr(addr);
                    set_carry_flag = Some(Self::lsb(old_value.into()) == 1);

                    let mut new_value = old_value >> 1;
                    if self.get_carry_flag() {
                        new_value += 0b1000000;
                    }

                    set_zero_flag = Some(new_value == 0);
                    self.set_addr(addr, new_value);
                }
            },
            "RRA" => {
                assert_eq!(params.len(), 0);
                let reg_name = &"A".to_string();

                set_half_carry_flag = Some(false);
                set_sub_flag = Some(false);

                let old_value = self.get_register(&reg_name);
                set_carry_flag = Some(Self::lsb(old_value.into()) == 1);

                let mut new_value = old_value >> 1;
                if self.get_carry_flag() {
                    new_value += 0b1000000;
                }

                set_zero_flag = Some(new_value == 0);
                self.set_register(&reg_name, new_value);
            },
            _ => {
                self.dump_memory();
                unimplemented!("Opcode name ({})", opcode_data["mnemonic"]);
            }
        }

        if should_inc_pc {
            trace!("Increasing PC");
            self.pc_reg += opcode_data["bytes"].as_u64().unwrap() as u16;
        } else {
            trace!("NOT Increasing PC");
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
    
    pub fn dump_memory(&self) {
        let mut i: usize = 0x00;
        let ram = self.ram_memory_ref.borrow_mut();
        while i < 0xFFFF {
            let mut data_string: String = "".to_string();
            for j in 0..8 {
                data_string += &format!("0x{:02X} ", ram.get_addr(i as u16 + j));
                // temp_vec.push(ram.get_addr(i as u16 + j));
            }
            trace!("0x{:04X} -> {}", i, data_string);
            i += 8;
        }
    }

    // Register stuff
    pub fn get_register(&self, reg: &String) -> u8 {
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

    fn set_register(&mut self, reg: &String, value: u8) {
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

        assert_eq!(reg.len(), 2, "CPU: Tried getting double register from single reg");

        let first_reg: String = reg[0..1].to_string();
        let second_reg: String = reg[1..2].to_string();

        let mut value: u16 = (self.get_register(&first_reg) as u16) << 8; // msb
        value += self.get_register(&second_reg) as u16; // lsb

        return value;
    }

    fn set_double_register(&mut self, reg: &String, value: u16) {
        assert_eq!(reg.len(), 2, "CPU: Tried setting double register from single reg");

        match reg.to_uppercase().as_str() {
            "SP" => self.sp_reg = value,
            _ => {
                let first_reg: String = reg[0..1].to_string();
                let second_reg: String = reg[1..2].to_string();

                let msb: u8 = Self::msb(value);
                let lsb: u8 = Self::lsb(value);

                self.set_register(&first_reg, msb);
                self.set_register(&second_reg, lsb);
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
        let lsb = Self::lsb(value);
        let msb = Self::msb(value);
        
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
        if addr < CARTRIDGE_ROM_SIZE_DEFAULT as u16 
        { // 0x0000 -> 0x8000
            return self.ram_memory_ref.borrow_mut().get_addr(addr);
        } 
        else if addr >= CARTRIDGE_ROM_SIZE_DEFAULT as u16 && addr < RAM_ECHO_RANGE_START 
        { // 0x8000 -> 0xE000
            return self.ram_memory_ref.borrow_mut().get_addr(addr);
        }
        else if addr >= RAM_ECHO_RANGE_START && addr < RAM_SPRITE_ATTRIBUTE_TABLE_RANGE_START
        { // 0xE000 -> 0xFE00
            return self.ram_memory_ref.borrow_mut().get_addr(addr - 0x2000);
        }
        else if addr >= RAM_SPRITE_ATTRIBUTE_TABLE_RANGE_START && addr < RAM_IO_PORTS_RANGE_START
        { // 0xFE00 -> 0xFF00
            return self.ram_memory_ref.borrow_mut().get_addr(addr);
        }
        else if addr >= RAM_IO_PORTS_RANGE_START && addr < RAM_EMPTY_RANGE_START 
        { // 0xFF00 -> 0xFF4C
            return self.ppu_ref.borrow().get_addr(addr);
        } 
        else if addr >= RAM_EMPTY_RANGE_START && addr < RAM_INTERNAL_RANGE_START
        { // 0xFF4C -> 0xFF80
            warn!("Requested addr at a memory addr that should not be used (0x{:04X})", addr);
            return self.ram_memory_ref.borrow_mut().get_addr(addr);
        } 
        else if addr >= RAM_INTERNAL_RANGE_START
        { // 0xFF80 -> END
            return self.ram_memory_ref.borrow_mut().get_addr(addr);
        } 
        else 
        { // DAFUK
            panic!("Dafuk? (0x{:04X})", addr);
        }
    }

    fn set_addr(&mut self, addr: u16, value: u8) {
        if addr < CARTRIDGE_ROM_SIZE_DEFAULT as u16 
        { // 0x0000 -> 0x8000
            self.ram_memory_ref.borrow_mut().set_addr(addr, value);
        } 
        else if addr >= CARTRIDGE_ROM_SIZE_DEFAULT as u16 && addr < RAM_ECHO_RANGE_START 
        { // 0x8000 -> 0xE000
            debug!("VRAM: Write {} to addr 0x{:04X}", value, addr);
            self.ram_memory_ref.borrow_mut().set_addr(addr, value);
        }
        else if addr >= RAM_ECHO_RANGE_START && addr < RAM_SPRITE_ATTRIBUTE_TABLE_RANGE_START
        { // 0xE000 -> 0xFE00
            self.ram_memory_ref.borrow_mut().set_addr(addr - 0x2000, value);
        }
        else if addr >= RAM_SPRITE_ATTRIBUTE_TABLE_RANGE_START && addr < RAM_IO_PORTS_RANGE_START
        { // 0xFE00 -> 0xFF00
            self.ram_memory_ref.borrow_mut().set_addr(addr, value);
        }
        else if addr >= RAM_IO_PORTS_RANGE_START && addr < RAM_EMPTY_RANGE_START
        { // 0xFF00 -> 0xFF4C
            return self.ppu_ref.borrow_mut().set_addr(addr, value);
        } 
        else if addr >= RAM_EMPTY_RANGE_START && addr < RAM_INTERNAL_RANGE_START
        { // 0xFF4C -> 0xFF80
            warn!("Requested write to addr at a memory addr that should not be used (0x{:04X})", addr);
            self.ram_memory_ref.borrow_mut().set_addr(addr, value);
        } 
        else if addr >= RAM_INTERNAL_RANGE_START
        { // 0xFF80 -> END
            self.ram_memory_ref.borrow_mut().set_addr(addr, value);
        } 
        else 
        { // DAFUK
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
        // trace!("Checking Flag {}, with doc \"{}\" and value \"{:?}\"", name, doc, value);
        if doc == "-" { assert!(value == Option::None, "{} Flag should be empty", name);}
        if doc == "1" { assert!(value == Option::Some(true), "{} Flag has to be true", name); }
        if doc == "0" { assert!(value == Option::Some(false), "{} Flag has to be false", name); }
        if doc != "-" { assert!(value != Option::None, "{} Flag cannot be empty", name); }
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

    fn msb(value: u16) -> u8 {
        (value >> 8) as u8
    }

    fn lsb(value: u16) -> u8 {
        (value & 0xff) as u8
    }

    fn pretty_opcode_data(opcode_data: &Value) -> Vec<String> {
        let mut opcode_string: Vec<String> = vec![];

        // Opcode name and length
        opcode_string.push(format!("{}: {} bytes", 
            opcode_data["mnemonic"].as_str().unwrap(),
            opcode_data["bytes"].as_u64().unwrap()
        ));

        // Flag
        for flag in vec!["C", "H", "N", "Z"] {
            let flag_name = match flag {
                "C" => "Carry",
                "H" => "Half-Carry",
                "N" => "Negative",
                "Z" => "Zero",
                _ => panic!("Unknown flag ({})", flag)
            };

            let flag_status = opcode_data["flags"][flag].as_str().unwrap();
            let flag_status_pretty = match flag_status {
                "-" => "Unaffected",
                "0" => "Turned Off",
                "1" => "Turned On",
                _ => "By Value"
            };

            opcode_string.push(
                format!("    {} -> {}",
                flag_name.to_string(),
                flag_status_pretty
            ))            
        }

        // Params
        // opcode_string.push("-- PARAMS --".to_string());
        for (i, param) in opcode_data["operands"].as_array().unwrap().iter().enumerate() {
            let mut param_string = format!("    Param #{} -> {}",
                i,
                param["name"].as_str().unwrap());

            let mut options: Vec<String> = vec![];

            let immediate = param["immediate"].as_bool();
            if immediate.is_some() && immediate.unwrap() {
                options.push("immediate".to_string());
            }

            let decrement = param["decrement"].as_bool();
            if decrement.is_some() && decrement.unwrap() {
                options.push("decrement".to_string());
            }

            let increment = param["increment"].as_bool();
            if increment.is_some() && increment.unwrap() {
                options.push("increment".to_string());
            }

            if options.len() != 0 {
                param_string += " ";
                param_string += &options.join(", ");
            }

            opcode_string.push(param_string);
        }

        return opcode_string;
    }

}


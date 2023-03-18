use serde_json::Value;

#[derive(Debug, Clone)]
pub enum MemValue {
    Null,
    Byte(u8),
    Double(u16),
    SignedByte(i8),
    Name(String)
}

#[derive(Debug)]
pub struct Param {
    name: String,
    value: MemValue,
    json_value: Value
}

impl Param {
    pub fn new(json_value: Value) -> Param {
        Param {
            name: json_value["name"].as_str().unwrap().to_string(),
            value: MemValue::Null,
            json_value: json_value
        }
    }

    // pub fn get_bytes(&self) -> u8 {
    //     let mut bytes_count: u8 = 0;
    //     if self.json_value["bytes"] != Value::Null {
    //         if !self.json_value["bytes"].is_u64() {
    //             panic!("Invalid operand bytes type");
    //         }

    //         bytes_count = self.json_value["bytes"].as_u64().unwrap() as u8;
    //     }

    //     return bytes_count;

    // }
    
    pub fn is_decrement(&self) -> bool {
        match self.json_value["decrement"].as_bool() {
            Some(value) => {
                value
            },
            None => {
                false
            }
        }
    }

    pub fn is_increment(&self) -> bool {
        match self.json_value["increment"].as_bool() {
            Some(value) => {
                value
            },
            None => {
                false
            }
        }
    }

    pub fn set_param_value(&mut self, value: MemValue) {
        self.value = value;
    } 

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_value(&self) -> MemValue {
        self.value.clone()
    }

    pub fn get_printable(&self) -> String {
        let mut param_text = match &self.value {
            MemValue::Byte(value) => {
                format!("0x{:02X}", value)
            },
            MemValue::SignedByte(value) => {
                //TODO : Make this really signed (currently show 0xFB and not negative something)
                format!("0x{:02X}", value)
            },
            MemValue::Double(value) => {
                format!("0x{:04X}", value)
            },
            MemValue::Name(value) => {
                format!("{}", value)
            },
            _ => panic!("Failed getting printable string for this param")
        };

        // let mut param_data: String;
        if !self.is_immediate() {
            if self.is_decrement() {
                param_text = param_text + "-";
            } else if self.is_increment() {
                param_text = param_text + "+";
            }
            param_text = format!("({})", param_text);
        }

        return param_text;        
    }

    pub fn is_immediate(&self) -> bool {
        self.json_value["immediate"] != Value::Null && self.json_value["immediate"].as_bool().unwrap()
    }

    pub fn get_signed_byte(&self) -> i8 {
        match self.value {
            MemValue::SignedByte(value) => value,
            _ => panic!("Tries getting param value as signed byte, but it is {:?}", self.value)
        }
    }

    pub fn get_double(&self) -> u16 {
        match self.value {
            MemValue::Double(value) => value,
            _ => panic!("Tries getting param value as double, but it is {:?}", self.value)
        }
    }

    pub fn get_byte(&self) -> u8 {
        match self.value {
            MemValue::Byte(value) => value,
            _ => panic!("Tries getting param value as byte, but it is {:?}", self.value)
        }
    }

}
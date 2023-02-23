use serde_json::Value;

#[derive(Debug, Clone)]
pub enum MemValue {
    Null,
    Bool(bool),
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

    pub fn get_bytes(&self) -> u8 {
        let mut bytes_count: u8 = 0;
        if self.json_value["bytes"] != Value::Null {
            if !self.json_value["bytes"].is_u64() {
                panic!("Invalid operand bytes type");
            }

            bytes_count = self.json_value["bytes"].as_u64().unwrap() as u8;
        }

        return bytes_count;

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

    pub fn get_bool(&self) -> bool {
        match self.value {
            MemValue::Bool(value) => value,
            _ => panic!("Tries getting param value as bool, but it is {:?}", self.value)
        }
    }

}
#[derive(Debug)]
pub enum MemValue {
    Null,
    Bool(bool),
    Byte(u8),
    Double(u16)
}

#[derive(Debug)]
pub struct Param {
    name: String,
    immediate: bool,
    bytes: usize,
    value: MemValue
}

impl Param {
    pub fn new(name: String, immediate: bool, bytes: usize) -> Param {
        Param {
            name: name,
            immediate: immediate,
            bytes: bytes,
            value: MemValue::Null
        }
    }

    pub fn set_bool(&mut self, value: bool) {
        self.value = MemValue::Bool(value);
    }

    pub fn set_byte(&mut self, value: u8) {
        self.value = MemValue::Byte(value);
    }

    pub fn set_double(&mut self, value: u16) {
        self.value = MemValue::Double(value);
    }

    pub fn set_null(&mut self) {
        self.value = MemValue::Null;
    }

    pub fn set_param_value(&mut self, value: MemValue) {
        self.value = value;
    } 

    pub fn get_name(&self) -> String {
        self.name.clone()
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
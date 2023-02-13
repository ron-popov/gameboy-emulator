pub enum ParamType {
    Null,
    Bool,
    Byte,
    Double
}

pub enum ParamValue {
    Null,
    Bool(bool),
    Byte(u8),
    Double(u16)
}

pub struct Param {
    name: String,
    immediate: bool,
    bytes: usize,
    value: ParamValue
}

impl Param {
    pub fn new(name: String, immediate: bool, bytes: usize) -> Param {
        Param {
            name: name,
            immediate: immediate,
            bytes: bytes
        }
    }
}
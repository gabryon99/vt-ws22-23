#[derive(Debug)]
pub enum OpCode {
    HALT = 0x00,
    CLRA = 0x01,
    INC3A = 0x02,
    DECA = 0x03,
    SETL = 0x04,
    BACK7 = 0x05,
    SPILL = 0x06
}

impl TryFrom<u8> for OpCode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 => Ok(unsafe { std::mem::transmute(value) }),
            _ => Err("invalid OpCode value"),
        }
    }
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        unsafe {
            std::mem::transmute(self)
        }
    }
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

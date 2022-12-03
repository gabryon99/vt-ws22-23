#[derive(Debug)]
pub enum OpCode {
    HALT = 0x00,
    CLRA = 0x01,
    INC3A = 0x02,
    DECA = 0x03,
    SETL = 0x04,
    BACK7 = 0x05,
}

impl TryFrom<u8> for OpCode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 => Ok(unsafe { std::mem::transmute(value) }),
            _ => Err("invalid OpCode value"),
        }
    }
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub enum OpCode {
    HALT = 0x00,
    CLRA = 0x01,
    INC3A = 0x02,
    DECA = 0x03,
    SETL = 0x04,
    BACK7 = 0x05,
    UKN = 0xff,
}

impl From<u8> for OpCode {
    fn from(b: u8) -> Self {
        match b {
            0x00 => Self::HALT,
            0x01 => Self::CLRA,
            0x02 => Self::INC3A,
            0x03 => Self::DECA,
            0x04 => Self::SETL,
            0x05 => Self::BACK7,
            _ => Self::UKN,
        }
    }
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_opcode = match self {
            OpCode::HALT => "HALT",
            OpCode::CLRA => "CLRA",
            OpCode::INC3A => "INC3A",
            OpCode::DECA => "DECA",
            OpCode::SETL => "SETL",
            OpCode::BACK7 => "BACK7",
            OpCode::UKN => "UKN",
        };
        write!(f, "{}", str_opcode)
    }
}

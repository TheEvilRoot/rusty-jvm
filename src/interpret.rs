pub enum Opcode {
    AConstNull,
    IConst(i8),
    IAdd,
    IAnd,
    I2B,
    I2C,
    I2D,
    I2F,
    I2L,
    I2S,
    IMul
}

#[derive(Debug)]
pub enum InterpreterError {
    UnimplementedOpcode(u8)
}

pub struct Interpreter {
}

impl Interpreter {
    pub fn new() -> Self {
        return Interpreter { }
    }

    pub fn decode(&self, byte_code: u8) -> Result<Opcode, InterpreterError> {
        match byte_code {
            0x01 => Ok(Opcode::AConstNull),
            0x02 => Ok(Opcode::IConst(-1)),
            0x03 => Ok(Opcode::IConst(0)),
            0x04 => Ok(Opcode::IConst(1)),
            0x05 => Ok(Opcode::IConst(2)),
            0x06 => Ok(Opcode::IConst(3)),
            0x07 => Ok(Opcode::IConst(4)),
            0x08 => Ok(Opcode::IConst(5)),
            0x60 => Ok(Opcode::IAdd),
            0x7e => Ok(Opcode::IAnd),
            0x91 => Ok(Opcode::I2B),
            0x92 => Ok(Opcode::I2C),
            0x87 => Ok(Opcode::I2D),
            0x86 => Ok(Opcode::I2F),
            0x85 => Ok(Opcode::I2L),
            0x93 => Ok(Opcode::I2S),
            _ => Err(InterpreterError::UnimplementedOpcode(byte_code))
        }
    }
}
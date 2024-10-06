use std::str::FromStr;
use crate::error::Error;

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    PARAM,
    MOV,
    SWAP,
    CMP,
    ADD,
    SUB,
    LOAD,
    STORE,
    INC,
    DEC,
    INT,
    JMP,
    JE,
    JNE,
    PUSH,
    POP,
}

impl FromStr for Operation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PARAM" => Ok(Operation::PARAM),
            "MOV" => Ok(Operation::MOV),
            "SWAP" => Ok(Operation::SWAP),
            "CMP" => Ok(Operation::CMP),
            "ADD" => Ok(Operation::ADD),
            "SUB" => Ok(Operation::SUB),
            "LOAD" => Ok(Operation::LOAD),
            "STORE" => Ok(Operation::STORE),
            "INC" => Ok(Operation::INC),
            "DEC" => Ok(Operation::DEC),
            "INT" => Ok(Operation::INT),
            "JMP" => Ok(Operation::JMP),
            "JE" => Ok(Operation::JE),
            "JNE" => Ok(Operation::JNE),
            "PUSH" => Ok(Operation::PUSH),
            "POP" => Ok(Operation::POP),
            &_ => Err(Self::Err::ParseOperationError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub operation: Operation,
    pub operands: Vec<String>,
}

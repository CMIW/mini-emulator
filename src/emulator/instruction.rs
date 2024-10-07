use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Instruction {
    pub operation: Operation,
    pub operands: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_deserialize_operation() {
        let operation = Operation::MOV;

        let serialize = bincode::serialize(&operation).unwrap();

        let deserialize = bincode::deserialize(&serialize).unwrap();

        assert_eq!(operation, deserialize);
    }

    #[test]
    fn serialize_deserialize_instruction() {
        let instruction = Instruction {
            operation: Operation::MOV,
            operands: vec!["AX".to_string(), "5".to_string()],
        };

        let serialize = bincode::serialize(&instruction).unwrap();

        let deserialize = bincode::deserialize(&serialize).unwrap();

        assert_eq!(instruction, deserialize);
    }

    #[test]
    fn serialize_deserialize_instructions() {
        let instructions = vec![
            Instruction {
                operation: Operation::MOV,
                operands: vec!["AX".to_string(), "5".to_string()],
            },
            Instruction {
                operation: Operation::MOV,
                operands: vec!["AX".to_string(), "5".to_string()],
            },
        ];

        let serialize = bincode::serialize(&instructions).unwrap();

        let deserialize: Vec<Instruction> = bincode::deserialize(&serialize).unwrap();

        assert_eq!(instructions, deserialize);
    }
}

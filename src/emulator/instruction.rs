use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::io::Write;
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

impl From<u8> for Operation {
    fn from(i: u8) -> Self {
        match i {
            1 => Operation::PARAM,
            2 => Operation::MOV,
            3 => Operation::SWAP,
            4 => Operation::CMP,
            5 => Operation::ADD,
            6 => Operation::SUB,
            7 => Operation::LOAD,
            8 => Operation::STORE,
            9 => Operation::INC,
            10 => Operation::DEC,
            11 => Operation::INT,
            12 => Operation::JMP,
            13 => Operation::JE,
            14 => Operation::JNE,
            15 => Operation::PUSH,
            16 => Operation::POP,
            _ => todo!(),
        }
    }
}

impl From<Operation> for u8 {
    fn from(o: Operation) -> u8 {
        match o {
            Operation::PARAM => 1,
            Operation::MOV => 2,
            Operation::SWAP => 3,
            Operation::CMP => 4,
            Operation::ADD => 5,
            Operation::SUB => 6,
            Operation::LOAD => 7,
            Operation::STORE => 8,
            Operation::INC => 9,
            Operation::DEC => 10,
            Operation::INT => 11,
            Operation::JMP => 12,
            Operation::JE => 13,
            Operation::JNE => 14,
            Operation::PUSH => 15,
            Operation::POP => 16,
        }
    }
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

impl Operation {
    pub fn maybe_from(byte: u8) -> Option<Self> {
        match byte {
            1..16 => Some(Operation::from(byte)),
            _ => None,
        }
    }

    pub fn maybe_into(option: Option<Operation>) -> u8 {
        match option {
            Some(operation) => operation.into(),
            None => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Instruction {
    pub operation: Operation,
    pub operands: Vec<String>,
}

impl From<&[u8]> for Instruction {
    fn from(bytes: &[u8]) -> Self {
        let operands_len = bytes[1];
        let mut operands: Vec<String> = vec![];

        let mut i = 2;
        // Read the rest of the bytes and converts them to String and store them on a list
        while i < operands_len {
            let len = bytes[i as usize];
            operands.push(
                std::str::from_utf8(&bytes[(i + 1) as usize..(i + 1 + len) as usize])
                    .unwrap()
                    .to_string(),
            );
            i += len + 1;
        }

        Self {
            // The operand is stored on the first byte
            operation: Operation::from(bytes[0]),
            operands,
        }
    }
}

impl From<Instruction> for Vec<u8> {
    fn from(i: Instruction) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        bytes.push(i.operation.into());
        for operand in i.operands {
            let operand_bytes = operand.as_bytes();
            bytes.push(operand_bytes.len() as u8);
            let _ = bytes.write(operand_bytes);
        }
        bytes.insert(1, (bytes.len() + 1) as u8);
        bytes
    }
}

pub fn to_bytes(instructions: Vec<Instruction>) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![];

    for instruction in instructions {
        let mut instruction_u8: Vec<u8> = instruction.clone().into();
        instruction_u8.insert(0, (instruction_u8.len() + 1) as u8);
        bytes.append(&mut instruction_u8);
    }

    bytes
}

pub fn from_bytes(bytes: &[u8]) -> Vec<Instruction> {
    let mut instructions: Vec<Instruction> = vec![];

    let mut i = 0;

    while i < bytes.len() {
        let len = bytes[i] as usize;
        let test = &bytes[(i + 1)..(i + len)];

        instructions.push(Instruction::from(test));
        i += len;
    }

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_into_operation() {
        let operation = Operation::MOV;
        let operation_u8: u8 = operation.into();
        let deserialize: Operation = Operation::from(operation_u8);

        assert_eq!(operation, deserialize);
    }

    #[test]
    fn from_into_instruction() {
        let instruction = Instruction {
            operation: Operation::MOV,
            operands: vec!["AX".to_string(), "5".to_string()],
        };

        let instruction_u8: Vec<u8> = instruction.clone().into();
        let deserialize: Instruction = Instruction::from(&instruction_u8[..]);

        assert_eq!(instruction, deserialize);
    }

    #[test]
    fn from_into_instructions() {
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

        let serialize = to_bytes(instructions.clone());

        let deserialize: Vec<Instruction> = from_bytes(&serialize);

        assert_eq!(instructions, deserialize);
    }
}

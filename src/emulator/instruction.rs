use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::fmt;
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

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operation::PARAM => write!(f, "PARAM"),
            Operation::MOV => write!(f, "MOV"),
            Operation::SWAP => write!(f, "SWAP"),
            Operation::CMP => write!(f, "CMP"),
            Operation::ADD => write!(f, "ADD"),
            Operation::SUB => write!(f, "SUB"),
            Operation::LOAD => write!(f, "LOAD"),
            Operation::STORE => write!(f, "STORE"),
            Operation::INC => write!(f, "INC"),
            Operation::DEC => write!(f, "DEC"),
            Operation::INT => write!(f, "INT"),
            Operation::JMP => write!(f, "JMP"),
            Operation::JE => write!(f, "JE"),
            Operation::JNE => write!(f, "JNE"),
            Operation::PUSH => write!(f, "PUSH"),
            Operation::POP => write!(f, "POP"),
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

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
// ["AX", "BX", "CX", "DX"]
pub enum Register {
    AX,
    BX,
    CX,
    DX,
}

impl From<u8> for Register {
    fn from(i: u8) -> Self {
        match i {
            1 => Register::AX,
            2 => Register::BX,
            3 => Register::CX,
            4 => Register::DX,
            _ => todo!(),
        }
    }
}

impl From<Register> for u8 {
    fn from(o: Register) -> u8 {
        match o {
            Register::AX => 1,
            Register::BX => 2,
            Register::CX => 3,
            Register::DX => 4,
        }
    }
}

impl FromStr for Register {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AX" => Ok(Register::AX),
            "BX" => Ok(Register::BX),
            "CX" => Ok(Register::CX),
            "DX" => Ok(Register::DX),
            &_ => Err(Self::Err::ParseRegisterError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
// ["09H", "10H", "20H"]
pub enum Interupt {
    H09,
    H10,
    H20,
}

impl From<u8> for Interupt {
    fn from(i: u8) -> Self {
        match i {
            1 => Interupt::H09,
            2 => Interupt::H10,
            3 => Interupt::H20,
            _ => todo!(),
        }
    }
}

impl From<Interupt> for u8 {
    fn from(o: Interupt) -> u8 {
        match o {
            Interupt::H09 => 1,
            Interupt::H10 => 2,
            Interupt::H20 => 3,
        }
    }
}

impl FromStr for Interupt {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "09H" => Ok(Interupt::H09),
            "10H" => Ok(Interupt::H10),
            "20H" => Ok(Interupt::H20),
            &_ => Err(Self::Err::ParseInteruptError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum Operands {
    V0,
    // - 2
    V1(u8, u8),
    // AX
    V2(Register),
    // 09H
    V3(Interupt),
    // [p1, p2, p3]
    V4(u8, u8, u8),
    // AX, 2
    V5(Register, u8),
    // AX, BX
    V6(Register, Register),
}

impl From<Operands> for Vec<u8> {
    fn from(o: Operands) -> Vec<u8> {
        match o {
            Operands::V0 => vec![0, 0, 0, 0],
            Operands::V1(sing, num) => vec![1, sing, num, 0],
            Operands::V2(register) => vec![2, register.into(), 0, 0],
            Operands::V3(interupt) => vec![3, interupt.into(), 0, 0],
            Operands::V4(p1, p2, p3) => vec![4, p1, p2, p3],
            Operands::V5(register, num) => vec![5, register.into(), num, 0],
            Operands::V6(register1, register2) => vec![6, register1.into(), register2.into(), 0],
        }
    }
}

impl From<&[u8]> for Operands {
    fn from(bytes: &[u8]) -> Operands {
        match bytes[0] {
            0 => Operands::V0,
            1 => Operands::V1(bytes[1], bytes[2]),
            2 => Operands::V2(Register::from(bytes[1])),
            3 => Operands::V3(Interupt::from(bytes[1])),
            4 => Operands::V4(bytes[1],bytes[2], bytes[3]),
            5 => Operands::V5(Register::from(bytes[1]), bytes[2]),
            6 => Operands::V6(Register::from(bytes[1]), Register::from(bytes[2])),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Instruction {
    pub operation: Operation,
    pub operands: Operands,
}

impl From<&[u8]> for Instruction {
    fn from(bytes: &[u8]) -> Self {
        Self {
            // The operand is stored on the first byte
            operation: Operation::from(bytes[0]),
            operands: Operands::from(&bytes[1..]),
        }
    }
}

impl From<Instruction> for Vec<u8> {
    fn from(i: Instruction) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        let mut operands: Vec<u8> = i.operands.into();
        bytes.push(i.operation.into());
        bytes.append(&mut operands);
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
            operands: Operands::V5(Register::AX, 5),
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
                operands: Operands::V5(Register::AX, 5),
            },
            Instruction {
                operation: Operation::MOV,
                operands: Operands::V5(Register::AX, 5),
            },
        ];

        let serialize = to_bytes(instructions.clone());

        let deserialize: Vec<Instruction> = from_bytes(&serialize);

        assert_eq!(instructions, deserialize);
    }
}

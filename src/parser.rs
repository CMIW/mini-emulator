use crate::emulator::{Instruction, Interupt, Operands, Operation, Register};
use crate::error::Error;
use std::str::FromStr;

const REGISTERS: [&str; 4] = ["AX", "BX", "CX", "DX"];
const INTERUPTS: [&str; 3] = ["09H", "10H", "20H"];

// Parse the asm file
pub fn read_file(stream: &[u8]) -> Result<Vec<Instruction>, Error> {
    // Read bytes to string and remove trailing spaces
    let string = match std::str::from_utf8(stream) {
        Ok(content) => content.trim(),
        Err(_) => return Err(Error::Utf8Error),
    };

    let mut instructions: Vec<Instruction> = vec![];

    // Read each line of the file
    for (i, line) in string.lines().enumerate() {
        let line = &line.replace(",", "");
        let mut instruction = line.split(" ").collect::<Vec<&str>>();
        instruction.reverse();

        let operation = instruction.pop().unwrap();

        // Ingore empty lines
        if !operation.is_empty() {
            instruction.reverse();

            // Validate the operation part of the expresion
            let operation = match Operation::from_str(operation) {
                Ok(operation) => operation,
                Err(_) => return Err(Error::InvalidOperation(i, operation.to_string())),
            };

            // Validate the number of operators
            let operands = validate_operators(i, &operation, &instruction)?;

            instructions.push(Instruction {
                operation,
                operands,
            });
        }
    }

    Ok(instructions)
}

fn validate_operators(
    row: usize,
    operation: &Operation,
    operators: &[&str],
) -> Result<Operands, Error> {
    match operation {
        Operation::PARAM => {
            if operators.len() > 3 || operators.is_empty() {
                return Err(Error::InvalidNumberOperands(
                    row,
                    *operation,
                    operators.iter().map(|s| s.to_string()).collect(),
                ));
            } else {
                for param in operators {
                    if REGISTERS.contains(param) {
                        return Err(Error::InvalidOperand(row, *operation, param.to_string()));
                    }
                }
            }
            let line1 = &operators[0].replace("-", "");
            let line1 = line1.replace("+", "");
            match line1.parse::<u8>() {
                Ok(num1) => {
                    let line2 = &operators[1].replace("-", "");
                    let line2 = line2.replace("+", "");
                    match line2.parse::<u8>() {
                        Ok(num2) => {
                            let line3 = &operators[2].replace("-", "");
                            let line3 = line3.replace("+", "");
                            match line3.parse::<u8>() {
                                Ok(num3) => Ok(Operands::V4(num1, num2, num3)),
                                Err(_) => Err(Error::ParseIntError),
                            }
                        }
                        Err(_) => Err(Error::ParseIntError),
                    }
                }
                Err(_) => Err(Error::ParseIntError),
            }
        }
        Operation::MOV => {
            if operators.len() != 2 {
                return Err(Error::InvalidNumberOperands(
                    row,
                    *operation,
                    operators.iter().map(|s| s.to_string()).collect(),
                ));
            } else if !REGISTERS.contains(&operators[0]) {
                return Err(Error::InvalidOperand(
                    row,
                    *operation,
                    operators[0].to_string(),
                ));
            }
            match Register::from_str(operators[0]) {
                Ok(r1) => {
                    if REGISTERS.contains(&operators[1]) {
                        match Register::from_str(operators[1]) {
                            Ok(r2) => Ok(Operands::V6(r1, r2)),
                            Err(err) => Err(err),
                        }
                    } else {
                        let line = &operators[1].replace("-", "");
                        let line = line.replace("+", "");
                        match line.parse::<u8>() {
                            Ok(num) => Ok(Operands::V5(r1, num)),
                            Err(_) => Err(Error::ParseIntError),
                        }
                    }
                }
                Err(err) => Err(err),
            }
        }
        Operation::SWAP | Operation::CMP => {
            if operators.len() != 2 {
                return Err(Error::InvalidNumberOperands(
                    row,
                    *operation,
                    operators.iter().map(|s| s.to_string()).collect(),
                ));
            } else {
                for operator in operators {
                    if !REGISTERS.contains(operator) {
                        return Err(Error::InvalidOperand(row, *operation, operator.to_string()));
                    }
                }
            }
            match Register::from_str(operators[0]) {
                Ok(r1) => match Register::from_str(operators[1]) {
                    Ok(r2) => Ok(Operands::V6(r1, r2)),
                    Err(err) => Err(err),
                },
                Err(err) => Err(err),
            }
        }
        Operation::JMP | Operation::JE | Operation::JNE => {
            if operators.len() != 1 {
                return Err(Error::InvalidNumberOperands(
                    row,
                    *operation,
                    operators.iter().map(|s| s.to_string()).collect(),
                ));
            } else if REGISTERS.contains(&operators[0]) {
                return Err(Error::InvalidOperand(
                    row,
                    *operation,
                    operators[0].to_string(),
                ));
            }
            if operators[0].contains("-") {
                let line = &operators[0].replace("-", "");
                match line.parse::<u8>() {
                    Ok(num) => Ok(Operands::V1(1, num)),
                    Err(_) => Err(Error::ParseIntError),
                }
            } else {
                let line = &operators[0].replace("+", "");
                match line.parse::<u8>() {
                    Ok(num) => Ok(Operands::V1(0, num)),
                    Err(_) => Err(Error::ParseIntError),
                }
            }
        }
        Operation::ADD
        | Operation::SUB
        | Operation::LOAD
        | Operation::STORE
        | Operation::PUSH
        | Operation::POP => {
            if operators.len() != 1 {
                return Err(Error::InvalidNumberOperands(
                    row,
                    *operation,
                    operators.iter().map(|s| s.to_string()).collect(),
                ));
            } else if !REGISTERS.contains(&operators[0]) {
                return Err(Error::InvalidOperand(
                    row,
                    *operation,
                    operators[0].to_string(),
                ));
            }
            match Register::from_str(operators[0]) {
                Ok(test) => Ok(Operands::V2(test)),
                Err(err) => Err(err),
            }
        }
        Operation::INT => {
            if operators.len() != 1 {
                return Err(Error::InvalidNumberOperands(
                    row,
                    *operation,
                    operators.iter().map(|s| s.to_string()).collect(),
                ));
            } else if !INTERUPTS.contains(&operators[0]) {
                return Err(Error::InvalidOperand(
                    row,
                    *operation,
                    operators[0].to_string(),
                ));
            }
            match Interupt::from_str(operators[0]) {
                Ok(test) => Ok(Operands::V3(test)),
                Err(err) => Err(err),
            }
        }
        Operation::INC | Operation::DEC => {
            if operators.len() > 1 {
                return Err(Error::InvalidNumberOperands(
                    row,
                    *operation,
                    operators.iter().map(|s| s.to_string()).collect(),
                ));
            } else if operators.len() == 1 && !REGISTERS.contains(&operators[0]) {
                return Err(Error::InvalidOperand(
                    row,
                    *operation,
                    operators[0].to_string(),
                ));
            } else if operators.len() == 1 {
                match Register::from_str(operators[0]) {
                    Ok(test) => {
                        return Ok(Operands::V2(test));
                    }
                    Err(err) => return Err(err),
                };
            } else {
                return Ok(Operands::V0);
            }
        }
    }
}

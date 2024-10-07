use crate::emulator::{Instruction, Operation};
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
            validate_operators(i, &operation, &instruction)?;

            instructions.push(Instruction {
                operation,
                operands: instruction.into_iter().map(|s| s.to_string()).collect(),
            });
        }
    }

    Ok(instructions)
}

fn validate_operators(row: usize, operation: &Operation, operators: &[&str]) -> Result<(), Error> {
    match operation {
        Operation::PARAM => {
            if operators.len() != 3 {
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
        }
        Operation::ADD
        | Operation::SUB
        | Operation::LOAD
        | Operation::STORE
        | Operation::JMP
        | Operation::JE
        | Operation::JNE
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
            }
        }
    }

    Ok(())
}

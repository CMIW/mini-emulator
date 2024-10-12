use crate::emulator::Operation;
use std::io;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Not a valid value, value must be <= 255.")]
    ParseIntError,
    #[error("File select dialog closed.")]
    DialogClosed,
    #[error("IO Error")]
    IO(io::ErrorKind),
    #[error("No file has been opened, first open a file the you can save it.")]
    NoFileOpened,
    #[error("The given path does not contain a file name.")]
    NotFile,
    #[error("Not enough space in memory, {0} won't be stored.")]
    NotEnoughStorage(String),
    #[error("Not enough space in user memory.")]
    NotEnoughUserMemory,
    #[error("Not enough space in OS memory.")]
    NotEnoughOsMemory,
    #[error("File should contain valid utf8")]
    Utf8Error,
    #[error("Invalid Operation {1} on line {0},")]
    InvalidOperation(usize, String),
    #[error("Invalid Operation '{0}'.")]
    ParseOperationError(String),
    #[error("Invalid Register '{0}'.")]
    ParseRegisterError(String),
    #[error("Invalid Interupt code '{0}'.")]
    ParseInteruptError(String),
    #[error("Invalid number of operands for {1:?}: {2:?} on line: {0}.")]
    InvalidNumberOperands(usize, Operation, Vec<String>),
    #[error("Invalid operand '{2:?}' for {1:?} on line: {0}.")]
    InvalidOperand(usize, Operation, String),
}

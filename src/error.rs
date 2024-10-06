use std::io;
use thiserror::Error;
use crate::emulator::Operation;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("File select dialog closed.")]
    DialogClosed,
    #[error("IO Error")]
    IO(io::ErrorKind),
    #[error("No file has been opened, first open a file the you can save it.")]
    NoFileOpened,
    #[error("The given path does not contain a file name.")]
    NotFile,
    #[error("Not enough space in memory, {0} won't be stored.")]
    NotEnoughMemory(String),
    #[error("File should contain valid utf8")]
    Utf8Error,
    #[error("Invalid Operation {1} on line {0},")]
    InvalidOperation(usize, String),
    #[error("Invalid Operation '{0}'.")]
    ParseOperationError(String),
    #[error("Invalid number of operands for {1:?}: {2:?} on line: {0}.")]
    InvalidNumberOperands(usize, Operation, Vec<String>),
    #[error("Invalid operand '{2:?}' for {1:?} on line: {0}.")]
    InvalidOperand(usize, Operation, String),
}

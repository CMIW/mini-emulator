use serde::{Deserialize, Serialize};

use crate::emulator::Operation;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ProcessState {
    New,
    Ready,
    Running,
    Waiting,
    Terminated
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PCB {
    pub id: usize,
    pub code_segment: usize,
    pub code_segment_size: usize,
    pub stack_segment: usize,
    pub stack_segment_size: usize,
    pub process_state: ProcessState,
    pub priority: u8,
    pub ax: u64,
    pub bx: u64,
    pub cx: u64,
    pub dx: u64,
    pub ac: u64,
    pub pc: u64,
    pub sp: u64,
    pub ir: Option<Operation>,
    pub z: bool,
}

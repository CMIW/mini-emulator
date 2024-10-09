use std::default::Default;
use serde::{Deserialize, Serialize};

use crate::emulator::Operation;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub enum ProcessState {
    #[default]
    New,
    Ready,
    Running,
    Waiting,
    Terminated
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct PCB {
    pub id: usize,
    pub code_segment: usize,
    pub code_segment_size: usize,
    pub stack_segment: usize,
    pub stack_segment_size: usize,
    pub process_state: ProcessState,
    pub priority: u8,
    pub ax: u8,
    pub bx: u8,
    pub cx: u8,
    pub dx: u8,
    pub ac: u8,
    pub pc: u8,
    pub sp: u8,
    pub ir: Option<Operation>,
    pub z: bool,
}

impl PCB {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn code_segment(&mut self, address: usize, size: usize) -> &mut Self {
        self.code_segment = address;
        self.code_segment_size = size;

        self
    }

    pub fn stack_segment(&mut self, address: usize, size: usize) -> &mut Self {
        self.stack_segment = address;
        self.stack_segment_size = size;

        self
    }
}

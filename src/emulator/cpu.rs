use crate::emulator::Operation;

#[derive(Debug, Default)]
pub struct CPU {
    pub ax: u8,
    pub bx: u8,
    pub cx: u8,
    pub dx: u8,
    pub ac: u8,
    pub pc: usize,
    pub sp: usize,
    pub ir: Option<Operation>,
    pub z: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

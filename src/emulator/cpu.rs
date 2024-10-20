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

    pub fn is_empty(&self) -> bool {
        self.ax == 0 && self.bx == 0 && self.cx == 0 && self.dx == 0 && self.ac == 0 && self.pc == 0 && self.sp == 0 && self.ir == None && self.z == false
    }
}

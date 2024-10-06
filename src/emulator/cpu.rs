#[derive(Debug, Default)]
pub struct CPU {
    pub ax: u64,
    pub bx: u64,
    pub cx: u64,
    pub dx: u64,
    pub ac: u64,
    pub pc: u64,
    pub sp: u64,
    pub z: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

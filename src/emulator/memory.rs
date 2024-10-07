use crate::error::Error;

#[derive(Debug, Default)]
pub struct Memory {
    pub data: Vec<u8>,
    os_segment_size: usize,
    // (address, size)
    pub used: Vec<(usize, usize)>,
    // (address, size)
    pub freed: Vec<(usize, usize)>,
}

impl Memory {
    fn new(size: usize, os_segment: usize) -> Self {
        Self {
            data: vec![0; size],
            os_segment_size: os_segment,
            used: vec![],
            freed: vec![],
        }
    }
}

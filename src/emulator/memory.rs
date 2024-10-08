use crate::error::Error;
use crate::emulator::PCB;

#[derive(Debug, Default)]
pub struct Memory {
    pub data: Vec<u8>,
    os_segment_size: usize,
    // (address, size)
    pub used: Vec<(usize, usize)>,
    // (address, size)
    pub freed: Vec<(usize, usize)>,
    // (pcb_id, address, size)
    pub pcb_table: Vec<(usize, usize, usize)>
}

impl Memory {
    pub fn new(size: usize, os_segment: usize) -> Self {
        Self {
            data: vec![0; size],
            os_segment_size: os_segment,
            used: vec![],
            freed: vec![],
            pcb_table: vec![],
        }
    }

    pub fn store(&mut self, data: Vec<u8>, size: usize) -> Result<(usize, usize), Error> {
        // No memory space has been freed
        if !self.freed.is_empty() {
            todo!();
        }
        // No memory has been used
        else if self.used.is_empty() {
            if (self.data.len() - self.os_segment_size) > size {
                // Copy data to "memory"
                self.data[self.os_segment_size..size].copy_from_slice(&data[..]);
                self.used.push((self.os_segment_size, size));
                return Ok((self.os_segment_size, size))
            } else {
                return Err(Error::NotEnoughUserMemory);
            }
        } else {
            // Last used memory information
            let (address, data_size) = &self.used[self.used.len()-1];

            let next_address = address + data_size;
            let available_space = (self.data.len() - self.os_segment_size) - next_address;

            if available_space > size {
                self.data[next_address..next_address + size].copy_from_slice(&data[..]);
                self.used.push((next_address, size));
                return Ok((next_address, size));
            } else {
                return Err(Error::NotEnoughUserMemory);
            }
        }
    }

    pub fn store_pcb(&mut self, pcb: PCB) -> Result<(), Error> {
        let bytes = bincode::serialize(&pcb).unwrap();
        // No PCB has been stored
        if self.pcb_table.is_empty() {
            if self.os_segment_size > bytes.len() {
                self.data[0..bytes.len()].copy_from_slice(&bytes[..]);
                self.pcb_table.push((pcb.id, 0, bytes.len()));
            } else {
                return Err(Error::NotEnoughOsMemory);
            }
        } else {
            // Last stored pcb
            let (_, address, data_size) = &self.pcb_table[self.pcb_table.len()-1];

            let next_address = address + data_size;
            let available_space = self.os_segment_size - next_address;

            if available_space > bytes.len() {
                self.data[next_address..bytes.len()].copy_from_slice(&bytes[..]);
                self.pcb_table.push((pcb.id, next_address, bytes.len()));
            } else {
                return Err(Error::NotEnoughOsMemory);
            }
        }
        Ok(())
    }
}

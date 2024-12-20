use crate::emulator::{ProcessState, PCB};
use crate::error::Error;

#[derive(Debug, Default)]
pub struct Memory {
    pub data: Vec<u8>,
    os_segment_size: usize,
    // (address, size)
    pub used: Vec<(usize, usize)>,
    // (address, size)
    pub freed: Vec<(usize, usize)>,
    // (pcb_id, address, size)
    pub pcb_table: Vec<(usize, usize, usize)>,
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
        // Some memory space has been freed
        if !self.freed.is_empty() && !self.used.is_empty() {
            for (i, (address, m_size)) in self.freed.clone().iter_mut().enumerate() {
                if size == *m_size {
                    println!("{:?} {:?}", &size, &m_size);
                    self.data[*address..*address + size].copy_from_slice(&data[..]);
                    self.used.push(self.freed.remove(i));
                    return Ok((*address, *m_size));
                }
            }
        }
        // No memory has been used
        if self.used.is_empty() {
            if (self.data.len() - self.os_segment_size) > size {
                // Copy data to "memory"
                self.data[self.os_segment_size..self.os_segment_size + size]
                    .copy_from_slice(&data[..]);
                self.used.push((self.os_segment_size, size));
                Ok((self.os_segment_size, size))
            } else {
                Err(Error::NotEnoughUserMemory)
            }
        } else {
            // Last used memory information
            let (address, data_size) = &self.used.last().unwrap();

            // We need to know if there is enough space in memory
            let next_address = address + data_size;
            let available_space = self.data.len() - next_address;
            // Store the data in memory when we have the space
            if available_space > size {
                self.data[next_address..next_address + size].copy_from_slice(&data[..]);
                self.used.push((next_address, size));
                Ok((next_address, size))
            } else {
                Err(Error::NotEnoughUserMemory)
            }
        }
    }

    // Move the memory space data to the freed queue
    pub fn free_memory(&mut self, address: usize) -> Result<(), Error> {
        if let Some(position) = self.used.iter().position(|x| x.0 == address) {
            let space = self.used.remove(position);
            // Set memory to 0
            self.data[space.0..space.0 + space.1].copy_from_slice(&vec![0; space.1]);
            self.freed.push(space);
            if self.used.is_empty() {
                self.freed.clear();
            }
        }

        Ok(())
    }

    pub fn store_pcb(&mut self, pcb: PCB) -> Result<(), Error> {
        let bytes: Vec<u8> = pcb.into();
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
            let (_, address, data_size) = &self.pcb_table.last().unwrap();

            let next_address = address + data_size;
            let available_space = self.os_segment_size - next_address;

            if available_space > bytes.len() {
                self.data[next_address..next_address + bytes.len()].copy_from_slice(&bytes[..]);
                self.pcb_table.push((pcb.id, next_address, bytes.len()));
            } else {
                return Err(Error::NotEnoughOsMemory);
            }
        }
        Ok(())
    }

    pub fn last_pcb_id(&self) -> usize {
        match self.pcb_table.last() {
            Some((id, _, _)) => *id,
            None => 0,
        }
    }

    pub fn running_process(&self) -> Option<((usize, usize, usize), PCB)> {
        for (id, address, data_size) in &self.pcb_table {
            let pcb = PCB::from(&self.data[*address..*address + *data_size]);
            if pcb.process_state == ProcessState::Running {
                return Some(((*id, *address, *data_size), pcb));
            }
        }
        None
    }

    pub fn free_size(&self) -> usize {
        let mut data = self.data.clone();
        data.retain(|x| *x == 0);

        data.len().saturating_sub(self.os_segment_size)
    }
}

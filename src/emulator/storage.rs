use crate::error::Error;

#[derive(Debug, Default)]
pub struct Storage {
    pub data: Vec<u8>,
    pub used: Vec<(String, usize, usize)>,
    pub freed: Vec<(String, usize, usize)>,
}

impl Storage {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            used: vec![],
            freed: vec![],
        }
    }

    pub fn store_files(
        &mut self,
        file_name: &str,
        size: usize,
        data: Vec<u8>,
    ) -> Result<(), Error> {
        // No memory space has been freed
        if !self.freed.is_empty() {
            todo!();
        }
        // No memory has been used
        else if self.used.is_empty() {
            if self.data.len() > size {
                self.data[0..size].copy_from_slice(&data[..]);
                self.used.push((file_name.to_string(), 0, size));
            } else {
                return Err(Error::NotEnoughMemory(file_name.to_string()));
            }
        } else {
            // last used memory information
            let (_, address, data_size) = &self.used[self.used.len() - 1];

            let next_address = address + data_size;
            let available_space = self.data.len() - next_address;

            if available_space > size {
                self.data[next_address..next_address + size].copy_from_slice(&data[..]);
                self.used.push((file_name.to_string(), next_address, size));
            } else {
                return Err(Error::NotEnoughMemory(file_name.to_string()));
            }
        }

        Ok(())
    }
}

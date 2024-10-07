use std::default::Default;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Config {
    pub memory: usize,
    pub storage: usize,
    os_segment: usize,
    user_segment: usize,
    virtual_memory: usize,
}

/*impl Config {
    fn new(
        memory: usize,
        storage: usize,
        os_segment: usize,
        user_segment: usize,
        virtual_memory: usize,
    ) -> Self {
        Self {
            memory,
            storage,
            os_segment,
            user_segment,
            virtual_memory,
        }
    }
}*/

impl Default for Config {
    fn default() -> Self {
        Self {
            memory: 256,
            storage: 512,
            os_segment: 120,
            user_segment: 100,
            virtual_memory: 64,
        }
    }
}

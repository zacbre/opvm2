use extism::{convert::Json, FromBytes, ToBytes};
use serde::{Deserialize, Serialize};

const MAX_MEMORY_SIZE: usize = 1024 * 1024 * 16; // max 16MB in 32 bits

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToBytes, FromBytes)]
#[encoding(Json)]
pub struct Memory {
    data: [u8; MAX_MEMORY_SIZE],
    pointer: u32,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            data: [0; MAX_MEMORY_SIZE],
            pointer: 0,
        }
    }

    pub fn push(&mut self, data: &[u8]) -> u32 {
        let start = self.pointer;
        for (i, byte) in data.iter().enumerate() {
            self.data[self.pointer as usize + i] = *byte;
        }
        self.pointer += data.len() as u32 + 1; // empty string.
        start
    }
}

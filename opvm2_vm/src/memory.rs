use extism::{convert::Json, FromBytes, ToBytes};
use serde::{Deserialize, Serialize};

const MAX_MEMORY_SIZE: usize = 1024 * 1024; // 1MB of memory, can be adjustable, but tests get extremely slow with any more.

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToBytes, FromBytes, Clone,
)]
#[encoding(Json)]
pub struct Memory {
    data: Vec<u8>,
    pointer: usize,
}

impl Memory {
    // todo: build a better allocator besides just a bump allocator
    pub fn new() -> Self {
        Self {
            data: vec![0; MAX_MEMORY_SIZE],
            pointer: 0,
        }
    }

    pub fn push(&mut self, data: &[u8], spacer: bool) -> usize {
        let start = self.pointer;
        for (i, byte) in data.iter().enumerate() {
            self.data[self.pointer as usize + i] = *byte;
        }
        self.pointer += data.len() as usize; // empty string.
        if spacer {
            self.pointer += 1;
        }
        start
    }

    pub fn get_instruction(&mut self, pointer: usize) -> u128 {
        let start = pointer as usize;
        let end = pointer + 16 as usize;
        let mut instruction = 0;
        for i in 0..(end - start) {
            instruction |= (self.data[start + i] as u128) << (i * 8);
        }
        instruction
    }

    pub fn get_literal(&mut self, pointer: usize) -> &[u8] {
        let start = pointer as usize;
        let mut end = start;
        while self.data[end] != 0 {
            end += 1;
        }
        &self.data[start..end]
    }

    pub fn address(&self) -> usize {
        self.pointer
    }

    pub fn raw(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn from_raw(raw: Vec<u8>, base: usize) -> Self {
        Self {
            data: raw,
            pointer: base,
        }
    }
}

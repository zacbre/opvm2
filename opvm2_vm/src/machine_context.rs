use extism::{convert::Json, *};
use opvm2::{register::Registers, stack::Stack};
use serde::{Deserialize, Serialize};

use crate::memory::Memory;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToBytes, FromBytes)]
#[encoding(Json)]
pub struct MachineContext {
    pub base_address: usize,
    pub registers: Registers,
    pub stack: Stack<usize>,
    pub call_stack: Stack<usize>,
    pub memory: Memory,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToBytes, FromBytes)]
#[encoding(Json)]
pub struct Label {
    pub name: String,
    pub address: usize,
}

impl MachineContext {
    pub fn new() -> MachineContext {
        MachineContext {
            registers: Registers::new(),
            stack: Stack::new(),
            call_stack: Stack::new(),
            memory: Memory::new(),
            base_address: 0,
        }
    }
}

use extism::{convert::Json, *};
use opvm2::{parser::program::Program, register::Registers, stack::Stack};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, FromBytes)]
#[encoding(Json)]
pub struct Store {
    // todo: this stuff should live in a machine context? that plugins have access to? then the VM should also have access to
    // at the same level as the plugin
    pub registers: Registers,
    pub stack: Stack<u64>,
    pub call_stack: Stack<u64>,
    pub current_program: Program,
}

impl Store {
    pub fn new() -> Store {
        Store {
            registers: Registers::new(),
            stack: Stack::new(),
            call_stack: Stack::new(),
            current_program: Program::empty(),
        }
    }
}

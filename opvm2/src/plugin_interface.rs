use crate::{
    instruction::Instruction,
    opcode::Opcode,
    operand::Operand,
    register::{Register, Registers},
};

use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, FromBytes, ToBytes, Clone)]
#[encoding(Json)]
pub struct OnInstructionValue {
    pub lhs: Operand,
    pub rhs: Operand,
    pub pc: usize,
    pub opcode: Opcode,
}

#[derive(Serialize, Deserialize, ToBytes, FromBytes, Debug, Clone, PartialEq, Eq)]
#[encoding(Json)]
pub struct Labels {
    pub list: Vec<Label>,
}

#[derive(Serialize, Deserialize, ToBytes, FromBytes, Debug, Clone, PartialEq, Eq)]
#[encoding(Json)]
pub struct Label {
    pub name: String,
    pub address: usize,
}

#[host_fn]
extern "ExtismHost" {
    pub fn all_registers() -> Registers;
    pub fn get_register(register: Register) -> u64;
    pub fn set_register(register: Register, value: u64);
    pub fn push_stack(value: u64);
    pub fn pop_stack() -> u64;
    pub fn get_input() -> String;
    pub fn jmp_to_label(label: String);
    pub fn get_labels() -> Labels;
    pub fn quit();
    pub fn print(value: String);
    pub fn execute(data: Instruction);
}

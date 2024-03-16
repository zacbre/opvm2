use crate::{
    operand::Operand,
    parser::program::Labels,
    register::{Register, Registers},
};

use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, FromBytes, ToBytes, Clone)]
#[encoding(Json)]
pub struct OnInstructionValue {
    pub lhs: Operand,
    pub rhs: Operand,
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
}

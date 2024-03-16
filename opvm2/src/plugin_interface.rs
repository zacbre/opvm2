use crate::{
    opcode::Opcode,
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
    pub pc: u64,
    pub opcode: Opcode,
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

    // todo: add a function to execute instructions within the vm...perhaps we have to patch current_program
    //pub fn execute_instruction(Json(ins): Json<OnInstructionValue>) -> Option<u64>;
}

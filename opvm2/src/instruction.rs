use std::collections::BTreeMap;

use extism_pdk::{FromBytes, Json, ToBytes};
use serde::{Deserialize, Serialize};

use crate::{
    opcode::{Opcode, PluginValue},
    operand::Operand,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToBytes, FromBytes)]
#[encoding(Json)]
pub struct Instruction {
    pub opcode: Opcode,
    pub lhs: Operand,
    pub rhs: Operand,
}

impl Instruction {
    pub fn new(opcode: Opcode, lhs: Operand, rhs: Operand) -> Instruction {
        Instruction { opcode, lhs, rhs }
    }

    pub fn new_l(opcode: Opcode, lhs: Operand) -> Instruction {
        Instruction {
            opcode,
            lhs,
            rhs: Operand::None,
        }
    }

    pub fn new_e(opcode: Opcode) -> Instruction {
        Instruction {
            opcode,
            lhs: Operand::None,
            rhs: Operand::None,
        }
    }

    // fuck it, we use a 128 bit instruction size, aka 16 bytes.
    // data map: [0-127]
    // [0-5] = opcode
    // [6-7] = operand count (none, lhs, rhs, both)
    // [8-40] = if plugin opcode, this is where the plugin string comes from (address, 32 bits)
    // [41-42] = lhs type (register, number, address, literal address?)
    // [43-75] = lhs (32 bit operator)
    // [76-77] = rhs type (register, number, address, literal address?)
    // [78-110] = rhs (32 bit operator)
    // [111-127] = 16 bits reserved
    // 32 bit memory address
    // operand mapping:
    // [0-4] = register
    // [0-32] = number
    // [0-32] = address

    // the problem with this, is we need to be able to pass in the addresses for plugin opcode, and literals mapped later.
    // actually, maybe we can just pass in a literal "map" that shows where each one was mapped to? and have that map
    // just be label offset => address, so we can resolve them here.
    // idk if i like this literal map, because we only get passed the label...we need a way to store these addresses in memory at this point?
    pub fn encode(&self, literal_map: &BTreeMap<String, usize>) -> u128 {
        let mut instruction = 0u128;
        instruction |= (self.opcode.to_u8() as u128) << 122;
        instruction |= (self.operand_count() as u128) << 120;
        instruction |= (self.lhs.operand_type() as u128) << 118;
        instruction |= (self.lhs.encode(literal_map) as u128) << 86;
        instruction |= (self.rhs.operand_type() as u128) << 84;
        instruction |= (self.rhs.encode(literal_map) as u128) << 52;
        // last 32 bits is for opcode custom name.
        if self.opcode.is_plugin() {
            // we should maybe insert this label into memory to use it there?
            instruction |= (self.opcode.get_plugin_address(literal_map) as u128) << 20;
        }

        // try to reconstruct:
        // println!("Opcode: {}", instruction >> 122);
        // println!("Operand Count: {}", (instruction >> 120) & 0b11);
        // println!("LHS Type: {}", (instruction >> 118) & 0b11);
        // println!("LHS: {}", (instruction >> 86) & 0xFFFFFFFF);
        // println!("RHS Type: {}", (instruction >> 84) & 0b11);
        // println!("RHS: {}", (instruction >> 52) & 0xFFFFFFFF);
        // // reconstruct the instruction
        // println!("{:?}", Self::decode(instruction));
        instruction
    }

    pub fn decode(instruction: u128) -> Instruction {
        let mut opcode = Opcode::from_u8((instruction >> 122) as u8);
        if opcode.is_plugin() {
            opcode = Opcode::Plugin(PluginValue::Address(
                ((instruction >> 20) & 0xFFFFFFFF) as u32,
            ));
        }

        let operand_count = (instruction >> 120) & 0b11;
        match operand_count {
            1 => {
                let lhs_type = (instruction >> 118) & 0b11;
                let lhs =
                    Operand::decode(lhs_type as u8, ((instruction >> 86) & 0xFFFFFFFF) as u32);
                Instruction::new_l(opcode, lhs)
            }
            2 => {
                let lhs_type = (instruction >> 118) & 0b11;
                let lhs =
                    Operand::decode(lhs_type as u8, ((instruction >> 86) & 0xFFFFFFFF) as u32);
                let rhs_type = (instruction >> 84) & 0b11;
                let rhs =
                    Operand::decode(rhs_type as u8, ((instruction >> 52) & 0xFFFFFFFF) as u32);
                Instruction::new(opcode, lhs, rhs)
            }
            _ => Instruction::new_e(opcode),
        }
    }

    fn operand_count(&self) -> u8 {
        match (&self.lhs, &self.rhs) {
            (Operand::None, Operand::None) => 0,
            (Operand::None, _) => 1,
            _ => 2,
        }
    }
}

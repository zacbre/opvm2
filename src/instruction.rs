use crate::{opcode::Opcode, operand::Operand};

#[derive(Debug, PartialEq, Clone)]
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
}

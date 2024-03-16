use extism_pdk::{FromBytes, Json, ToBytes};
use serde::{Deserialize, Serialize};

use crate::register::Register;

// operands have different types. for now we only have registers.
// we will add more types in the future.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToBytes, FromBytes)]
#[encoding(Json)]
pub enum Operand {
    None,        // no operand
    R(Register), // register
    N(u64),      // number
    L(String),   // label
}

impl TryFrom<String> for Operand {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // start trying from.
        if let Ok(val) = value.parse::<u64>() {
            return Ok(Operand::N(val));
        }

        if let Ok(val) = value.clone().try_into() {
            return Ok(Operand::R(val));
        }

        if let Ok(val) = value.clone().try_into() {
            return Ok(Operand::L(val));
        }

        if value.starts_with("0x") {
            if let Ok(val) = u64::from_str_radix(value.trim_start_matches("0x"), 16) {
                return Ok(Operand::N(val));
            }
        }

        Err(format!("Invalid operand: {}", value))
    }
}

impl Operand {
    pub fn get_register(&self) -> Result<Register, String> {
        match self {
            Operand::R(register) => Ok(*register),
            _ => Err(format!("Register not valid: {:?}", self)),
        }
    }
}

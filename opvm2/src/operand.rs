use extism_pdk::{FromBytes, Json, ToBytes};
use serde::{Deserialize, Serialize};

use crate::register::Register;

// operands have different types. for now we only have registers.
// we will add more types in the future.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToBytes, FromBytes)]
#[encoding(Json)]
pub enum Operand {
    None, // no operand
    Register(Register),
    Number(u64),
    Label(String),
}

impl TryFrom<String> for Operand {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // start trying from.
        if let Ok(val) = value.parse::<u64>() {
            return Ok(Operand::Number(val));
        }

        if let Ok(val) = value.clone().try_into() {
            return Ok(Operand::Register(val));
        }

        if let Ok(val) = value.clone().try_into() {
            return Ok(Operand::Label(val));
        }

        if value.starts_with("0x") {
            if let Ok(val) = u64::from_str_radix(value.trim_start_matches("0x"), 16) {
                return Ok(Operand::Number(val));
            }
        }

        Err(format!("Invalid operand: {}", value))
    }
}

impl Operand {
    pub fn get_register(&self) -> Result<Register, String> {
        match self {
            Operand::Register(register) => Ok(*register),
            _ => Err(format!("Register not valid: {:?}", self)),
        }
    }

    pub fn get_number(&self) -> Result<u64, String> {
        match self {
            Operand::Number(number) => Ok(*number),
            _ => Err(format!("Number not valid: {:?}", self)),
        }
    }
}

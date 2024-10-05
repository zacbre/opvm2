use extism_pdk::{FromBytes, Json, ToBytes};
use serde::{Deserialize, Serialize};

use crate::{parser::program::LabelValue, register::Register};

// operands have different types. for now we only have registers.
// we will add more types in the future.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToBytes, FromBytes)]
#[encoding(Json)]
pub enum Operand {
    None,               // no operand
    Register(Register), // todo: this can't just be any number mapped, because what if it's just a regular address? need some way to denote "hey this is a register, not an address."
    Number(usize),
    Label(LabelValue), // todo: this will need to be interpreted as an address at runtime, need to map into memory and store this value later.
    Offset(Offset),    // this will get confusing, maybe split this up
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToBytes, FromBytes)]
#[encoding(Json)]
pub struct Offset {
    pub lhs_operand: String,
    pub operator: Option<String>,
    pub rhs_operand: Option<String>,
}

impl TryFrom<String> for Operand {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.starts_with("[") && value.ends_with("]") {
            // let value = value.trim_start_matches("[").trim_end_matches("]");
            // // perhaps we parse this instead with the parser?
            // let mut parts = value.split("+");
            // let operand = parts.next().unwrap().to_string();
            // let offset = parts.next().map(|x| x.parse::<usize>().unwrap());
            // return Ok(Operand::Offset(Offset { operand, offset }));
        }

        // start trying from.
        if let Ok(val) = value.parse::<usize>() {
            return Ok(Operand::Number(val));
        }

        if let Ok(val) = value.clone().try_into() {
            return Ok(Operand::Register(val));
        }

        if let Ok(val) = value.clone().try_into() {
            return Ok(Operand::Label(LabelValue::Literal(val)));
        }

        if value.starts_with("0x") {
            if let Ok(val) = usize::from_str_radix(value.trim_start_matches("0x"), 16) {
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

    pub fn get_number(&self) -> Result<usize, String> {
        match self {
            Operand::Number(number) => Ok(*number),
            _ => Err(format!("Number not valid: {:?}", self)),
        }
    }

    pub fn operand_type(&self) -> u8 {
        match self {
            Operand::None => 0,
            Operand::Register(_) => 1,
            Operand::Number(_) => 2,
            Operand::Label(_) => 3,
            Operand::Offset(_) => 4,
        }
    }

    pub fn encode(&self, literal_map: &std::collections::BTreeMap<String, usize>) -> usize {
        match self {
            Operand::Register(register) => register.encode() as usize,
            Operand::Number(number) => *number as usize,
            Operand::Label(label) => match label {
                LabelValue::Literal(literal) => {
                    if let Some(res) = literal_map.get(literal) {
                        return *res;
                    }
                    0usize
                }
                LabelValue::Address(address) => *address,
            },
            _ => 0usize,
        }
    }

    pub fn decode(operand_type: u8, operand: u32) -> Operand {
        match operand_type {
            1 => Operand::Register(Register::decode(operand)),
            2 => Operand::Number(operand as usize),
            3 => Operand::Label(LabelValue::Address(operand as usize)),
            _ => Operand::None,
        }
    }

    /// Returns `true` if the operand is [`Offset`].
    ///
    /// [`Offset`]: Operand::Offset
    #[must_use]
    pub fn is_offset(&self) -> bool {
        matches!(self, Self::Offset(..))
    }
}

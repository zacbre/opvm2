use std::collections::BTreeMap;

use extism_pdk::{FromBytes, Json, ToBytes};
use serde::{Deserialize, Serialize};

use crate::{instruction::Instruction, lexer::token::Token, operand::Operand};

#[derive(Serialize, Deserialize, Debug, PartialEq, FromBytes, ToBytes, Clone)]
#[encoding(Json)]
pub enum LabelValue {
    Address(u64),
    Literal(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, FromBytes, ToBytes, Clone)]
#[encoding(Json)]
pub struct Labels {
    pub list: BTreeMap<String, LabelValue>,
}

impl Labels {
    pub fn new() -> Self {
        Self {
            list: Default::default(),
        }
    }
}

impl From<Vec<(String, LabelValue)>> for Labels {
    fn from(vec: Vec<(String, LabelValue)>) -> Self {
        Self {
            list: vec.into_iter().collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToBytes, FromBytes)]
#[encoding(Json)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub labels: Labels,
}

impl Program {
    pub fn from(input: &str) -> Self {
        let tokens = crate::lexer::lex_input(input);
        match tokens {
            Ok(tokens) => Self::new(tokens),
            Err(e) => panic!("Error: {}", e),
        }
    }

    pub fn new(tokens: Vec<Vec<Token>>) -> Self {
        let result = Self::tokens_to_program(tokens);
        match result {
            Ok(program) => program,
            Err(e) => panic!("Error: {}", e),
        }
    }

    fn tokens_to_program(tokens: Vec<Vec<Token>>) -> Result<Self, String> {
        let mut instructions = Vec::new();
        let mut labels: Labels = Labels {
            list: Default::default(),
        };
        //let mut directives = Vec::new();
        for token_list in tokens {
            for token in token_list {
                match token {
                    Token::Label(l) => {
                        // mark the location at which the label is located at, use instructions.len()
                        labels
                            .list
                            .insert(l, LabelValue::Address(instructions.len() as u64));
                    }
                    Token::LabelWithLiteral(l) => {
                        // see if we can parse the l.value as a number
                        if let Ok(val) = l.value.parse::<u64>() {
                            labels.list.insert(l.name, LabelValue::Address(val));
                            continue;
                        }

                        labels.list.insert(l.name, LabelValue::Literal(l.value));
                    }
                    Token::Directive(_) => todo!(),
                    Token::Expression(e) => {
                        let lhs = match e.lhs {
                            Some(lhs) => lhs.try_into()?,
                            None => Operand::None,
                        };
                        let rhs = match e.rhs {
                            Some(rhs) => rhs.try_into()?,
                            None => Operand::None,
                        };
                        let instruction = Instruction::new(e.opcode.into(), lhs, rhs);
                        instructions.push(instruction);
                    }
                    _ => (),
                }
            }
        }
        Ok(Program {
            instructions,
            labels,
        })
    }

    pub fn empty() -> Program {
        Program {
            instructions: vec![],
            labels: Labels::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Token;
    use super::*;
    use crate::lexer::token::{Expression, LabelWithLiteral};
    use crate::opcode::Opcode;
    use crate::operand::Operand;
    use crate::register::Register;

    #[test]
    fn can_convert_tokens_to_instructions() {
        assert_eq!(
            Program::new(vec![
                vec![Token::Expression(Expression {
                    opcode: "mov".to_string(),
                    lhs: Some("ra".to_string()),
                    rhs: Some("1".to_string()),
                })],
                vec![Token::Expression(Expression {
                    opcode: "add".to_string(),
                    lhs: Some("ra".to_string()),
                    rhs: Some("rb".to_string()),
                })],
            ]),
            Program {
                instructions: vec![
                    Instruction::new(
                        Opcode::Mov,
                        Operand::Register(Register::Ra),
                        Operand::Number(1)
                    ),
                    Instruction::new(
                        Opcode::Add,
                        Operand::Register(Register::Ra),
                        Operand::Register(Register::Rb)
                    ),
                ],
                labels: Labels::new(),
            }
        )
    }

    #[test]
    fn can_parse_labels() {
        assert_eq!(
            Program::new(vec![
                vec![Token::Label("start".to_string())],
                vec![Token::Expression(Expression {
                    opcode: "mov".to_string(),
                    lhs: Some("ra".to_string()),
                    rhs: Some("1".to_string()),
                })],
                vec![Token::Expression(Expression {
                    opcode: "add".to_string(),
                    lhs: Some("ra".to_string()),
                    rhs: Some("rb".to_string()),
                })],
                vec![Token::Label("end".to_string())],
            ]),
            Program {
                instructions: vec![
                    Instruction::new(
                        Opcode::Mov,
                        Operand::Register(Register::Ra),
                        Operand::Number(1)
                    ),
                    Instruction::new(
                        Opcode::Add,
                        Operand::Register(Register::Ra),
                        Operand::Register(Register::Rb)
                    ),
                ],
                labels: Labels::from(vec![
                    ("start".to_string(), LabelValue::Address(0)),
                    ("end".to_string(), LabelValue::Address(2))
                ])
            }
        )
    }

    #[test]
    fn can_parse_labels_with_literals() {
        assert_eq!(
            Program::new(vec![
                vec![Token::LabelWithLiteral(LabelWithLiteral {
                    name: "custom".to_string(),
                    value: "10".to_string(),
                })],
                vec![Token::LabelWithLiteral(LabelWithLiteral {
                    name: "custom_str".to_string(),
                    value: "this is my custom value".to_string(),
                })],
                vec![Token::Expression(Expression {
                    opcode: "mov".to_string(),
                    lhs: Some("ra".to_string()),
                    rhs: Some("1".to_string()),
                })],
                vec![Token::Expression(Expression {
                    opcode: "add".to_string(),
                    lhs: Some("ra".to_string()),
                    rhs: Some("rb".to_string()),
                })],
                vec![Token::Label("end".to_string())],
            ]),
            Program {
                instructions: vec![
                    Instruction::new(
                        Opcode::Mov,
                        Operand::Register(Register::Ra),
                        Operand::Number(1)
                    ),
                    Instruction::new(
                        Opcode::Add,
                        Operand::Register(Register::Ra),
                        Operand::Register(Register::Rb)
                    ),
                ],
                labels: Labels::from(vec![
                    ("custom".to_string(), LabelValue::Address(10)),
                    (
                        "custom_str".to_string(),
                        LabelValue::Literal("this is my custom value".to_string())
                    ),
                    ("end".to_string(), LabelValue::Address(2))
                ])
            }
        )
    }
}

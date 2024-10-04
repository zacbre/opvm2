use std::collections::BTreeMap;

use extism_pdk::{FromBytes, Json, ToBytes};
use serde::{Deserialize, Serialize};

use crate::{
    instruction::Instruction,
    lexer::token::{SideType, Token},
    operand::{Offset, Operand},
};

#[derive(Serialize, Deserialize, Debug, PartialEq, FromBytes, ToBytes, Clone)]
#[encoding(Json)]
pub enum LabelValue {
    Address(usize),
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
    pub plugins: Vec<Vec<u8>>,
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
        // make an instruction, then convert said instruction into bytecode.
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
                            .insert(l, LabelValue::Address(instructions.len()));
                    }
                    Token::LabelWithLiteral(l) => {
                        // see if we can parse the l.value as a number
                        if let Ok(val) = l.value.parse::<usize>() {
                            labels.list.insert(l.name, LabelValue::Address(val));
                            continue;
                        }

                        // todo, get rid of this "literal" space and make it an address always after mapping.
                        labels.list.insert(l.name, LabelValue::Literal(l.value));
                    }
                    Token::Directive(_) => todo!(),
                    Token::Expression(e) => {
                        let lhs = Self::parse_side_type(e.lhs)?;
                        let rhs = Self::parse_side_type(e.rhs)?;
                        let instruction = Instruction::new(e.opcode.into(), lhs, rhs);
                        instructions.push(instruction);
                    } // will require multiple passes if the labels are not defined in order (above the expression it's used in.)
                    _ => (),
                }
            }
        }
        Ok(Program {
            instructions,
            labels,
            plugins: vec![],
        })
    }

    fn parse_side_type(s: SideType) -> Result<Operand, String> {
        Ok(match s {
            SideType::Normal(rhs) => rhs.try_into()?,
            SideType::Offset(offset) => Operand::Offset(Offset {
                lhs_operand: offset.lhs,
                operator: offset.operator,
                rhs_operand: offset.rhs,
            }),
            SideType::None => Operand::None,
        })
    }

    pub fn empty() -> Program {
        Program {
            instructions: vec![],
            labels: Labels::new(),
            plugins: vec![],
        }
    }
}

#[cfg(test)]
mod test {
    use super::Token;
    use super::*;
    use crate::lexer::token::{Expression, ExpressionOffset, LabelWithLiteral};
    use crate::opcode::Opcode;
    use crate::operand::Operand;
    use crate::register::Register;

    #[test]
    fn can_convert_tokens_to_instructions() {
        assert_eq!(
            Program::new(vec![
                vec![Token::Expression(Expression {
                    opcode: "mov".to_string(),
                    lhs: SideType::Normal("ra".to_string()),
                    rhs: SideType::Normal("1".to_string()),
                })],
                vec![Token::Expression(Expression {
                    opcode: "add".to_string(),
                    lhs: SideType::Normal("ra".to_string()),
                    rhs: SideType::Normal("rb".to_string()),
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
                plugins: vec![]
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
                    lhs: SideType::Normal("ra".to_string()),
                    rhs: SideType::Normal("1".to_string()),
                })],
                vec![Token::Expression(Expression {
                    opcode: "add".to_string(),
                    lhs: SideType::Normal("ra".to_string()),
                    rhs: SideType::Normal("rb".to_string()),
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
                ]),
                plugins: vec![]
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
                    lhs: SideType::Normal("ra".to_string()),
                    rhs: SideType::Normal("1".to_string()),
                })],
                vec![Token::Expression(Expression {
                    opcode: "add".to_string(),
                    lhs: SideType::Normal("ra".to_string()),
                    rhs: SideType::Normal("rb".to_string()),
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
                ]),
                plugins: vec![]
            }
        )
    }

    #[test]
    fn can_parse_expressions_with_offsets() {
        assert_eq!(
            Program::new(vec![vec![Token::Expression(Expression {
                opcode: "mov".to_string(),
                lhs: SideType::Normal("ra".to_string()),
                rhs: SideType::Offset(ExpressionOffset {
                    lhs: "rb".to_string(),
                    operator: Some("+".to_string()),
                    rhs: Some("1".to_string()),
                }),
            })],]),
            Program {
                instructions: vec![Instruction::new(
                    Opcode::Mov,
                    Operand::Register(Register::Ra),
                    Operand::Offset(Offset {
                        lhs_operand: "rb".to_string(),
                        operator: Some("+".to_string()),
                        rhs_operand: Some("1".to_string()),
                    })
                ),],
                labels: Labels::new(),
                plugins: vec![]
            }
        );

        assert_eq!(
            Program::new(vec![vec![Token::Expression(Expression {
                opcode: "mov".to_string(),
                lhs: SideType::Normal("ra".to_string()),
                rhs: SideType::Offset(ExpressionOffset {
                    lhs: "rb".to_string(),
                    operator: None,
                    rhs: None,
                }),
            })],]),
            Program {
                instructions: vec![Instruction::new(
                    Opcode::Mov,
                    Operand::Register(Register::Ra),
                    Operand::Offset(Offset {
                        lhs_operand: "rb".to_string(),
                        operator: None,
                        rhs_operand: None,
                    })
                ),],
                labels: Labels::new(),
                plugins: vec![]
            }
        );
    }
}

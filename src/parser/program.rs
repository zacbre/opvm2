use std::collections::HashMap;

use crate::{instruction::Instruction, lexer::token::Token};

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub labels: HashMap<String, usize>,
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
        let mut labels: HashMap<String, usize> = HashMap::new();
        //let mut directives = Vec::new();
        for token_list in tokens {
            for token in token_list {
                match token {
                    Token::Label(l) => {
                        // mark the location at which the label is located at, use instructions.len()
                        labels.insert(l, instructions.len());
                    }
                    Token::Directive(d) => todo!(),
                    Token::Expression(e) => {
                        let instruction =
                            Instruction::new(e.opcode.into(), e.lhs.try_into()?, e.rhs.try_into()?);
                        instructions.push(instruction);
                    }
                    Token::LExpression(e) => {
                        let instruction = Instruction::new_l(e.opcode.into(), e.lhs.try_into()?);
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

    pub(crate) fn empty() -> Program {
        Program {
            instructions: vec![],
            labels: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Token;
    use super::*;
    use crate::lexer::token::Expression;
    use crate::opcode::Opcode;
    use crate::operand::Operand;
    use crate::register::Register;

    #[test]
    fn can_convert_tokens_to_instructions() {
        assert_eq!(
            Program::new(vec![
                vec![Token::Expression(Expression {
                    opcode: "mov".to_string(),
                    lhs: "ra".to_string(),
                    rhs: "1".to_string(),
                })],
                vec![Token::Expression(Expression {
                    opcode: "add".to_string(),
                    lhs: "ra".to_string(),
                    rhs: "rb".to_string(),
                })],
            ]),
            Program {
                instructions: vec![
                    Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(1)),
                    Instruction::new(
                        Opcode::Add,
                        Operand::R(Register::Ra),
                        Operand::R(Register::Rb)
                    ),
                ],
                labels: Default::default(),
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
                    lhs: "ra".to_string(),
                    rhs: "1".to_string(),
                })],
                vec![Token::Expression(Expression {
                    opcode: "add".to_string(),
                    lhs: "ra".to_string(),
                    rhs: "rb".to_string(),
                })],
                vec![Token::Label("end".to_string())],
            ]),
            Program {
                instructions: vec![
                    Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(1)),
                    Instruction::new(
                        Opcode::Add,
                        Operand::R(Register::Ra),
                        Operand::R(Register::Rb)
                    ),
                ],
                labels: vec![("start".to_string(), 0), ("end".to_string(), 2)]
                    .into_iter()
                    .collect(),
            }
        )
    }
}

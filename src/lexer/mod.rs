pub mod token;

use self::token::{Expression, Token, TokenType};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    combinator::opt,
    sequence::{delimited, preceded, terminated},
    AsChar, IResult,
};

pub(crate) fn lex_input(i: &str) -> Result<Vec<Vec<Token>>, String> {
    // line by line lexing
    let mut tokens: Vec<Vec<Token>> = vec![];
    let mut errors: Vec<String> = vec![];
    for (line_num, line) in i.lines().enumerate() {
        let result = lex_line(line);
        match result {
            Ok((_, line_tokens)) => {
                if line_tokens.len() > 0 {
                    tokens.push(line_tokens);
                }
            }
            Err(e) => {
                errors.push(format!("Line {}: {}", line_num, e));
            }
        }
    }
    if errors.len() > 0 {
        Err(errors.join("\n"))
    } else {
        Ok(tokens)
    }
}

fn build_token<'a>(
    res: IResult<&'a str, &'a str>,
    token_type: TokenType,
) -> IResult<&'a str, Token> {
    if let Ok((i, token)) = res {
        let token = match token_type {
            TokenType::Comment => Token::Comment(token.to_string()),
            TokenType::Label => Token::Label(token.to_string()),
            TokenType::Directive => Token::Directive(token.to_string()),
        };
        Ok((i, token))
    } else {
        Err(res.unwrap_err())
    }
}

fn lex_line(i: &str) -> IResult<&str, Vec<Token>> {
    // in each line, we expect several things to attempt to match up, so we keep trying to match the input
    // until there isn't any left?
    let mut og = i;
    let mut tokens: Vec<Token> = vec![];
    loop {
        if og.trim().len() == 0 {
            break;
        }

        let res = build_token(comment(og), TokenType::Comment)
            .or_else(|_| build_token(label(og), TokenType::Label))
            .or_else(|_| expression(og))
            .or_else(|_| build_token(directive(og), TokenType::Directive));
        if let Ok((out, tok)) = res {
            tokens.push(tok);
            og = out;
            println!("output: {:?}", og);
        } else {
            break;
        }
    }

    Ok((og, tokens))
}

fn label(i: &str) -> IResult<&str, &str> {
    delimited(
        preceded(opt(whitespace), tag("_")),
        take_while(|c| c == '_' || AsChar::is_alphanum(c)),
        tag(":"),
    )(i)
}

fn comment(i: &str) -> IResult<&str, &str> {
    preceded(
        preceded(opt(whitespace), tag(";")),
        preceded(whitespace, take_while(|c| c != '\n')),
    )(i)
}

fn literal(i: &str) -> IResult<&str, &str> {
    alt((literal_single_quote, literal_double_quote))(i)
}

fn literal_single_quote(i: &str) -> IResult<&str, &str> {
    preceded(
        opt(whitespace),
        delimited(tag("'"), take_while(|c| c != '\''), tag("'")),
    )(i)
}

fn literal_double_quote(i: &str) -> IResult<&str, &str> {
    preceded(
        opt(whitespace),
        delimited(tag("\""), take_while(|c| c != '"'), tag("\"")),
    )(i)
}

fn take_until_whitespace(i: &str) -> IResult<&str, &str> {
    take_while1(|c| (c as char).is_alphanumeric())(i)
}

fn expression(i: &str) -> IResult<&str, Token> {
    // this expression needs an opcode, and then operands, potentially registers or literals.
    let (i, opcode) = preceded(
        opt(whitespace),
        terminated(take_until_whitespace, opt(whitespace)),
    )(i)?;

    if i.trim().len() == 0 {
        return Ok((i, Token::Expression(Expression {
            opcode: opcode.to_string(),
            lhs: None,
            rhs: None,
        })));
    }

    let (i, token) = if let Ok((i, lhs)) = preceded(
        opt(whitespace),
        terminated(
            alt((literal, take_until_whitespace)),
            preceded(opt(whitespace), tag(",")),
        ),
    )(i)
    {
        // get RHS
        let (i, rhs) = preceded(opt(whitespace), alt((literal, take_until_whitespace)))(i)?;
        (
            i,
            Token::Expression(Expression {
                opcode: opcode.to_string(),
                lhs: Some(lhs.to_string()),
                rhs: Some(rhs.to_string()),
            }),
        )
    } else {
        let (i, lhs) = preceded(opt(whitespace), alt((literal, take_until_whitespace)))(i)?;
        (
            i,
            Token::Expression(Expression {
                opcode: opcode.to_string(),
                lhs: Some(lhs.to_string()),
                rhs: None,
            }),
        )
    };

    Ok((i, token))
}

fn whitespace(i: &str) -> IResult<&str, &str> {
    take_while(|c| c == ' ')(i)
}

fn directive(i: &str) -> IResult<&str, &str> {
    preceded(
        opt(whitespace),
        preceded(tag("section ."), take_while(|c| c != ' ')),
    )(i)
}

#[cfg(test)]
mod test {
    use crate::lexer::token::{Expression, Token};

    #[test]
    fn can_parse_comments() {
        assert_eq!(
            super::comment("; this is a comment"),
            Ok(("", "this is a comment"))
        );
        assert_eq!(
            super::comment("      ;this is a comment"),
            Ok(("", "this is a comment"))
        );
        assert_eq!(
            super::comment("      ;           this is a comment"),
            Ok(("", "this is a comment"))
        );
    }

    #[test]
    fn can_parse_literal() {
        assert_eq!(
            super::literal("'this is a literal'"),
            Ok(("", "this is a literal"))
        );
        assert_eq!(
            super::literal("  'this is a literal'"),
            Ok(("", "this is a literal"))
        );
        assert_eq!(
            super::literal("  \"this is a literal\""),
            Ok(("", "this is a literal"))
        );
        assert_eq!(
            super::literal("  'this is a literal'  "),
            Ok(("  ", "this is a literal"))
        );
        assert_eq!(
            super::literal("  'this is a literal\""),
            Err(nom::Err::Error(nom::error::Error {
                input: "'this is a literal\"",
                code: nom::error::ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn can_parse_whitespace() {
        assert_eq!(super::whitespace(" "), Ok(("", " ")));
        assert_eq!(super::whitespace("  "), Ok(("", "  ")));
        assert_eq!(super::whitespace("   "), Ok(("", "   ")));
    }

    #[test]
    fn can_parse_label() {
        assert_eq!(
            super::label("_supercrazy label:"),
            Err(nom::Err::Error(nom::error::Error {
                input: " label:",
                code: nom::error::ErrorKind::Tag
            }))
        );
        assert_eq!(
            super::label("_super,label:"),
            Err(nom::Err::Error(nom::error::Error {
                input: ",label:",
                code: nom::error::ErrorKind::Tag
            }))
        );
        assert_eq!(super::label("_label:"), Ok(("", "label")));
        assert_eq!(super::label("_label: "), Ok((" ", "label")));
        assert_eq!(super::label("_label: "), Ok((" ", "label")));
        assert_eq!(super::label("_label_1: "), Ok((" ", "label_1")));
    }

    #[test]
    fn can_parse_directive() {
        assert_eq!(super::directive("section .data"), Ok(("", "data")));
        assert_eq!(super::directive("section .data "), Ok((" ", "data")));
        assert_eq!(super::directive("section .data  "), Ok(("  ", "data")));
        assert_eq!(super::directive(" section .data  "), Ok(("  ", "data")));
        assert_eq!(super::directive("  section .data  "), Ok(("  ", "data")));
        assert_eq!(super::directive("  section .data  "), Ok(("  ", "data")));
    }

    #[test]
    fn can_parse_expression() {
        assert_eq!(
            super::expression("mov rax, rdx"),
            Ok((
                "",
                super::Token::Expression(super::Expression {
                    opcode: "mov".to_string(),
                    lhs: Some("rax".to_string()),
                    rhs: Some("rdx".to_string())
                })
            ))
        );

        assert_eq!(
            super::expression("mov rax, 'a'"),
            Ok((
                "",
                super::Token::Expression(super::Expression {
                    opcode: "mov".to_string(),
                    lhs: Some("rax".to_string()),
                    rhs: Some("a".to_string())
                })
            ))
        );

        assert_eq!(
            super::expression("print rax"),
            Ok((
                "",
                super::Token::Expression(super::Expression {
                    opcode: "print".to_string(),
                    lhs: Some("rax".to_string()),
                    rhs: None
                })
            ))
        );

        assert_eq!(
            super::expression("ret  "),
            Ok((
                "",
                super::Token::Expression(super::Expression {
                    opcode: "ret".to_string(),
                    lhs: None,
                    rhs: None
                })
            ))
        );
    }

    #[test]
    fn can_parse_line() {
        assert_eq!(
            super::lex_line("_label: mov r0, 'a' ; comment"),
            Ok((
                "",
                vec![
                    super::Token::Label("label".to_string()),
                    super::Token::Expression(super::Expression {
                        opcode: "mov".to_string(),
                        lhs: Some("r0".to_string()),
                        rhs: Some("a".to_string())
                    }),
                    super::Token::Comment("comment".to_string())
                ]
            ))
        );
    }

    #[test]
    fn can_parse_input() {
        assert_eq!(
            super::lex_input(
                r"
            section .data
                _label: mov rax, 0
                        mov rcx, 'a'
                        print rcx
        "
            ),
            Ok(vec![
                vec![Token::Directive("data".to_string())],
                vec![
                    Token::Label("label".to_string()),
                    Token::Expression(Expression {
                        opcode: "mov".to_string(),
                        lhs: Some("rax".to_string()),
                        rhs: Some("0".to_string())
                    })
                ],
                vec![Token::Expression(Expression {
                    opcode: "mov".to_string(),
                    lhs: Some("rcx".to_string()),
                    rhs: Some("a".to_string())
                })],
                vec![Token::Expression(Expression {
                    opcode: "print".to_string(),
                    lhs: Some("rcx".to_string()),
                    rhs: None
                })]
            ])
        );
    }
}

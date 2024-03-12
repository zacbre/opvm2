#[derive(Debug, PartialEq)]
pub enum Token {
    Comment(String),
    Label(String),
    Directive(String),
    Expression(Expression),
    LExpression(LExpression),
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Comment,
    Label,
    Directive,
}

#[derive(Debug, PartialEq)]
pub struct LExpression {
    pub opcode: String,
    pub lhs: String,
}

#[derive(Debug, PartialEq)]
pub struct Expression {
    pub opcode: String,
    pub lhs: String,
    pub rhs: String,
}

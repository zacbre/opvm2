#[derive(Debug, PartialEq)]
pub enum Token {
    Comment(String),
    Label(String),
    LabelWithLiteral(LabelWithLiteral),
    Directive(String),
    Expression(Expression),
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Comment,
    Label,
    Directive,
}

#[derive(Debug, PartialEq)]
pub struct LabelWithLiteral {
    pub name: String,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Expression {
    pub opcode: String,
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

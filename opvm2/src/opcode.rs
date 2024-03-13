use std::fmt::Display;

use extism_pdk::{FromBytes, Json};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Clone, FromBytes)]
#[encoding(Json)]
pub enum Opcode {
    Mov,
    /* Arithmetic */
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Xor,
    Inc,
    Dec,
    /* Stack */
    Push,
    Pop,
    Dup,
    /* Program Flow */
    Test,
    Jmp,
    Je,
    Jne,
    Jle,
    Jge,
    Jl,
    Jg,
    Call,
    Return,
    /* Various */
    Assert,
    Print,
    Nop,
    Hlt,
    Plugin(String),
}

impl From<String> for Opcode {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "mov" => Self::Mov,
            "add" => Self::Add,
            "sub" => Self::Sub,
            "mul" => Self::Mul,
            "div" => Self::Div,
            "mod" => Self::Mod,
            "xor" => Self::Xor,
            "inc" => Self::Inc,
            "dec" => Self::Dec,
            "push" => Self::Push,
            "pop" => Self::Pop,
            "dup" => Self::Dup,
            "test" => Self::Test,
            "jmp" => Self::Jmp,
            "je" => Self::Je,
            "jne" => Self::Jne,
            "jle" => Self::Jle,
            "jge" => Self::Jge,
            "jl" => Self::Jl,
            "jg" => Self::Jg,
            "call" => Self::Call,
            "ret" => Self::Return,
            "assert" => Self::Assert,
            "print" => Self::Print,
            "nop" => Self::Nop,
            "hlt" => Self::Hlt,
            _ => Self::Plugin(value),
            // todo: add ability to extend with extism? check if the instruction exists.
        }
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mov => write!(f, "mov"),
            Self::Add => write!(f, "add"),
            Self::Sub => write!(f, "sub"),
            Self::Mul => write!(f, "mul"),
            Self::Div => write!(f, "div"),
            Self::Mod => write!(f, "mod"),
            Self::Xor => write!(f, "xor"),
            Self::Inc => write!(f, "inc"),
            Self::Dec => write!(f, "dec"),
            Self::Push => write!(f, "push"),
            Self::Pop => write!(f, "pop"),
            Self::Dup => write!(f, "dup"),
            Self::Test => write!(f, "test"),
            Self::Jmp => write!(f, "jmp"),
            Self::Je => write!(f, "je"),
            Self::Jne => write!(f, "jne"),
            Self::Jle => write!(f, "jle"),
            Self::Jge => write!(f, "jge"),
            Self::Jl => write!(f, "jl"),
            Self::Jg => write!(f, "jg"),
            Self::Call => write!(f, "call"),
            Self::Return => write!(f, "ret"),
            Self::Assert => write!(f, "assert"),
            Self::Print => write!(f, "print"),
            Self::Nop => write!(f, "nop"),
            Self::Hlt => write!(f, "hlt"),
            Self::Plugin(s) => write!(f, "{}", s),
        }
    }
}
// todo: add from/into for u8 for binary compiling

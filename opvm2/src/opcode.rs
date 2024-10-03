use std::fmt::Display;

use extism_pdk::{FromBytes, Json, ToBytes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToBytes, FromBytes)]
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
    Sleep,
    Nop,
    Halt,
    Plugin(String),
}

impl Opcode {
    pub fn is_plugin(&self) -> bool {
        match &self {
            Self::Plugin(_) => true,
            _ => false,
        }
    }

    pub fn get_plugin_opcode(&self) -> String {
        match &self {
            Self::Plugin(name) => name.clone(),
            _ => String::new(),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Mov => 0,
            Self::Add => 1,
            Self::Sub => 2,
            Self::Mul => 3,
            Self::Div => 4,
            Self::Mod => 5,
            Self::Xor => 6,
            Self::Inc => 7,
            Self::Dec => 8,
            Self::Push => 9,
            Self::Pop => 10,
            Self::Dup => 11,
            Self::Test => 12,
            Self::Jmp => 13,
            Self::Je => 14,
            Self::Jne => 15,
            Self::Jle => 16,
            Self::Jge => 17,
            Self::Jl => 18,
            Self::Jg => 19,
            Self::Call => 20,
            Self::Return => 21,
            Self::Assert => 22,
            Self::Print => 23,
            Self::Sleep => 24,
            Self::Nop => 25,
            Self::Halt => 26,
            _ => 25, // anything else is nop rn
                     //Self::Plugin(_) => 27, again, going to have to do something about this.
        }
    }

    pub fn from_u8(value: u8) -> Opcode {
        match value {
            0 => Opcode::Mov,
            1 => Opcode::Add,
            2 => Opcode::Sub,
            3 => Opcode::Mul,
            4 => Opcode::Div,
            5 => Opcode::Mod,
            6 => Opcode::Xor,
            7 => Opcode::Inc,
            8 => Opcode::Dec,
            9 => Opcode::Push,
            10 => Opcode::Pop,
            11 => Opcode::Dup,
            12 => Opcode::Test,
            13 => Opcode::Jmp,
            14 => Opcode::Je,
            15 => Opcode::Jne,
            16 => Opcode::Jle,
            17 => Opcode::Jge,
            18 => Opcode::Jl,
            19 => Opcode::Jg,
            20 => Opcode::Call,
            21 => Opcode::Return,
            22 => Opcode::Assert,
            23 => Opcode::Print,
            24 => Opcode::Sleep,
            25 => Opcode::Nop,
            26 => Opcode::Halt,
            _ => Opcode::Nop,
            //27 => Self::Plugin(str), // todo: this can't exist in the current state because the plugin will need to have a string passed.
            // idea: in the future, probably map the plugin to a literal and pass it as part of the opcode, idk how yet.
        }
    }
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
            "sleep" => Self::Sleep,
            "nop" => Self::Nop,
            "hlt" => Self::Halt,
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
            Self::Sleep => write!(f, "sleep"),
            Self::Nop => write!(f, "nop"),
            Self::Halt => write!(f, "hlt"),
            Self::Plugin(s) => write!(f, "{}", s),
        }
    }
}
// todo: add from/into for u8 for binary compiling

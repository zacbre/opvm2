#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Opcode {
    Mov,
    /* Arithmetic */
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Xor,
    /* Stack */
    Push,
    Pop,
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
    Print,
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
            "push" => Self::Push,
            "pop" => Self::Pop,
            "test" => Self::Test,
            "jmp" => Self::Jmp,
            "je" => Self::Je,
            "jne" => Self::Jne,
            "jle" => Self::Jle,
            "jge" => Self::Jge,
            "jl" => Self::Jl,
            "jg" => Self::Jg,
            "call" => Self::Call,
            "return" => Self::Return,
            "print" => Self::Print,
            _ => panic!("Invalid opcode"),
            // todo: add ability to extend with extism?
        }
    }
}

// todo: add from/into for u8 for binary compiling

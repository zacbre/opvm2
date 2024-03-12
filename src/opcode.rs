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
            _ => panic!("Invalid opcode"),
            // todo: add ability to extend with extism?
        }
    }
}

// todo: add from/into for u8 for binary compiling
use extism_pdk::*;
use opvm2::{
    parser::program::Program,
    plugin_interface::{
        all_registers, execute, get_input, get_labels, print, quit, set_register,
        OnInstructionValue,
    },
    register::Register,
};
use var::get_memory;

#[plugin_fn]
pub fn name() -> FnResult<String> {
    Ok("debugger".to_string())
}

// todo: handle_error should be implemented
// todo: also implement handle_post_instruction

static mut BREAKPOINTS: Vec<usize> = vec![];
static mut STEP: bool = true;
static mut FIRST_RUN: bool = false;

#[plugin_fn]
pub fn handle_instruction(Json(ins): Json<OnInstructionValue>) -> FnResult<Option<u64>> {
    if !unsafe { FIRST_RUN } {
        unsafe { FIRST_RUN = true };
        unsafe { print(format!("Welcome to the debugger!\n"))? };
        unsafe { print(format!("Type 'help' for a list of commands\n"))? };
        //unsafe { print(format!("Program length: {:?}", ))}
    }
    // let us use a basic interpreter, powered by the VM itself? whoa
    loop {
        unsafe {
            if BREAKPOINTS.contains(&ins.pc) {
                print(format!("Breakpoint hit at {}!\n", ins.pc))?;
                STEP = true;
            } else if !STEP {
                return Ok(None);
            }
        }
        unsafe { print(format!("{:#02x}: ", ins.pc))? };

        let input = unsafe { get_input()? };
        let input = input.trim_end_matches('\n');
        if input.starts_with("bp") {
            let bp = input.split_whitespace().nth(1).unwrap();
            let pc: usize = bp.parse::<usize>()?;
            unsafe { BREAKPOINTS.push(pc) };
            unsafe { print(format!("Added breakpoint at {}\n", bp))? }
            continue;
        }
        if input.starts_with("dbp") {
            let bp = input.split_whitespace().nth(1).unwrap();
            let pc: usize = bp.parse::<usize>()?;
            let index = unsafe { BREAKPOINTS.iter().position(|&x| x == pc) };
            if index.is_none() {
                unsafe { print(format!("Breakpoint not found!\n"))? };
                continue;
            }
            unsafe { BREAKPOINTS.remove(index.unwrap()) };
            unsafe { print(format!("Removed breakpoint at {}\n", bp))? }
            continue;
        }
        if input.starts_with("set") {
            let mut input = input.split_whitespace();
            input.next(); // set
            let p_register = input.next().unwrap().to_string();
            let result = Register::try_from(p_register.clone());
            if result.is_err() {
                unsafe { print(format!("Invalid register `{}`!\n", p_register))? }
                continue;
            }
            let register = result.unwrap();
            let value = input.next().unwrap().parse().unwrap();
            unsafe { set_register(register, value)? };
            unsafe { print(format!("Set register {} to {}!\n", p_register, value))? }
            continue;
        }
        if input.starts_with("ins") {
            let program = Program::from(&input[3..]);
            let instruction = program.instructions[0].clone();

            unsafe {
                execute(instruction)?;
            }
            unsafe { STEP = true };
            return Ok(None);
        }
        if input.starts_with("in") {
            let program = Program::from(&input[3..]);
            let instruction = program.instructions[0].clone();
            let pc = *unsafe { all_registers()? }.check_pc();
            unsafe {
                BREAKPOINTS.push(pc);
            }

            unsafe {
                execute(instruction)?;
            }
            unsafe { STEP = false };
            return Ok(None);
        }
        match input {
            "x" => {
                unsafe { STEP = false };
                return Ok(None);
            }
            "step" | "s" => {
                unsafe { STEP = true };
                return Ok(None);
            }
            "print" | "p" => {
                unsafe { print(format!("Instruction: {:?}\n", ins))? };
            }
            "registers" | "r" => {
                unsafe { print(format!("{:?}\n", all_registers()?))? };
            }
            "labels" | "l" => {
                unsafe { print(format!("{:?}\n", get_labels()?))? };
            }
            "quit" | "q" => {
                unsafe { quit()? };
            }
            _ => {
                unsafe { print(format!("Unknown command!\n"))? };
            }
        }
    }
}

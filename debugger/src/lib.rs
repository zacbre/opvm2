use extism_pdk::*;
use opvm2::{
    plugin_interface::{
        all_registers, get_input, get_labels, quit, set_register, OnInstructionValue,
    },
    register::Register,
};

#[plugin_fn]
pub fn name() -> FnResult<String> {
    Ok("debugger".to_string())
}

// todo: handle_error should be implemented
// todo: also implement handle_post_instruction
//
static mut BREAKPOINTS: Vec<u64> = vec![];
static mut STEP: bool = true;

#[plugin_fn]
pub fn handle_instruction(Json(ins): Json<OnInstructionValue>) -> FnResult<Option<u64>> {
    // let us use a basic interpreter, powered by the VM itself? whoa
    loop {
        info!("{:#02x}", ins.pc);
        unsafe {
            if BREAKPOINTS.contains(&ins.pc) {
                info!("Breakpoint hit at {}", ins.pc);
                STEP = true;
            } else if !STEP {
                return Ok(None);
            }
        }

        let input = unsafe { get_input()? };
        let input = input.trim_end_matches('\n');
        if input.starts_with("bp") {
            let bp = input.split_whitespace().nth(1).unwrap();
            let pc: u64 = bp.parse::<u64>()?;
            unsafe { BREAKPOINTS.push(pc) };
            info!("Added breakpoint at {}", bp);
            continue;
        }
        if input.starts_with("dbp") {
            let bp = input.split_whitespace().nth(1).unwrap();
            let pc: u64 = bp.parse::<u64>()?;
            let index = unsafe { BREAKPOINTS.iter().position(|&x| x == pc) };
            if index.is_none() {
                info!("Breakpoint not found!");
                continue;
            }
            unsafe { BREAKPOINTS.remove(index.unwrap()) };
            info!("Removed breakpoint at {}", bp);
            continue;
        }
        if input.starts_with("set") {
            let mut input = input.split_whitespace();
            input.next(); // set
            let p_register = input.next().unwrap().to_string();
            let result = Register::try_from(p_register.clone());
            if result.is_err() {
                info!("Invalid register `{}`!", p_register);
                continue;
            }
            let register = result.unwrap();
            let value = input.next().unwrap().parse().unwrap();
            unsafe { set_register(register, value)? };
            info!("Set register {} to {}!", p_register, value);
            continue;
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
                info!("Instruction: {:?}", ins);
            }
            "registers" | "r" => {
                info!("{:?}", unsafe { all_registers()? });
            }
            "labels" | "l" => {
                info!("{:?}", unsafe { get_labels()? });
            }
            "quit" | "q" => {
                unsafe { quit()? };
            }
            _ => {
                info!("Unknown command!");
            }
        }
    }
}

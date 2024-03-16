use extism_pdk::*;
use opvm2::{
    parser::program::Labels,
    plugin_interface::*,
    register::{Register, Registers},
};
use serde::{Deserialize, Serialize};

#[plugin_fn]
pub fn name() -> FnResult<String> {
    Ok("Test Plugin".to_string())
}

#[plugin_fn]
pub fn get_all_registers_test() -> FnResult<Registers> {
    Ok(unsafe { all_registers() }?)
}

#[plugin_fn]
pub fn get_register_test(register: Register) -> FnResult<u64> {
    Ok(unsafe { get_register(register) }?)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SetRegisterValue {
    pub register: Register,
    pub value: u64,
}

#[plugin_fn]
pub fn set_register_test(Json(value): Json<SetRegisterValue>) -> FnResult<()> {
    Ok(unsafe { set_register(value.register, value.value) }?)
}

#[plugin_fn]
pub fn push_stack_test(value: u64) -> FnResult<()> {
    Ok(unsafe { push_stack(value) }?)
}

#[plugin_fn]
pub fn pop_stack_test() -> FnResult<u64> {
    Ok(unsafe { pop_stack() }?)
}

#[plugin_fn]
pub fn get_all_labels_test() -> FnResult<Labels> {
    Ok(unsafe { get_labels() }?)
}

#[plugin_fn]
pub fn jmp_to_label_test(label: String) -> FnResult<()> {
    Ok(unsafe { jmp_to_label(label) }?)
}

#[plugin_fn]
pub fn handle_life(Json(ins): Json<OnInstructionValue>) -> FnResult<()> {
    let register = ins
        .lhs
        .get_register()
        .map_err(|e| extism_pdk::Error::msg(e.to_string()))?;
    unsafe { set_register(register, 42)? }
    Ok(())
}

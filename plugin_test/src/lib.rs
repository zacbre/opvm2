use extism_pdk::*;
use opvm2::plugin_interface::*;

#[plugin_fn]
pub fn name() -> FnResult<String> {
    Ok("Test Plugin".to_string())
}

#[plugin_fn]
pub fn get_register_test() -> FnResult<u64> {
    let register = unsafe { get_register(opvm2::register::Register::Ra) };
    Ok(register.unwrap())
}

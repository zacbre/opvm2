use extism_pdk::*;
use opvm2::plugin_interface::*;

#[plugin_fn]
pub fn name() -> FnResult<String> {
    Ok("debugger".to_string())
}

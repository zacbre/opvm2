use extism_pdk::*;

#[plugin_fn]
pub fn name() -> FnResult<String> {
    Ok("debugger".to_string())
}

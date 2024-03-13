use crate::register::Register;

use extism_pdk::*;

#[host_fn]
extern "ExtismHost" {
    pub fn get_register(register: Register) -> u64;
}

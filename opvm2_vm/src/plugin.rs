use std::{collections::BTreeMap, ffi::CString, io::Write};

use extism::*;
use opvm2::{
    instruction::Instruction,
    opcode::Opcode,
    parser::program::LabelValue,
    plugin_interface::{Label, Labels, OnInstructionValue},
};

use crate::{machine_context::MachineContext, register::Register};

#[derive(Debug)]
pub struct PluginLoader {
    pub plugins: Vec<Plugin>,
    context: UserData<MachineContext>,
}

// extern "C" fn log(data: *const std::ffi::c_char, _size: Size) {
//     unsafe {
//         let line = CStr::from_ptr(data);
//         println!("ttt: {}", line.to_str().unwrap());
//     }
// }

impl PluginLoader {
    pub fn new(context: UserData<MachineContext>) -> Self {
        unsafe {
            if !cfg!(test) {
                let stdout = CString::new("stdout").unwrap();
                let info = CString::new("info").unwrap();
                sdk::extism_log_file(stdout.as_ptr(), info.as_ptr());
            }

            //extism_log_custom(info.as_ptr());
            //extism_log_drain(log)
        }
        Self {
            plugins: vec![],
            context,
        }
    }

    pub fn execute_plugin_fn(
        &mut self,
        name: String,
        ins: OnInstructionValue,
        is_hook: bool,
        base_address: usize,
    ) -> Result<usize, String> {
        let mut executed_count = 0;
        for plugin in self.plugins.iter_mut() {
            if !plugin.function_exists(&name) {
                continue;
            }

            let addr = plugin
                .call::<&OnInstructionValue, Option<u64>>(&name, &ins)
                .map_err(|e| e.to_string())?;

            let context = self.context.get().map_err(|e| e.to_string()).unwrap();
            let mut context = context.lock().unwrap();
            match addr {
                Some(addr) => context
                    .registers
                    .set_pc(base_address as usize + addr as usize),
                None => {
                    if !is_hook {
                        context.registers.increment_pc()
                    }
                }
            }
            executed_count += 1;
        }
        Ok(executed_count)
    }

    pub fn load_all(&mut self, plugins: &Vec<Vec<u8>>, verbose: bool) -> Result<(), String> {
        for plugin in plugins {
            let manifest = Manifest::new([Wasm::data(plugin.clone())]);
            self.load(manifest, verbose);
        }
        Ok(())
    }

    pub fn load_from_path(&mut self, path: &str, verbose: bool) -> Result<(), String> {
        let manifest = Manifest::new([Wasm::file(path)]);
        self.load(manifest, verbose);
        Ok(())
    }

    pub fn load(&mut self, manifest: Manifest, verbose: bool) {
        let mut plugin = PluginBuilder::new(manifest)
            .with_wasi(true)
            .with_function(
                "all_registers",
                [],
                [PTR],
                self.context.clone(),
                all_registers,
            )
            .with_function(
                "get_register",
                [PTR],
                [PTR],
                self.context.clone(),
                get_register,
            )
            .with_function(
                "set_register",
                [PTR, PTR],
                [],
                self.context.clone(),
                set_register,
            )
            .with_function("push_stack", [PTR], [], self.context.clone(), push_stack)
            .with_function("pop_stack", [], [PTR], self.context.clone(), pop_stack)
            .with_function("get_input", [], [PTR], self.context.clone(), get_input)
            .with_function(
                "jmp_to_label",
                [PTR],
                [],
                self.context.clone(),
                jmp_to_label,
            )
            .with_function("get_labels", [], [PTR], self.context.clone(), get_labels)
            .with_function("quit", [], [], self.context.clone(), quit)
            .with_function("print", [PTR], [], self.context.clone(), print)
            .with_function("execute", [PTR], [], self.context.clone(), execute)
            .build()
            .unwrap();
        if !plugin.function_exists("name") {
            panic!("Plugin does not have a `name` function");
        }
        let name = plugin.call::<(), String>("name", ()).unwrap();
        if verbose {
            println!("Loaded plugin: {}", name);
        }
        self.plugins.push(plugin);
    }
}

host_fn!(pub all_registers(user_data: MachineContext;) -> Result<Registers, String> {
    let context = user_data.get()?;
    let context = context.lock().unwrap();
    Ok(context.registers.clone())
});

host_fn!(pub get_register(user_data: MachineContext; register: Register) -> Result<u64, String> {
    let context = user_data.get()?;
    let context = context.lock().unwrap();
    Ok(context.registers.get(&register) as u64)
});

host_fn!(pub set_register(user_data: MachineContext; register: Register, value: u64) -> Result<()> {
    let context = user_data.get()?;
    let mut context = context.lock().unwrap();
    context.registers.set(&register, value as usize);
    Ok(())
});

host_fn!(pub push_stack(user_data: MachineContext; value: u64) -> Result<()> {
    let context = user_data.get()?;
    let mut context = context.lock().unwrap();
    context.stack.push(value as usize);
    Ok(())
});

host_fn!(pub pop_stack(user_data: MachineContext;) -> Result<u64, String> {
    let context = user_data.get()?;
    let mut context = context.lock().unwrap();
    Ok(context.stack.pop().unwrap() as u64)
});

host_fn!(pub get_input(user_data: MachineContext;) -> Result<String, String> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input)
});

host_fn!(pub jmp_to_label(user_data: MachineContext; label: String) -> Result<(), String> {
    let context = user_data.get()?;
    let context = context.lock().unwrap();
    //
    // let address = context.current_program.labels.list.get(&label);
    // if address.is_none() {
    //     return Err(extism::Error::msg(format!("Label '{}' does not exist!", &label)))
    // }
    // // todo: check if this is an address or a label with a literal/value?
    // match *address.unwrap() {
    //     LabelValue::Address(address) => context.registers.set_pc(address),
    //     _ => return Err(extism::Error::msg(format!("Label '{}' does not contain an address!", &label)))
    // };
    Ok(())
});

host_fn!(pub get_labels(user_data: MachineContext;) -> Result<Labels, String> {
    let context = user_data.get()?;
    let mut context = context.lock().unwrap();
    // scan for labels in memory?
    let base = context.base_address;
    let mut labels = Labels{list: vec![]};
    let mut i = 0;
    while i < base {
        let val = context.memory.get_literal(i);
        let converted = String::from_utf8(val.to_vec()).unwrap();
        labels.list.push(Label { name: converted, address: i });
        i = i + val.len() + 1;
    }
    // ok so there are no more labels anymore...we need to get this from memory?
    Ok(labels)
});

host_fn!(pub quit(user_data: MachineContext;) -> Result<(), String> {
    std::process::exit(0);
    #[allow(unreachable_code)]
    {return Ok(());}
});

host_fn!(pub print(user_data: MachineContext; data: String) -> Result<(), String> {
   print!("{}", data);
   std::io::stdout().flush()?;
   Ok(())
});

host_fn!(pub execute(user_data: MachineContext; data: Instruction) -> Result<(), String> {
    let context = user_data.get()?;
    let mut context = context.lock().unwrap();
    let empty_map: BTreeMap<String, usize> = BTreeMap::new();
    let base = context.base_address;
    let current_end = context.memory.address();
    let current_pc = *context.registers.check_pc();

    // insert a jump at the beginning and at the end
    // the beginning ensures we never hit this routine again.
    // the end ensures we return to the original location.
    let jmp_address = context.memory.push(&Instruction::get_u8_array(Instruction::new_l(Opcode::Jmp, opvm2::operand::Operand::Label(LabelValue::Address((current_end - base) + 48))).encode(&empty_map)), false);
    context.memory.push(&Instruction::get_u8_array(data.encode(&empty_map)), false);
    context.memory.push(&Instruction::get_u8_array(Instruction::new_l(Opcode::Jmp, opvm2::operand::Operand::Label(LabelValue::Address(current_pc - base))).encode(&empty_map)), false);
    context.registers.set_pc(jmp_address);
    Ok(())
});

#[cfg(test)]
mod test {
    use extism::convert::Json;
    use opvm2::{
        instruction::Instruction,
        parser::program::{LabelValue, Program},
        register::Registers,
    };
    use serde::{Deserialize, Serialize};

    use crate::{plugin::Labels, register::Register, vm::Vm};

    fn load_plugins(plugins: Vec<String>) -> Result<Vec<Vec<u8>>, String> {
        let mut loaded = Vec::new();
        for plugin in plugins {
            let content = std::fs::read(plugin).map_err(|e| e.to_string())?;
            loaded.push(content);
        }
        Ok(loaded)
    }

    fn run_program(program: Program) -> Result<Vm, String> {
        let mut vm = load_vm();
        let mut plugins = load_plugins(vec![
            "../target/wasm32-unknown-unknown/debug/plugin_test.wasm".to_string(),
        ])?;
        let mut program = program.clone();
        program.plugins.append(&mut plugins);
        vm.run_program(program)?;
        Ok(vm)
    }

    fn read_registers(vm: &Vm) -> Registers {
        let context = vm.context.get().map_err(|e| e.to_string()).unwrap();
        let context = context.lock().unwrap();
        context.registers.clone()
    }

    fn load_vm() -> Vm {
        let context = super::MachineContext::new();
        let vm = crate::vm::Vm::new(context);
        vm
    }

    #[test]
    fn can_load_plugin() -> Result<(), String> {
        let mut vm = run_program(Program::from(""))?;
        assert_eq!(vm.plugin.plugins.len(), 1);
        assert!(vm.plugin.plugins[0].function_exists("name"));
        let name = vm.plugin.plugins[0].call::<(), &str>("name", ()).unwrap();
        assert_eq!(name, "Test Plugin");
        Ok(())
    }

    #[test]
    fn can_give_plugins_access_to_vm() -> Result<(), String> {
        let mut vm = run_program(Program::from("mov ra, 5"))?;
        let result = vm.plugin.plugins[0].call::<Register, u64>("get_register_test", Register::Ra);
        assert_eq!(result.unwrap(), 5);
        let result = vm.plugin.plugins[0].call::<Register, u64>("get_register_test", Register::Rb);
        assert_eq!(result.unwrap(), 0);
        Ok(())
    }

    #[test]
    fn can_use_plugin_to_set_register() -> Result<(), extism::Error> {
        let mut vm = run_program(Program::from("")).unwrap();

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        pub struct SetRegisterValue {
            pub register: Register,
            pub value: usize,
        }

        vm.plugin.plugins[0].call::<Json<SetRegisterValue>, ()>(
            "set_register_test",
            Json(SetRegisterValue {
                register: Register::Ra,
                value: 5,
            }),
        )?;
        vm.run_program(Program::from("mov rb, ra")).unwrap();
        assert_eq!(read_registers(&vm).ra, 5);
        assert_eq!(read_registers(&vm).rb, 5);
        Ok(())
    }

    #[test]
    fn can_use_plugin_to_push_and_pop_stack() -> Result<(), extism::Error> {
        let mut vm = run_program(Program::from("")).unwrap();
        vm.plugin.plugins[0].call::<u64, ()>("push_stack_test", 5)?;
        assert_eq!(vm.context.get()?.lock().unwrap().stack.peek(), Some(&5));
        let result = vm.plugin.plugins[0].call::<(), u64>("pop_stack_test", ());
        assert_eq!(result.unwrap(), 5);
        Ok(())
    }

    #[test]
    fn can_get_all_registers() -> Result<(), extism::Error> {
        let mut vm = run_program(Program::from("mov ra, 10\nmov rb, 3")).unwrap();
        let registers = vm.plugin.plugins[0].call::<(), Registers>("get_all_registers_test", ())?;
        assert_eq!(registers.ra, 10);
        assert_eq!(registers.rb, 3);
        assert_eq!(*registers.check_pc(), 32);
        Ok(())
    }

    #[test]
    fn can_get_labels_and_jmp_to_one() -> Result<(), extism::Error> {
        let vm = run_program(Program::from(
            r"
            jmp _label
            mov ra, 10
            _label: mov rb, 3
            mov rc, 5",
        ))
        .map_err(|e| extism::Error::msg(e.to_string()))?;
        {
            let context = vm.context.get()?;
            let mut context = context.lock().unwrap();
            // try to pull the first instruction from memory, and decode the label?
            let ins = context.memory.get_instruction(0);
            let ins_decoded = Instruction::decode(ins);
            match ins_decoded.lhs {
                opvm2::operand::Operand::Label(LabelValue::Address(address)) => {
                    assert_eq!(address, 32);
                }
                _ => panic!("Expected label address!"),
            }
        }
        Ok(())
    }

    #[test]
    fn can_re_labels() -> Result<(), extism::Error> {
        let mut vm = run_program(Program::from(
            r"
            label1: 'test'
            label2: 'testa'
            label3: 'tesb'
            l1: mov ra, rb
            end:",
        ))
        .map_err(|e| extism::Error::msg(e.to_string()))?;
        let labels = vm.plugin.plugins[0].call::<(), Labels>("get_all_labels_test", ())?;
        assert_eq!(labels.list.len(), 3);
        assert_eq!(labels.list[0].name, "test");
        assert_eq!(labels.list[0].address, 0);
        assert_eq!(labels.list[1].name, "testa");
        assert_eq!(labels.list[1].address, 5);
        assert_eq!(labels.list[2].name, "tesb");
        assert_eq!(labels.list[2].address, 11);
        Ok(())
    }

    #[test]
    fn can_handle_custom_opcode() {
        let vm = run_program(Program::from("life ra")).unwrap();
        let registers = read_registers(&vm);
        assert_eq!(registers.ra, 42);
    }
}

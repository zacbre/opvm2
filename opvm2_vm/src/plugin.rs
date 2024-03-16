use extism::*;

use crate::{register::Register, store::Store};

#[derive(Debug)]
pub struct PluginLoader {
    pub plugins: Vec<Plugin>,
    store: UserData<Store>,
}

impl PluginLoader {
    pub fn new(store: UserData<Store>) -> Self {
        Self {
            plugins: vec![],
            store,
        }
    }

    pub fn load(&mut self, path: &str) {
        let manifest = Manifest::new([Wasm::file(path)]);
        let plugin = PluginBuilder::new(manifest)
            .with_wasi(true)
            .with_function(
                "all_registers",
                [],
                [PTR],
                self.store.clone(),
                all_registers,
            )
            .with_function(
                "get_register",
                [PTR],
                [PTR],
                self.store.clone(),
                get_register,
            )
            .with_function(
                "set_register",
                [PTR, PTR],
                [],
                self.store.clone(),
                set_register,
            )
            .with_function("push_stack", [PTR], [], self.store.clone(), push_stack)
            .with_function("pop_stack", [], [PTR], self.store.clone(), pop_stack)
            .with_function("get_input", [], [PTR], self.store.clone(), get_input)
            .with_function("jmp_to_label", [PTR], [], self.store.clone(), jmp_to_label)
            .with_function("get_labels", [], [PTR], self.store.clone(), get_labels)
            .build()
            .unwrap();
        //let plugin = Plugin::new(&manifest, [], true).unwrap();
        self.plugins.push(plugin);
    }
}

host_fn!(pub all_registers(user_data: Store;) -> Result<Registers, String> {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    Ok(store.registers.clone())
});

host_fn!(pub get_register(user_data: Store; register: Register) -> Result<u64, String> {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    Ok(store.registers.get(&register))
});

host_fn!(pub set_register(user_data: Store; register: Register, value: u64) -> Result<()> {
    let store = user_data.get()?;
    let mut store = store.lock().unwrap();
    store.registers.set(&register, value);
    Ok(())
});

host_fn!(pub push_stack(user_data: Store; value: u64) -> Result<()> {
    let store = user_data.get()?;
    let mut store = store.lock().unwrap();
    store.stack.push(value);
    Ok(())
});

host_fn!(pub pop_stack(user_data: Store;) -> Result<u64, String> {
    let store = user_data.get()?;
    let mut store = store.lock().unwrap();
    Ok(store.stack.pop().unwrap())
});

host_fn!(pub get_input(user_data: Store;) -> Result<String, String> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input)
});

host_fn!(pub jmp_to_label(user_data: Store; label: String) -> Result<(), String> {
    let store = user_data.get()?;
    let mut store = store.lock().unwrap();
    let address = *store.current_program.labels.list.get(&label).expect("Label not found");
    store.registers.set_pc(address);
    Ok(())
});

host_fn!(pub get_labels(user_data: Store;) -> Result<Labels, String> {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    Ok(store.current_program.labels.clone())
});

#[cfg(test)]
mod test {
    use extism::convert::Json;
    use opvm2::{
        parser::program::{Labels, Program},
        register::Registers,
    };
    use serde::{Deserialize, Serialize};

    use crate::{register::Register, vm::Vm};

    fn read_registers(vm: &Vm) -> Registers {
        let store = vm.store.get().map_err(|e| e.to_string()).unwrap();
        let store = store.lock().unwrap();
        store.registers.clone()
    }

    fn load_vm() -> Vm {
        let store = super::Store::new();
        let mut vm = crate::vm::Vm::new(store);
        vm.plugin
            .load("../target/wasm32-unknown-unknown/debug/plugin_test.wasm");

        vm
    }

    #[test]
    fn can_load_plugin() {
        let mut vm = load_vm();
        assert_eq!(vm.plugin.plugins.len(), 1);
        assert!(vm.plugin.plugins[0].function_exists("name"));
        let name = vm.plugin.plugins[0].call::<(), &str>("name", ()).unwrap();
        assert_eq!(name, "Test Plugin");
    }

    #[test]
    fn can_give_plugins_access_to_vm() {
        let mut vm = load_vm();
        let program = Program::from(
            r"
            mov ra, 5
        ",
        );
        vm.run(program).unwrap();
        vm.plugin
            .load("../target/wasm32-unknown-unknown/debug/plugin_test.wasm");
        let result = vm.plugin.plugins[0].call::<Register, u64>("get_register_test", Register::Ra);
        assert_eq!(result.unwrap(), 5);
        let result = vm.plugin.plugins[0].call::<Register, u64>("get_register_test", Register::Rb);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn can_use_plugin_to_set_register() -> Result<(), extism::Error> {
        let mut vm = load_vm();

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        pub struct SetRegisterValue {
            pub register: Register,
            pub value: u64,
        }

        vm.plugin.plugins[0].call::<Json<SetRegisterValue>, ()>(
            "set_register_test",
            Json(SetRegisterValue {
                register: Register::Ra,
                value: 5,
            }),
        )?;
        vm.run(Program::from("mov rb, ra")).unwrap();
        assert_eq!(read_registers(&vm).ra, 5);
        assert_eq!(read_registers(&vm).rb, 5);
        Ok(())
    }

    #[test]
    fn can_use_plugin_to_push_and_pop_stack() -> Result<(), extism::Error> {
        let mut vm = load_vm();
        vm.plugin.plugins[0].call::<u64, ()>("push_stack_test", 5)?;
        assert_eq!(vm.store.get()?.lock().unwrap().stack.peek(), Some(&5));
        let result = vm.plugin.plugins[0].call::<(), u64>("pop_stack_test", ());
        assert_eq!(result.unwrap(), 5);
        Ok(())
    }

    #[test]
    fn can_get_all_registers() -> Result<(), extism::Error> {
        let mut vm = load_vm();
        vm.run(Program::from("mov ra, 10\nmov rb, 3")).unwrap();
        let registers = vm.plugin.plugins[0].call::<(), Registers>("get_all_registers_test", ())?;
        assert_eq!(registers.ra, 10);
        assert_eq!(registers.rb, 3);
        assert_eq!(*registers.check_pc(), 2);
        Ok(())
    }

    #[test]
    fn can_get_labels_and_jmp_to_one() -> Result<(), extism::Error> {
        let mut vm = load_vm();
        vm.run(Program::from(
            r"
            jmp label
            mov ra, 10
            _label: mov rb, 3
            mov rc, 5",
        ))
        .map_err(|e| extism::Error::msg(e.to_string()))?;
        {
            let store = vm.store.get()?;
            let store = store.lock().unwrap();
            assert_eq!(store.current_program.labels.list.len(), 1);
            assert_eq!(store.current_program.labels.list.get("label"), Some(&2));
        }
        let labels = vm.plugin.plugins[0].call::<(), Labels>("get_all_labels_test", ())?;
        assert_eq!(labels.list.get("label"), Some(&2));
        vm.plugin.plugins[0].call::<&str, ()>("jmp_to_label_test", "label")?;
        let registers = read_registers(&vm);
        assert_eq!(registers.ra, 0);
        assert_eq!(registers.rb, 3);
        assert_eq!(registers.rc, 5);
        assert_eq!(*registers.check_pc(), 2);
        Ok(())
    }

    #[test]
    fn can_handle_custom_opcode() {
        let mut vm = load_vm();
        vm.run(Program::from(
            r"
            life ra
            ",
        ))
        .unwrap();
        let registers = read_registers(&vm);
        assert_eq!(registers.ra, 42);
    }
}

use extism::*;

use crate::{register::Register, store::Store, vm::Vm};

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
                "get_register",
                [PTR],
                [PTR],
                self.store.clone(),
                get_register,
            )
            .build()
            .unwrap();
        //let plugin = Plugin::new(&manifest, [], true).unwrap();
        self.plugins.push(plugin);
    }
}

host_fn!(pub get_register(user_data: Store; register: Register) -> Result<u64, String> {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    Ok(store.registers.get(&register))
});

#[cfg(test)]
mod test {
    use crate::register::Register;

    #[test]
    fn can_load_plugin() {
        let store = super::Store::new();
        let mut vm = super::Vm::new(store);
        vm.plugin
            .load("../target/wasm32-unknown-unknown/debug/plugin_test.wasm");
        assert_eq!(vm.plugin.plugins.len(), 1);
        assert!(vm.plugin.plugins[0].function_exists("name"));
        let name = vm.plugin.plugins[0].call::<(), &str>("name", ()).unwrap();
        assert_eq!(name, "Test Plugin");
    }

    #[test]
    fn can_give_plugins_access_to_vm() {
        let store = super::Store::new();
        let mut vm = super::Vm::new(store);
        {
            let borrowed_store = vm.store.get().unwrap();
            let mut borrowed_store = borrowed_store.lock().unwrap();
            borrowed_store.registers.set(&Register::Ra, 5);
        }
        vm.plugin
            .load("../target/wasm32-unknown-unknown/debug/plugin_test.wasm");
        let result = vm.plugin.plugins[0].call::<Register, u64>("get_register_test", Register::Ra);
        assert_eq!(result.unwrap(), 5);
    }
}

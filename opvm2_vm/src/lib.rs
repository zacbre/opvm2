//pub mod heap;
pub mod machine_context;
pub mod memory;
pub mod plugin;
pub mod vm;

use std::collections::BTreeMap;

use extism::{convert::Json, FromBytes, ToBytes, UserData};
use machine_context::MachineContext;
use memory::Memory;
use opvm2::{opcode::Opcode, parser::program::Program, *};
use parser::program::LabelValue;
use plugin::PluginLoader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, ToBytes, FromBytes, PartialEq, Clone)]
#[encoding(Json)]
pub struct CompiledProgram {
    pub start_address: usize,
    pub memory_address: usize,
    pub program: Vec<u8>,
    pub plugins: Vec<Vec<u8>>,
}

impl CompiledProgram {
    pub fn new(plugins: Vec<Vec<u8>>) -> Self {
        Self {
            start_address: 0,
            plugins,
            program: vec![],
            memory_address: 0,
        }
    }

    pub fn new_e() -> Self {
        Self {
            start_address: 0,
            plugins: vec![],
            program: vec![],
            memory_address: 0,
        }
    }

    pub fn compile(&mut self, program: Program, verbose: bool) -> Result<Vec<u8>, String> {
        let mut loader = PluginLoader::new(UserData::new(MachineContext::new()));
        loader.load_all(&program.plugins, verbose)?;
        let mut err_msg = String::new();
        for ins in program.instructions.iter() {
            match ins.opcode {
                Opcode::Plugin(ref name) => {
                    // if there are no plugins, we can't handle the opcode
                    if loader.plugins.is_empty() {
                        err_msg = format!(
                            "{}No plugins found for handling opcode: `{}`. ",
                            err_msg,
                            name.to_string().to_lowercase()
                        );
                        continue;
                    }
                    let mut found = false;
                    for plugin in loader.plugins.iter_mut() {
                        if plugin
                            .function_exists(format!("handle_{}", name.to_string().to_lowercase()))
                        {
                            found = true;
                            continue;
                        }
                    }
                    if !found {
                        err_msg = format!(
                            "{}No plugin found for handling opcode: `{}`. ",
                            err_msg,
                            name.to_string().to_lowercase()
                        );
                    }
                }
                _ => {}
            }
        }
        self.plugins = program.plugins.clone();
        let (base, memory) = Self::remap(program);
        self.program = memory.raw();
        self.start_address = base;
        self.memory_address = memory.address();
        if !err_msg.is_empty() {
            return Err(err_msg);
        }
        let bytes = (*self).to_bytes().map_err(|e| e.to_string())?;
        Ok(bytes)
    }

    fn remap(program: Program) -> (usize, Memory) {
        let mut memory = Memory::new();
        // loop through program, get all literals/instructions, remap into existing memory space.
        let mut literal_list: BTreeMap<String, usize> = BTreeMap::new();
        for (label, value) in program.labels.list.iter() {
            if literal_list.contains_key(label) {
                continue;
            }

            match value {
                LabelValue::Literal(value) => {
                    // store in memory.
                    let address = memory.push(value.as_bytes(), true);
                    literal_list.insert(label.clone(), address);
                }
                LabelValue::Address(address) => {
                    // these addresses need to be relocated to the new memory space. start_address + (address * 128).
                    literal_list.insert(label.clone(), *address * 16);
                }
            }
        }

        for instruction in program.instructions.iter() {
            match &instruction.opcode {
                Opcode::Plugin(plugin) => {
                    match plugin {
                        opvm2::opcode::PluginValue::None => {}
                        opvm2::opcode::PluginValue::Name(name) => {
                            if literal_list.contains_key(name) {
                                continue;
                            }
                            // store the name of the plugin in memory.
                            let address = memory.push(name.as_bytes(), true);
                            literal_list.insert(name.clone(), address);
                        }
                        opvm2::opcode::PluginValue::Address(_) => {}
                    }
                }
                _ => {}
            }
        }

        let start_address = memory.address();
        for instruction in program.instructions.iter() {
            let encoded = instruction.encode(&literal_list);
            memory.push(&Self::get_u8_array(encoded), false);
        }

        (start_address, memory)
    }

    fn get_u8_array(data: u128) -> Vec<u8> {
        let mut bytes = Vec::new();
        for i in 0..16 {
            let byte = (data >> (i * 8)) as u8;
            bytes.push(byte);
        }
        bytes
    }
}

impl From<Vec<u8>> for CompiledProgram {
    fn from(input: Vec<u8>) -> Self {
        let program = CompiledProgram::from_bytes(&input);
        match program {
            Ok(program) => program,
            Err(e) => panic!("Error: {}", e),
        }
    }
}

impl From<Program> for CompiledProgram {
    fn from(program: Program) -> Self {
        let mut compiled = CompiledProgram::new_e();
        let output = compiled.compile(program, false);
        match output {
            Ok(_) => compiled,
            Err(e) => panic!("Error: {}", e),
        }
    }
}

impl From<&str> for CompiledProgram {
    fn from(str: &str) -> Self {
        let program = Program::from(str);
        let mut compiled = CompiledProgram::new_e();
        let output = compiled.compile(program, false);
        match output {
            Ok(_) => compiled,
            Err(e) => panic!("Error: {}", e),
        }
    }
}

#[cfg(test)]
mod test {
    use opvm2::register::Registers;

    use crate::{parser::program::Program, vm::Vm};

    fn read_registers(vm: &Vm) -> Registers {
        let context = vm.context.get().unwrap();
        let context = context.lock().unwrap();
        context.registers.clone()
    }

    #[test]
    fn can_add_two_numbers() -> Result<(), String> {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov ra, 1
            mov rb, 2
            add ra, rb
        ",
        );
        vm.run_program(program)?;
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Ra), 3);
        Ok(())
    }

    #[test]
    fn can_inc() {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov ra, 1
            inc ra
        ",
        );
        vm.run_program(program).unwrap();
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Ra), 2);
    }

    #[test]
    fn can_dec() {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov ra, 1
            dec ra
        ",
        );
        vm.run_program(program).unwrap();
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Ra), 0);
    }

    #[test]
    fn can_xor_two_numbers() {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov ra, 3
            mov rb, 5
            xor ra, rb
        ",
        );
        vm.run_program(program).unwrap();
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Ra), 6);
    }

    #[test]
    fn can_push_and_pop() {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov ra, 1
            push ra
            pop rb
        ",
        );
        vm.run_program(program).unwrap();
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rb), 1);
    }

    #[test]
    fn can_dup_stack() {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov ra, 5
            push ra
            dup
            pop rb
            pop rc
        ",
        );
        vm.run_program(program).unwrap();
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rb), 5);
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rc), 5);
    }

    #[test]
    fn can_jump() -> Result<(), String> {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov r0, 64
            jmp r0
            mov ra, 2   ; this should be skipped
            mov rb, 3   ; this should be skipped
            mov rc, 5
        ",
        );
        vm.run_program(program)?;
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Ra), 0);
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rb), 0);
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rc), 5);
        Ok(())
    }

    #[test]
    fn can_jump_with_labels() -> Result<(), String> {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            jmp _start
            mov ra, 2   ; this should be skipped
            mov rb, 3   ; this should be skipped
            _start: mov rc, 5
        ",
        );
        vm.run_program(program)?;
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Ra), 0);
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rb), 0);
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rc), 5);
        Ok(())
    }

    #[test]
    fn can_jump_when_less_than() {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov ra, 2
            mov rb, 3
            test ra, rb
            jl _less_than
            mov rc, 5
            jmp _exit
            _less_than: mov rc, 10
            _exit:
        ",
        );
        vm.run_program(program).unwrap();
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rc), 10);
    }

    #[test]
    fn can_jump_when_less_than_or_equal() {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov ra, 3
            mov rb, 3
            test ra, rb
            jle _less_than
            mov rc, 5
            jmp _exit
            _less_than: mov rc, 10
            _exit:
        ",
        );
        vm.run_program(program).unwrap();
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rc), 10);
    }

    #[test]
    fn can_jump_when_greater_than() {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov ra, 4
            mov rb, 3
            test ra, rb
            jg _greater_than
            mov rc, 5
            jmp _exit
            _greater_than: mov rc, 10
            _exit:
        ",
        );
        vm.run_program(program).unwrap();
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rc), 10);
    }

    #[test]
    fn can_jump_when_greater_than_or_equal() {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov ra, 3
            mov rb, 3
            test ra, rb
            jge _greater_than
            mov rc, 5
            jmp _exit
            _greater_than: mov rc, 10
            _exit:
        ",
        );
        vm.run_program(program).unwrap();
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rc), 10);
    }

    #[test]
    fn can_jump_when_equal() {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov ra, 3
            mov rb, 3
            test ra, rb
            je _equal
            mov rc, 5
            jmp _exit
            _equal: mov rc, 10
            _exit:
        ",
        );
        vm.run_program(program).unwrap();
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rc), 10);
    }

    #[test]
    fn can_jump_when_not_equal() {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            mov ra, 3
            mov rb, 4
            test ra, rb
            jne _not_equal
            mov rc, 5
            jmp _exit
            _not_equal: mov rc, 10
            _exit:
        ",
        );
        vm.run_program(program).unwrap();
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rc), 10);
    }

    #[test]
    fn can_call_and_return() {
        let mut vm = super::vm::Vm::new_e();
        let program = Program::from(
            r"
            call add
            jmp exit
            add: mov ra, 3
            mov rb, 4
            add ra, rb
            ret
            mov rc, 5
            exit:
            mov rd, 6
        ",
        );
        vm.run_program(program).unwrap();
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Ra), 7);
        assert_eq!(read_registers(&vm).get(&crate::register::Register::Rd), 6);
        assert_ne!(read_registers(&vm).get(&crate::register::Register::Rc), 5);
    }
}

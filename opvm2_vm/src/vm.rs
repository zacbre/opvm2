use std::sync::MutexGuard;

use extism::{convert::Json, FromBytes, UserData};
use opvm2::register::Register;
use serde::Deserialize;

use crate::{
    opcode::Opcode, operand::Operand, parser::program::Program, plugin::PluginLoader,
    register::Registers, stack::Stack, store::Store,
};

#[derive(Debug)]
pub struct Vm {
    // todo: this stuff should live in a machine context? that plugins have access to? then the VM should also have access to
    // at the same level as the plugin
    pub store: UserData<Store>,
    pub plugin: PluginLoader,
}

impl Vm {
    pub fn new(store: Store) -> Vm {
        let store = UserData::new(store);

        Vm {
            store: store.clone(),
            plugin: PluginLoader::new(store),
        }
    }

    pub fn new_e() -> Vm {
        let store = UserData::new(Store::new());

        Vm {
            store: store.clone(),
            plugin: PluginLoader::new(store),
        }
    }

    // pub fn load_plugins(&self) -> PluginLoader {
    //     //let mut plugin_loader = PluginLoader::new();
    //     //plugin_loader.load("../target/wasm32-unknown-unknown/debug/debugger.wasm");
    //     //plugin_loader
    // }
    //

    // pub fn read_registers(&self) -> Registers {
    //     let store = self.store.get().map_err(|e| e.to_string()).unwrap();
    //     let store = store.lock().unwrap();
    //     store.registers.clone()
    // }

    // pub fn pop_stack(&mut self) -> Result<u64, String> {
    //     let store = self.store.get().map_err(|e| e.to_string()).unwrap();
    //     let mut store = store.lock().unwrap();
    //     store.stack.pop().ok_or("Stack is empty".to_string())
    // }

    // pub fn push_stack(&mut self, value: u64) {
    //     let store = self.store.get().map_err(|e| e.to_string()).unwrap();
    //     let mut store = store.lock().unwrap();
    //     store.stack.push(value);
    // }

    // pub fn peek_stack(&mut self) -> Result<u64, String> {
    //     let store = self.store.get().map_err(|e| e.to_string()).unwrap();
    //     let store = store.lock().unwrap();
    //     let value = store.stack.peek().unwrap();
    //     Ok(*value)
    // }

    // pub fn set_register(&mut self, register: &Register, value: u64) -> Result<(), String> {
    //     let store = self.store.get().map_err(|e| e.to_string()).unwrap();
    //     let mut store = store.lock().unwrap();
    //     store.registers.set(register, value);
    //     Ok(())
    // }

    // pub fn get_current_program(&self) -> Result<Program, String> {
    //     let store = self.store.get().map_err(|e| e.to_string()).unwrap();
    //     let store = store.lock().unwrap();
    //     Ok(store.current_program.clone())
    // }
    //

    pub fn check_pc(&self) -> u64 {
        let store = self.store.get().map_err(|e| e.to_string()).unwrap();
        let store = store.lock().unwrap();
        *store.registers.clone().check_pc()
    }

    pub fn run(&mut self, program: Program) -> Result<(), String> {
        // every time we need access to the store, we need to unlock it.
        {
            let store = self.store.get().map_err(|e| e.to_string()).unwrap();
            let mut store = store.lock().unwrap();
            store.current_program = program.clone();
        }
        //let plugin_loader = self.load_plugins();
        while (self.check_pc() as usize) < program.instructions.len() {
            let item = &program.instructions[self.check_pc() as usize];
            // for plugin in &plugin_loader.plugins {
            //     if plugin.function_exists(&item.opcode.to_string()) {
            //         let lhs = self.get_value(&item.lhs);
            //         let rhs = self.get_value(&item.rhs);
            //         let result =
            //             plugin.call::<(u64, u64), u64>(&item.opcode.to_string(), (lhs, rhs));
            //         self.registers
            //             .set(&item.lhs.get_register()?, result.unwrap());
            //         self.read_registers().increment_pc();
            //         continue;
            //     }
            // }
            let store = self.store.get().map_err(|e| e.to_string()).unwrap();
            let mut store = store.lock().unwrap();

            match item.opcode.clone() {
                Opcode::Mov => {
                    let lhs = item.lhs.get_register()?;
                    let rhs_value = self.get_value(&mut store, &item.rhs)?;
                    store.registers.set(&lhs, rhs_value);
                }
                Opcode::Add => self.math(&mut store, &item.lhs, &item.rhs, item.opcode.clone())?,
                Opcode::Sub => self.math(&mut store, &item.lhs, &item.rhs, item.opcode.clone())?,
                Opcode::Mul => self.math(&mut store, &item.lhs, &item.rhs, item.opcode.clone())?,
                Opcode::Div => self.math(&mut store, &item.lhs, &item.rhs, item.opcode.clone())?,
                Opcode::Mod => self.math(&mut store, &item.lhs, &item.rhs, item.opcode.clone())?,
                Opcode::Xor => self.math(&mut store, &item.lhs, &item.rhs, item.opcode.clone())?,
                Opcode::Inc => {
                    let lhs = self.get_value(&mut store, &item.lhs)?;

                    store.registers.set(&item.lhs.get_register()?, lhs + 1);
                }
                Opcode::Dec => {
                    let lhs = self.get_value(&mut store, &item.lhs)?;

                    store.registers.set(&item.lhs.get_register()?, lhs - 1);
                }
                Opcode::Print => {
                    let lhs_value = self.get_value(&mut store, &item.lhs)?;
                    println!("{}", lhs_value);
                }
                Opcode::Push => {
                    let lhs_value = self.get_value(&mut store, &item.lhs)?;
                    store.stack.push(lhs_value);
                }
                Opcode::Pop => {
                    let lhs = item.lhs.get_register()?;
                    let value = store.stack.pop().unwrap();
                    store.registers.set(&lhs, value);
                }
                Opcode::Dup => {
                    let peeked = *store.stack.peek().unwrap();
                    store.stack.push(peeked);
                }
                Opcode::Test => self.test(&mut store, &item.lhs, &item.rhs),
                Opcode::Jmp => {
                    let lhs_value = self.get_value(&mut store, &item.lhs)?;
                    store.registers.set_pc(lhs_value);
                    continue;
                }
                Opcode::Je => {
                    if store.registers.check_equals_flag() {
                        let lhs_value = self.get_value(&mut store, &item.lhs)?;
                        store.registers.set_pc(lhs_value);
                        continue;
                    }
                }
                Opcode::Jne => {
                    if !store.registers.check_equals_flag() {
                        let lhs_value = self.get_value(&mut store, &item.lhs)?;
                        store.registers.set_pc(lhs_value);
                        continue;
                    }
                }
                Opcode::Jle => {
                    if store.registers.check_equals_flag() || store.registers.check_less_than_flag()
                    {
                        let lhs_value = self.get_value(&mut store, &item.lhs)?;
                        store.registers.set_pc(lhs_value);
                        continue;
                    }
                }
                Opcode::Jge => {
                    if store.registers.check_equals_flag()
                        || store.registers.check_greater_than_flag()
                    {
                        let lhs_value = self.get_value(&mut store, &item.lhs)?;
                        store.registers.set_pc(lhs_value);
                        continue;
                    }
                }
                Opcode::Jl => {
                    if store.registers.check_less_than_flag() {
                        let lhs_value = self.get_value(&mut store, &item.lhs)?;
                        store.registers.set_pc(lhs_value);
                        continue;
                    }
                }
                Opcode::Jg => {
                    if store.registers.check_greater_than_flag() {
                        let lhs_value = self.get_value(&mut store, &item.lhs)?;
                        store.registers.set_pc(lhs_value);
                        continue;
                    }
                }
                Opcode::Call => {
                    let lhs_value = self.get_value(&mut store, &item.lhs)?;
                    let call_stack_pointer = store.registers.check_pc() + 1;
                    store.call_stack.push(call_stack_pointer);
                    store.registers.set_pc(lhs_value);
                    continue;
                }
                Opcode::Return => {
                    let return_address = store.call_stack.pop().unwrap();
                    store.registers.set_pc(return_address);
                    continue;
                }
                Opcode::Assert => {
                    self.test(&mut store, &item.lhs, &item.rhs);
                    if !store.registers.check_equals_flag() {
                        return Err(format!(
                            "Assertion failed at ins {}.",
                            store.registers.check_pc()
                        ));
                    }
                    store.registers.reset_flags();
                }
                Opcode::Nop => {}
                Opcode::Hlt => {
                    return Ok(());
                }
                Opcode::Plugin(s) => {
                    // error, this wasn't handled?
                    return Err(format!("Plugin for '{}' not found", s));
                }
            }
            store.registers.increment_pc();
        }
        Ok(())
    }

    fn test(&mut self, store: &mut MutexGuard<Store>, lhs: &Operand, rhs: &Operand) {
        let lhs_value = self.get_value(store, lhs);
        let rhs_value = self.get_value(store, rhs);
        store.registers.reset_flags();
        if lhs_value == rhs_value {
            store.registers.set_equals_flag(true);
        }
        if lhs_value < rhs_value {
            store.registers.set_less_than_flag(true);
        }
        if lhs_value > rhs_value {
            store.registers.set_greater_than_flag(true);
        }
    }

    fn get_value(&self, store: &mut MutexGuard<Store>, operand: &Operand) -> Result<u64, String> {
        match operand {
            Operand::N(n) => Ok(*n),
            Operand::R(r) => Ok(store.registers.get(&r)),
            Operand::L(l) => Ok(store
                .current_program
                .labels
                .get(l)
                .unwrap()
                .clone()
                .try_into()
                .unwrap()),
            _ => panic!(
                "{}",
                format!("Invalid operand for operation: {:?}", operand)
            ),
        }
    }

    fn math(
        &mut self,
        store: &mut MutexGuard<Store>,
        lhs: &Operand,
        rhs: &Operand,
        operator: Opcode,
    ) -> Result<(), String> {
        let lhs_value = self.get_value(store, lhs)?;
        let rhs_value = self.get_value(store, rhs)?;
        let value = match operator {
            Opcode::Add => lhs_value + rhs_value,
            Opcode::Sub => lhs_value - rhs_value,
            Opcode::Mul => lhs_value * rhs_value,
            Opcode::Div => lhs_value / rhs_value,
            Opcode::Mod => lhs_value % rhs_value,
            Opcode::Xor => lhs_value ^ rhs_value,
            _ => panic!("Invalid operator for math operation"),
        };

        store.registers.set(&lhs.get_register()?, value);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::instruction::Instruction;
    use crate::opcode::Opcode;
    use crate::operand::Operand;
    use crate::parser::program::Program;
    use crate::register::Register;

    fn run(input: Vec<Instruction>) -> Result<Vm, String> {
        let mut vm = super::Vm::new_e();
        vm.run(Program {
            instructions: input,
            labels: HashMap::new(),
        })?;
        Ok(vm)
    }

    fn run_l(input: Vec<Instruction>, labels: HashMap<String, usize>) -> Result<Vm, String> {
        let mut vm = super::Vm::new_e();
        vm.run(Program {
            instructions: input,
            labels,
        })?;
        Ok(vm)
    }

    fn read_registers(vm: &Vm) -> Registers {
        let store = vm.store.get().map_err(|e| e.to_string()).unwrap();
        let store = store.lock().unwrap();
        store.registers.clone()
    }

    fn pop_stack(vm: &mut Vm) -> Result<u64, String> {
        let store = vm.store.get().map_err(|e| e.to_string()).unwrap();
        let mut store = store.lock().unwrap();
        store.stack.pop().ok_or("Stack is empty".to_string())
    }

    use super::Vm;
    use opvm2::register::Registers;
    use test_case::test_case;

    #[test]
    fn can_run_vm() -> Result<(), String> {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(10)),
            Instruction::new(
                Opcode::Add,
                Operand::R(Register::Ra),
                Operand::R(Register::Rb),
            ),
            Instruction::new(
                Opcode::Add,
                Operand::R(Register::Ra),
                Operand::R(Register::Rc),
            ),
        ];
        let vm = run(input)?;
        assert_eq!(read_registers(&vm).ra, 30);

        Ok(())
    }

    #[test]
    fn can_mov_value_to_register() -> Result<(), String> {
        let input = vec![Instruction::new(
            Opcode::Mov,
            Operand::R(Register::Ra),
            Operand::N(10),
        )];
        let vm = run(input)?;
        assert_eq!(read_registers(&vm).ra, 10);
        Ok(())
    }

    #[test]
    fn can_mov_value_from_register_to_register() {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(
                Opcode::Mov,
                Operand::R(Register::Rb),
                Operand::R(Register::Ra),
            ),
        ];
        let vm = run(input).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 10);
    }

    #[test_case(Opcode::Add, "ra", 1, "rb", 2, 3; "can add rb + ra = 3")]
    #[test_case(Opcode::Sub, "ra", 10, "rb", 5, 5; "can sub rb - ra = 5")]
    #[test_case(Opcode::Mul, "ra", 2, "rb", 3, 6; "can mul rb * ra = 6")]
    #[test_case(Opcode::Div, "ra", 10, "rb", 2, 5; "can div rb / ra = 5")]
    #[test_case(Opcode::Mod, "ra", 10, "rb", 3, 1; "can mod rb % ra = 1")]
    #[test_case(Opcode::Xor, "ra", 10, "rb", 3, 9; "can xor rb ^ ra = 9")]
    fn can_use_math_functions(
        opcode: Opcode,
        lhs: &str,
        lval: u64,
        rhs: &str,
        rval: u64,
        expected: u64,
    ) -> Result<(), String> {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::R(Register::try_from(lhs.to_string()).unwrap()),
                Operand::N(lval),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::R(Register::try_from(rhs.to_string()).unwrap()),
                Operand::N(rval),
            ),
            Instruction::new(
                opcode,
                Operand::R(Register::try_from(lhs.to_string()).unwrap()),
                Operand::R(Register::try_from(rhs.to_string()).unwrap()),
            ),
        ];

        let vm = run(input)?;
        assert_eq!(
            read_registers(&vm).get(&Register::try_from(lhs.to_string()).unwrap()),
            expected
        );

        Ok(())
    }

    #[test_case(Opcode::Add, "ra", 1, 2, 3; "can add 2 + ra = 3")]
    #[test_case(Opcode::Sub, "ra", 10, 5, 5; "can sub 5 - ra = 5")]
    #[test_case(Opcode::Mul, "ra", 2, 3, 6; "can mul 3 * ra = 6")]
    #[test_case(Opcode::Div, "ra", 10, 2, 5; "can div 2 / ra = 5")]
    #[test_case(Opcode::Mod, "ra", 10, 3, 1; "can mod 3 % ra = 1")]
    #[test_case(Opcode::Xor, "ra", 10, 3, 9; "can xor 3 ^ ra = 9")]
    fn can_use_math_functions_with_immediate(
        opcode: Opcode,
        lhs: &str,
        lval: u64,
        rval: u64,
        expected: u64,
    ) -> Result<(), String> {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::R(Register::try_from(lhs.to_string()).unwrap()),
                Operand::N(lval),
            ),
            Instruction::new(
                opcode,
                Operand::R(Register::try_from(lhs.to_string()).unwrap()),
                Operand::N(rval),
            ),
        ];

        let vm = run(input)?;
        assert_eq!(
            read_registers(&vm).get(&Register::try_from(lhs.to_string()).unwrap()),
            expected
        );

        Ok(())
    }

    #[test]
    fn can_push_and_pop() -> Result<(), String> {
        let input = vec![
            Instruction::new_l(Opcode::Push, Operand::N(10)),
            Instruction::new_l(Opcode::Pop, Operand::R(Register::Ra)),
        ];
        let vm = run(input)?;
        assert_eq!(read_registers(&vm).ra, 10);
        Ok(())
    }

    #[test]
    fn can_push_and_pop_multiple() {
        let input = vec![
            Instruction::new_l(Opcode::Push, Operand::N(10)),
            Instruction::new_l(Opcode::Push, Operand::N(20)),
            Instruction::new_l(Opcode::Push, Operand::N(30)),
            Instruction::new_l(Opcode::Pop, Operand::R(Register::Ra)),
            Instruction::new_l(Opcode::Pop, Operand::R(Register::Rb)),
        ];
        let mut vm = run(input).unwrap();
        assert_eq!(read_registers(&vm).ra, 30);
        assert_eq!(read_registers(&vm).rb, 20);
        assert_eq!(pop_stack(&mut vm), Ok(10));
    }

    #[test]
    fn can_jump() -> Result<(), String> {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::R0), Operand::N(4)),
            Instruction::new_l(Opcode::Jmp, Operand::R(Register::R0)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(20)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(30)),
        ];
        let vm = run(input)?;
        assert_eq!(read_registers(&vm).ra, 0);
        assert_eq!(read_registers(&vm).rb, 0);
        assert_eq!(read_registers(&vm).rc, 30);
        Ok(())
    }

    #[test]
    fn can_jump_to_label() {
        let input = vec![
            Instruction::new_l(Opcode::Jmp, Operand::L("start".to_string())),
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(20)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(30)),
        ];
        let labels = vec![("start".to_string(), 3)].into_iter().collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 0);
        assert_eq!(read_registers(&vm).rb, 0);
        assert_eq!(read_registers(&vm).rc, 30);
    }

    #[test]
    fn can_test() {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(20)),
            Instruction::new(
                Opcode::Test,
                Operand::R(Register::Ra),
                Operand::R(Register::Rb),
            ),
            Instruction::new_l(Opcode::Jle, Operand::L("less".to_string())),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rd), Operand::N(1)),
            Instruction::new_l(Opcode::Jmp, Operand::L("end".to_string())),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(1)),
        ];
        let labels = vec![("less".to_string(), 6), ("end".to_string(), 7)]
            .into_iter()
            .collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 20);
        assert_eq!(read_registers(&vm).rc, 1);
        assert_ne!(read_registers(&vm).rd, 1);
    }

    #[test]
    fn can_call_and_return() {
        let input = vec![
            Instruction::new_l(Opcode::Call, Operand::L("start".to_string())),
            Instruction::new_l(Opcode::Jmp, Operand::L("end".to_string())),
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(20)),
            Instruction::new(
                Opcode::Add,
                Operand::R(Register::Rb),
                Operand::R(Register::Ra),
            ),
            Instruction::new_e(Opcode::Return),
        ];
        let labels = vec![("start".to_string(), 2), ("end".to_string(), 5)]
            .into_iter()
            .collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 20);
        assert_eq!(read_registers(&vm).rb, 20);
    }

    #[test]
    fn can_jump_if_equal() {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(10)),
            Instruction::new(
                Opcode::Test,
                Operand::R(Register::Ra),
                Operand::R(Register::Rb),
            ),
            Instruction::new_l(Opcode::Je, Operand::L("end".to_string())),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(1)),
        ];
        let labels = vec![("end".to_string(), 6)].into_iter().collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 10);
        assert_eq!(read_registers(&vm).rc, 0);
    }

    #[test]
    fn can_jump_if_not_equal() {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(20)),
            Instruction::new(
                Opcode::Test,
                Operand::R(Register::Ra),
                Operand::R(Register::Rb),
            ),
            Instruction::new_l(Opcode::Jne, Operand::L("end".to_string())),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(1)),
        ];
        let labels = vec![("end".to_string(), 6)].into_iter().collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 20);
        assert_eq!(read_registers(&vm).rc, 0);
    }

    #[test]
    fn can_jump_if_greater_than_or_equal() {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(10)),
            Instruction::new(
                Opcode::Test,
                Operand::R(Register::Ra),
                Operand::R(Register::Rb),
            ),
            Instruction::new_l(Opcode::Jge, Operand::L("end".to_string())),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(1)),
        ];
        let labels = vec![("end".to_string(), 6)].into_iter().collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 10);
        assert_ne!(read_registers(&vm).rc, 1);
    }

    #[test]
    fn can_jump_if_less_than_or_equal() {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(20)),
            Instruction::new(
                Opcode::Test,
                Operand::R(Register::Ra),
                Operand::R(Register::Rb),
            ),
            Instruction::new_l(Opcode::Jle, Operand::L("end".to_string())),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(1)),
        ];
        let labels = vec![("end".to_string(), 6)].into_iter().collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 20);
        assert_ne!(read_registers(&vm).rc, 1);
    }

    #[test]
    fn can_jump_if_greater_than() {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(20)),
            Instruction::new(
                Opcode::Test,
                Operand::R(Register::Ra),
                Operand::R(Register::Rb),
            ),
            Instruction::new_l(Opcode::Jg, Operand::L("end".to_string())),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(1)),
        ];
        let labels = vec![("end".to_string(), 6)].into_iter().collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 20);
        assert_eq!(read_registers(&vm).rc, 1);
    }

    #[test]
    fn can_jump_if_less_than() {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(20)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(10)),
            Instruction::new(
                Opcode::Test,
                Operand::R(Register::Ra),
                Operand::R(Register::Rb),
            ),
            Instruction::new_l(Opcode::Jl, Operand::L("end".to_string())),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(1)),
        ];
        let labels = vec![("end".to_string(), 6)].into_iter().collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 20);
        assert_eq!(read_registers(&vm).rb, 10);
        assert_eq!(read_registers(&vm).rc, 1);
    }

    #[test]
    fn nop_does_nothing() {
        let input = vec![Instruction::new_e(Opcode::Nop)];
        let vm = run(input).unwrap();
        assert_eq!(read_registers(&vm).ra, 0);
    }

    #[test]
    fn can_halt() {
        let input = vec![
            Instruction::new_e(Opcode::Hlt),
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
        ];
        let vm = run(input).unwrap();
        assert_eq!(read_registers(&vm).ra, 0);
    }

    #[test]
    fn can_assert() {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(10)),
            Instruction::new(
                Opcode::Assert,
                Operand::R(Register::Ra),
                Operand::R(Register::Rb),
            ),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(1)),
        ];
        let labels = vec![("end".to_string(), 6)].into_iter().collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 10);
        assert_eq!(read_registers(&vm).rc, 1);
    }

    #[test]
    fn can_assert_failure() {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(20)),
            Instruction::new(
                Opcode::Assert,
                Operand::R(Register::Ra),
                Operand::R(Register::Rb),
            ),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(1)),
        ];
        let labels = vec![("end".to_string(), 6)].into_iter().collect();
        let vm = run_l(input, labels);
        assert_eq!(vm.unwrap_err(), "Assertion failed at ins 2.".to_string());
    }
}
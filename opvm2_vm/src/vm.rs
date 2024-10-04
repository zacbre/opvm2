use std::{collections::BTreeMap, sync::MutexGuard};

use extism::UserData;
use opvm2::{
    instruction::Instruction,
    parser::program::{LabelValue, Program},
    plugin_interface::OnInstructionValue,
};

use crate::{
    machine_context::MachineContext, memory::Memory, opcode::Opcode, operand::Operand,
    plugin::PluginLoader, CompiledProgram,
};

#[derive(Debug)]
pub struct Vm {
    pub context: UserData<MachineContext>,
    pub plugin: PluginLoader,
}

impl Vm {
    pub fn new(context: MachineContext) -> Vm {
        let context = UserData::new(context);

        Vm {
            context: context.clone(),
            plugin: PluginLoader::new(context),
        }
    }

    pub fn new_e() -> Vm {
        let context = UserData::new(MachineContext::new());

        Vm {
            context: context.clone(),
            plugin: PluginLoader::new(context),
        }
    }

    pub fn check_pc(&self) -> usize {
        let context = self.context.get().map_err(|e| e.to_string()).unwrap();
        let context = context.lock().unwrap();
        *context.registers.clone().check_pc()
    }

    pub fn check_address(&self) -> usize {
        let context = self.context.get().map_err(|e| e.to_string()).unwrap();
        let context = context.lock().unwrap();
        context.memory.address()
    }

    pub fn get_instruction(&self) -> Instruction {
        let context = self.context.get().map_err(|e| e.to_string()).unwrap();
        let mut context = context.lock().unwrap();
        let pc = *context.registers.check_pc();
        let pre_decoded = context.memory.get_instruction(pc);
        Instruction::decode(pre_decoded)
    }

    pub fn run_program(&mut self, program: Program) -> Result<(), String> {
        let program = CompiledProgram::from(program);
        self.run(program)
    }

    pub fn run(&mut self, program: CompiledProgram) -> Result<(), String> {
        let start_address = program.start_address;
        self.plugin.load_all(&program.plugins, false)?;

        {
            let context = self.context.get().map_err(|e| e.to_string()).unwrap();
            let mut context = context.lock().unwrap();
            context.registers.set_pc(start_address);
            context.memory = Memory::from_raw(program.program, program.memory_address);
            context.base_address = start_address;
        }

        'outer: while (self.check_pc() as usize) < self.check_address() {
            let pc = self.check_pc();
            let item = self.get_instruction();
            let ins = OnInstructionValue {
                opcode: item.opcode.clone(),
                lhs: item.lhs.clone(),
                rhs: item.rhs.clone(),
                pc,
            };
            self.plugin.execute_plugin_fn(
                "handle_instruction".to_string(),
                ins.clone(),
                true,
                start_address,
            )?;
            // get plugin name from memory.
            let plugin_name = match ins.opcode {
                Opcode::Plugin(opvm2::opcode::PluginValue::Address(address)) => {
                    let context = self.context.get().map_err(|e| e.to_string()).unwrap();
                    let mut context = context.lock().unwrap();
                    let address = address as usize;
                    let plugin_name_bytes = context.memory.get_literal(address);
                    let plugin_name = String::from_utf8(plugin_name_bytes.to_vec()).unwrap();
                    plugin_name
                }
                _ => item.opcode.to_string(),
            };
            match self.plugin.execute_plugin_fn(
                format!("handle_{}", &plugin_name.to_lowercase()),
                ins.clone(),
                false,
                start_address,
            ) {
                Ok(count) => {
                    if count > 0 {
                        continue 'outer;
                    }
                }
                Err(e) => return Err(e),
            };

            let context = self.context.get().map_err(|e| e.to_string()).unwrap();
            let mut context = context.lock().unwrap();
            let (lhs, rhs) = (
                self.get_value(&mut context, &item.lhs)?,
                self.get_value(&mut context, &item.rhs)?,
            );

            match item.opcode.clone() {
                Opcode::Mov => {
                    // todo: handle operands with offsets here?
                    let lhs = item.lhs.get_register()?;
                    context.registers.set(&lhs, rhs.expect("rhs is None"));
                }
                Opcode::Add => {
                    self.math(&mut context, &item.lhs, &item.rhs, item.opcode.clone())?
                }
                Opcode::Sub => {
                    self.math(&mut context, &item.lhs, &item.rhs, item.opcode.clone())?
                }
                Opcode::Mul => {
                    self.math(&mut context, &item.lhs, &item.rhs, item.opcode.clone())?
                }
                Opcode::Div => {
                    self.math(&mut context, &item.lhs, &item.rhs, item.opcode.clone())?
                }
                Opcode::Mod => {
                    self.math(&mut context, &item.lhs, &item.rhs, item.opcode.clone())?
                }
                Opcode::Xor => {
                    self.math(&mut context, &item.lhs, &item.rhs, item.opcode.clone())?
                }
                Opcode::Inc => {
                    context
                        .registers
                        .set(&item.lhs.get_register()?, lhs.expect("lhs is none") + 1);
                }
                Opcode::Dec => {
                    context
                        .registers
                        .set(&item.lhs.get_register()?, lhs.expect("lhs is none") - 1);
                }
                Opcode::Print => {
                    print!("{}", lhs.expect("lhs is none"));
                }
                Opcode::Push => {
                    context.stack.push(lhs.expect("lhs is none"));
                }
                Opcode::Pop => {
                    let lhs = item.lhs.get_register()?;
                    let value = context.stack.pop().unwrap();
                    context.registers.set(&lhs, value);
                }
                Opcode::Dup => {
                    let peeked = *context.stack.peek().unwrap();
                    context.stack.push(peeked);
                }
                Opcode::Test => self.test(&mut context, &item.lhs, &item.rhs),
                Opcode::Jmp => {
                    context
                        .registers
                        .set_pc(start_address + lhs.expect("lhs is none"));
                    continue;
                }
                Opcode::Je => {
                    if context.registers.check_equals_flag() {
                        context
                            .registers
                            .set_pc(start_address + lhs.expect("lhs is none"));
                        continue;
                    }
                }
                Opcode::Jne => {
                    if !context.registers.check_equals_flag() {
                        context
                            .registers
                            .set_pc(start_address + lhs.expect("lhs is none"));
                        continue;
                    }
                }
                Opcode::Jle => {
                    if context.registers.check_equals_flag()
                        || context.registers.check_less_than_flag()
                    {
                        context
                            .registers
                            .set_pc(start_address + lhs.expect("lhs is none"));
                        continue;
                    }
                }
                Opcode::Jge => {
                    if context.registers.check_equals_flag()
                        || context.registers.check_greater_than_flag()
                    {
                        context
                            .registers
                            .set_pc(start_address + lhs.expect("lhs is none"));
                        continue;
                    }
                }
                Opcode::Jl => {
                    if context.registers.check_less_than_flag() {
                        context
                            .registers
                            .set_pc(start_address + lhs.expect("lhs is none"));
                        continue;
                    }
                }
                Opcode::Jg => {
                    if context.registers.check_greater_than_flag() {
                        context
                            .registers
                            .set_pc(start_address + lhs.expect("lhs is none"));
                        continue;
                    }
                }
                Opcode::Call => {
                    let call_stack_pointer = context.registers.check_pc() + 16;
                    context.call_stack.push(call_stack_pointer);
                    context
                        .registers
                        .set_pc(start_address + lhs.expect("lhs is none"));
                    continue;
                }
                Opcode::Return => {
                    let return_address = context.call_stack.pop().unwrap();
                    context.registers.set_pc(return_address);
                    continue;
                }
                Opcode::Assert => {
                    self.test(&mut context, &item.lhs, &item.rhs);
                    if !context.registers.check_equals_flag() {
                        return Err(format!(
                            "Assertion failed at ins {}.",
                            context.registers.check_pc()
                        ));
                    }
                    context.registers.reset_flags();
                }
                Opcode::Sleep => {
                    std::thread::sleep(std::time::Duration::from_millis(
                        lhs.expect("lhs is none") as u64
                    ));
                }
                Opcode::Nop => {}
                Opcode::Halt => {
                    return Ok(());
                }
                Opcode::Plugin(s) => {
                    // error, this wasn't handled?
                    return Err(format!("Plugin for '{}' not found", s));
                }
            }
            context.registers.increment_pc();
        }
        // bug in rust perhaps? using print! causes a % to be outputted if no newline is printed afterwards.
        println!("");
        Ok(())
    }

    fn test(&mut self, context: &mut MutexGuard<MachineContext>, lhs: &Operand, rhs: &Operand) {
        let lhs_value = self.get_value(context, lhs);
        let rhs_value = self.get_value(context, rhs);
        context.registers.reset_flags();
        if lhs_value == rhs_value {
            context.registers.set_equals_flag(true);
        }
        if lhs_value < rhs_value {
            context.registers.set_less_than_flag(true);
        }
        if lhs_value > rhs_value {
            context.registers.set_greater_than_flag(true);
        }
    }

    fn get_value(
        &self,
        context: &mut MutexGuard<MachineContext>,
        operand: &Operand,
    ) -> Result<Option<usize>, String> {
        match operand {
            Operand::Number(n) => Ok(Some(*n)),
            Operand::Register(r) => Ok(Some(context.registers.get(&r))),
            Operand::Label(l) => match l {
                LabelValue::Address(n) => Ok(Some(*n as usize)),
                _ => Err(format!("Label '{:?}' is not an address", l)),
            },
            // Operand::Offset(o) => {
            // what we want to do here is get the value at the specified offset.
            // if it is just a single lhs then we want to get the address?
            // if it is a lhs and rhs then we want to get the value at the address + offset
            // in the future, to get an address and specify an offset, we will do something like this:
            // mov lhs, [rhs] + operand

            // let lhs: Operand = o.lhs_operand.try_into()?;
            // match o.rhs_operand {
            //     Some(rhs) => {
            //         let rhs: Operand = rhs.try_into()?;
            //         self.math(context, &lhs, &rhs)?;
            //     }
            //     None => {
            //         // get the address of the lhs.
            //         // this should only work when X is a label?
            //     }
            // }
            // // match the operator

            // Ok(Some(value + offset))
            // }
            _ => Ok(None),
        }
    }

    fn math(
        &mut self,
        context: &mut MutexGuard<MachineContext>,
        lhs: &Operand,
        rhs: &Operand,
        operator: Opcode,
    ) -> Result<(), String> {
        let lhs_value = self.get_value(context, lhs)?.expect("lhs is none");
        let rhs_value = self.get_value(context, rhs)?.expect("rhs is none");
        let value = match operator {
            Opcode::Add => lhs_value + rhs_value,
            Opcode::Sub => lhs_value - rhs_value,
            Opcode::Mul => lhs_value * rhs_value,
            Opcode::Div => lhs_value / rhs_value,
            Opcode::Mod => lhs_value % rhs_value,
            Opcode::Xor => lhs_value ^ rhs_value,
            _ => panic!("Invalid operator for math operation"),
        };

        context.registers.set(&lhs.get_register()?, value);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use crate::instruction::Instruction;
    use crate::opcode::Opcode;
    use crate::operand::Operand;
    use crate::parser::program::{Labels, Program};
    use crate::register::Register;

    fn run(input: Vec<Instruction>) -> Result<Vm, String> {
        let mut vm = super::Vm::new_e();
        let program = Program {
            instructions: input,
            labels: Labels::new(),
            plugins: vec![],
        };
        vm.run_program(program)?;
        Ok(vm)
    }

    fn run_l(input: Vec<Instruction>, labels: Vec<(String, LabelValue)>) -> Result<Vm, String> {
        let mut vm = super::Vm::new_e();
        let program = Program {
            instructions: input,
            labels: Labels::from(labels),
            plugins: vec![],
        };
        vm.run_program(program)?;
        Ok(vm)
    }

    fn read_registers(vm: &Vm) -> Registers {
        let context = vm.context.get().map_err(|e| e.to_string()).unwrap();
        let context = context.lock().unwrap();
        context.registers.clone()
    }

    fn pop_stack(vm: &mut Vm) -> Result<usize, String> {
        let context = vm.context.get().map_err(|e| e.to_string()).unwrap();
        let mut context = context.lock().unwrap();
        context.stack.pop().ok_or("Stack is empty".to_string())
    }

    use super::Vm;
    use opvm2::parser::program::LabelValue;
    use opvm2::register::Registers;
    use test_case::test_case;

    #[test]
    fn can_run_vm() -> Result<(), String> {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rc),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Add,
                Operand::Register(Register::Ra),
                Operand::Register(Register::Rb),
            ),
            Instruction::new(
                Opcode::Add,
                Operand::Register(Register::Ra),
                Operand::Register(Register::Rc),
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
            Operand::Register(Register::Ra),
            Operand::Number(10),
        )];
        let vm = run(input)?;
        assert_eq!(read_registers(&vm).ra, 10);
        Ok(())
    }

    #[test]
    fn can_mov_value_from_register_to_register() {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Register(Register::Ra),
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
        lval: usize,
        rhs: &str,
        rval: usize,
        expected: usize,
    ) -> Result<(), String> {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::try_from(lhs.to_string()).unwrap()),
                Operand::Number(lval),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::try_from(rhs.to_string()).unwrap()),
                Operand::Number(rval),
            ),
            Instruction::new(
                opcode,
                Operand::Register(Register::try_from(lhs.to_string()).unwrap()),
                Operand::Register(Register::try_from(rhs.to_string()).unwrap()),
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
        lval: usize,
        rval: usize,
        expected: usize,
    ) -> Result<(), String> {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::try_from(lhs.to_string()).unwrap()),
                Operand::Number(lval),
            ),
            Instruction::new(
                opcode,
                Operand::Register(Register::try_from(lhs.to_string()).unwrap()),
                Operand::Number(rval),
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
            Instruction::new_l(Opcode::Push, Operand::Number(10)),
            Instruction::new_l(Opcode::Pop, Operand::Register(Register::Ra)),
        ];
        let vm = run(input)?;
        assert_eq!(read_registers(&vm).ra, 10);
        Ok(())
    }

    #[test]
    fn can_push_and_pop_multiple() {
        let input = vec![
            Instruction::new_l(Opcode::Push, Operand::Number(10)),
            Instruction::new_l(Opcode::Push, Operand::Number(20)),
            Instruction::new_l(Opcode::Push, Operand::Number(30)),
            Instruction::new_l(Opcode::Pop, Operand::Register(Register::Ra)),
            Instruction::new_l(Opcode::Pop, Operand::Register(Register::Rb)),
        ];
        let mut vm = run(input).unwrap();
        assert_eq!(read_registers(&vm).ra, 30);
        assert_eq!(read_registers(&vm).rb, 20);
        assert_eq!(pop_stack(&mut vm), Ok(10));
    }

    #[test]
    fn can_jump() -> Result<(), String> {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::R0),
                Operand::Number(64),
            ),
            Instruction::new_l(Opcode::Jmp, Operand::Register(Register::R0)),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Number(20),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rc),
                Operand::Number(30),
            ),
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
            Instruction::new_l(
                Opcode::Jmp,
                Operand::Label(LabelValue::Literal("start".to_string())),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Number(20),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rc),
                Operand::Number(30),
            ),
        ];
        let labels = vec![("start".to_string(), LabelValue::Address(3))];
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 0);
        assert_eq!(read_registers(&vm).rb, 0);
        assert_eq!(read_registers(&vm).rc, 30);
    }

    #[test]
    fn can_test() {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Number(20),
            ),
            Instruction::new(
                Opcode::Test,
                Operand::Register(Register::Ra),
                Operand::Register(Register::Rb),
            ),
            Instruction::new_l(
                Opcode::Jle,
                Operand::Label(LabelValue::Literal("less".to_string())),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rd),
                Operand::Number(1),
            ),
            Instruction::new_l(
                Opcode::Jmp,
                Operand::Label(LabelValue::Literal("end".to_string())),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rc),
                Operand::Number(1),
            ),
        ];
        let labels = vec![
            ("less".to_string(), LabelValue::Address(6)),
            ("end".to_string(), LabelValue::Address(7)),
        ];
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 20);
        assert_eq!(read_registers(&vm).rc, 1);
        assert_ne!(read_registers(&vm).rd, 1);
    }

    #[test]
    fn can_call_and_return() {
        let input = vec![
            Instruction::new_l(
                Opcode::Call,
                Operand::Label(LabelValue::Literal("start".to_string())),
            ),
            Instruction::new_l(
                Opcode::Jmp,
                Operand::Label(LabelValue::Literal("end".to_string())),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(20),
            ),
            Instruction::new(
                Opcode::Add,
                Operand::Register(Register::Rb),
                Operand::Register(Register::Ra),
            ),
            Instruction::new_e(Opcode::Return),
        ];
        let labels = vec![
            ("start".to_string(), LabelValue::Address(2)),
            ("end".to_string(), LabelValue::Address(5)),
        ];
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 20);
        assert_eq!(read_registers(&vm).rb, 20);
    }

    #[test]
    fn can_jump_if_equal() {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Test,
                Operand::Register(Register::Ra),
                Operand::Register(Register::Rb),
            ),
            Instruction::new_l(
                Opcode::Je,
                Operand::Label(LabelValue::Literal("end".to_string())),
            ), // todo: instead of passing literal, pass address where the label goes.
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rc),
                Operand::Number(1),
            ),
        ];
        let labels = vec![("end".to_string(), LabelValue::Address(6))];
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 10);
        assert_eq!(read_registers(&vm).rc, 0);
    }

    #[test]
    fn can_jump_if_not_equal() {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Number(20),
            ),
            Instruction::new(
                Opcode::Test,
                Operand::Register(Register::Ra),
                Operand::Register(Register::Rb),
            ),
            Instruction::new_l(
                Opcode::Jne,
                Operand::Label(LabelValue::Literal("end".to_string())),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rc),
                Operand::Number(1),
            ),
        ];
        let labels = vec![("end".to_string(), LabelValue::Address(6))];
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 20);
        assert_eq!(read_registers(&vm).rc, 0);
    }

    #[test]
    fn can_jump_if_greater_than_or_equal() {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Test,
                Operand::Register(Register::Ra),
                Operand::Register(Register::Rb),
            ),
            Instruction::new_l(
                Opcode::Jge,
                Operand::Label(LabelValue::Literal("end".to_string())),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rc),
                Operand::Number(1),
            ),
        ];
        let labels = vec![("end".to_string(), LabelValue::Address(6))];
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 10);
        assert_ne!(read_registers(&vm).rc, 1);
    }

    #[test]
    fn can_jump_if_less_than_or_equal() {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Number(20),
            ),
            Instruction::new(
                Opcode::Test,
                Operand::Register(Register::Ra),
                Operand::Register(Register::Rb),
            ),
            Instruction::new_l(
                Opcode::Jle,
                Operand::Label(LabelValue::Literal("end".to_string())),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rc),
                Operand::Number(1),
            ),
        ];
        let labels = vec![("end".to_string(), LabelValue::Address(6))];
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 20);
        assert_ne!(read_registers(&vm).rc, 1);
    }

    #[test]
    fn can_jump_if_greater_than() {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Number(20),
            ),
            Instruction::new(
                Opcode::Test,
                Operand::Register(Register::Ra),
                Operand::Register(Register::Rb),
            ),
            Instruction::new_l(
                Opcode::Jg,
                Operand::Label(LabelValue::Literal("end".to_string())),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rc),
                Operand::Number(1),
            ),
        ];
        let labels = vec![("end".to_string(), LabelValue::Address(6))];
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 20);
        assert_eq!(read_registers(&vm).rc, 1);
    }

    #[test]
    fn can_jump_if_less_than() {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(20),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Test,
                Operand::Register(Register::Ra),
                Operand::Register(Register::Rb),
            ),
            Instruction::new_l(
                Opcode::Jl,
                Operand::Label(LabelValue::Literal("end".to_string())),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rc),
                Operand::Number(1),
            ),
        ];
        let labels = vec![("end".to_string(), LabelValue::Address(6))];
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
            Instruction::new_e(Opcode::Halt),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
        ];
        let vm = run(input).unwrap();
        assert_eq!(read_registers(&vm).ra, 0);
    }

    #[test]
    fn can_assert() {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Assert,
                Operand::Register(Register::Ra),
                Operand::Register(Register::Rb),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rc),
                Operand::Number(1),
            ),
        ];
        let labels = vec![("end".to_string(), LabelValue::Address(6))];
        let vm = run_l(input, labels).unwrap();
        assert_eq!(read_registers(&vm).ra, 10);
        assert_eq!(read_registers(&vm).rb, 10);
        assert_eq!(read_registers(&vm).rc, 1);
    }

    #[test]
    fn can_assert_failure() {
        let input = vec![
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Ra),
                Operand::Number(10),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rb),
                Operand::Number(20),
            ),
            Instruction::new(
                Opcode::Assert,
                Operand::Register(Register::Ra),
                Operand::Register(Register::Rb),
            ),
            Instruction::new(
                Opcode::Mov,
                Operand::Register(Register::Rc),
                Operand::Number(1),
            ),
        ];
        let labels = vec![("end".to_string(), LabelValue::Address(5))];
        let vm = run_l(input, labels);
        assert_eq!(vm.unwrap_err(), "Assertion failed at ins 32.".to_string());
    }

    #[test]
    fn can_sleep() -> Result<(), String> {
        let start = Instant::now();
        let input = vec![Instruction::new_l(Opcode::Sleep, Operand::Number(100))];
        let labels = vec![("end".to_string(), LabelValue::Address(6))];
        run_l(input, labels)?;
        let end = start.elapsed();
        assert!(end.as_millis() > 100);
        Ok(())
    }
}

use crate::{
    opcode::Opcode, operand::Operand, parser::program::Program, register::Registers, stack::Stack,
};

#[derive(Debug, PartialEq)]
pub struct Vm {
    pub registers: Registers,
    pub stack: Stack<u64>,
    pub call_stack: Stack<u64>,
    pub current_program: Program,
}

impl Vm {
    pub fn new() -> Vm {
        Vm {
            registers: Registers::new(),
            stack: Stack::new(),
            call_stack: Stack::new(),
            current_program: Program::empty(),
        }
    }

    pub fn run(&mut self, program: Program) -> Result<(), String> {
        self.current_program = program.clone();
        while (*self.registers.check_pc() as usize) < program.instructions.len() {
            let item = &program.instructions[*self.registers.check_pc() as usize];
            match item.opcode {
                Opcode::Mov => {
                    let lhs = item.lhs.get_register()?;
                    let rhs_value = self.get_value(&item.rhs);
                    self.registers.set(&lhs, rhs_value);
                }
                Opcode::Add => self.math(&item.lhs, &item.rhs, item.opcode)?,
                Opcode::Sub => self.math(&item.lhs, &item.rhs, item.opcode)?,
                Opcode::Mul => self.math(&item.lhs, &item.rhs, item.opcode)?,
                Opcode::Div => self.math(&item.lhs, &item.rhs, item.opcode)?,
                Opcode::Mod => self.math(&item.lhs, &item.rhs, item.opcode)?,
                Opcode::Xor => self.math(&item.lhs, &item.rhs, item.opcode)?,
                Opcode::Inc => {
                    let lhs = self.get_value(&item.lhs);
                    self.registers.set(&item.lhs.get_register()?, lhs + 1);
                }
                Opcode::Dec => {
                    let lhs = self.get_value(&item.lhs);
                    self.registers.set(&item.lhs.get_register()?, lhs - 1);
                }
                Opcode::Print => {
                    let lhs_value = self.get_value(&item.lhs);
                    println!("{}", lhs_value);
                }
                Opcode::Push => {
                    let lhs_value = self.get_value(&item.lhs);
                    self.stack.push(lhs_value);
                }
                Opcode::Pop => {
                    let lhs = item.lhs.get_register()?;
                    let value = self.stack.pop().unwrap();
                    self.registers.set(&lhs, value);
                }
                Opcode::Dup => {
                    match self.stack.peek() {
                        Some(value) => self.stack.push(*value),
                        None => return Err("Stack is empty".to_string()),
                    }
                }
                Opcode::Test => {
                    self.test(&item.lhs, &item.rhs);
                }
                Opcode::Jmp => {
                    self.registers.set_pc(self.get_value(&item.lhs));
                    continue;
                }
                Opcode::Je => {
                    if self.registers.check_equals_flag() {
                        self.registers.set_pc(self.get_value(&item.lhs));
                        continue;
                    }
                }
                Opcode::Jne => {
                    if !self.registers.check_equals_flag() {
                        self.registers.set_pc(self.get_value(&item.lhs));
                        continue;
                    }
                }
                Opcode::Jle => {
                    if self.registers.check_equals_flag() || self.registers.check_less_than_flag() {
                        self.registers.set_pc(self.get_value(&item.lhs));
                        continue;
                    }
                }
                Opcode::Jge => {
                    if self.registers.check_equals_flag()
                        || self.registers.check_greater_than_flag()
                    {
                        self.registers.set_pc(self.get_value(&item.lhs));
                        continue;
                    }
                }
                Opcode::Jl => {
                    if self.registers.check_less_than_flag() {
                        self.registers.set_pc(self.get_value(&item.lhs));
                        continue;
                    }
                }
                Opcode::Jg => {
                    if self.registers.check_greater_than_flag() {
                        self.registers.set_pc(self.get_value(&item.lhs));
                        continue;
                    }
                }
                Opcode::Call => {
                    self.call_stack.push(*self.registers.check_pc() + 1);
                    self.registers.set_pc(self.get_value(&item.lhs));
                    continue;
                }
                Opcode::Return => {
                    self.registers.set_pc(self.call_stack.pop().unwrap());
                    continue;
                }
                Opcode::Assert => {
                    self.test(&item.lhs, &item.rhs);
                    if !self.registers.check_equals_flag() {
                        return Err(format!("Assertion failed at ins {}.", self.registers.check_pc()));
                    }
                    self.registers.reset_flags();
                }
                Opcode::Nop => {},
                Opcode::Hlt => {
                    return Ok(());
                }
            }
            self.registers.increment_pc();
        }
        Ok(())
    }

    fn test(&mut self, lhs: &Operand, rhs: &Operand) {
        let lhs_value = self.get_value(lhs);
        let rhs_value = self.get_value(rhs);
        self.registers.reset_flags();
        if lhs_value == rhs_value {
            self.registers.set_equals_flag(true);
        }
        if lhs_value < rhs_value {
            self.registers.set_less_than_flag(true);
        }
        if lhs_value > rhs_value {
            self.registers.set_greater_than_flag(true);
        }
    }
    
    fn get_value(&self, operand: &Operand) -> u64 {
        match operand {
            Operand::N(n) => *n,
            Operand::R(r) => self.registers.get(&r),
            Operand::L(l) => self
                .current_program
                .labels
                .get(l)
                .unwrap()
                .clone()
                .try_into()
                .unwrap(),
            _ => panic!(
                "{}",
                format!("Invalid operand for operation: {:?}", operand)
            ),
        }
    }

    fn math(&mut self, lhs: &Operand, rhs: &Operand, operator: Opcode) -> Result<(), String> {
        let lhs_value = self.get_value(lhs);
        let rhs_value = self.get_value(rhs);
        let value = match operator {
            Opcode::Add => lhs_value + rhs_value,
            Opcode::Sub => lhs_value - rhs_value,
            Opcode::Mul => lhs_value * rhs_value,
            Opcode::Div => lhs_value / rhs_value,
            Opcode::Mod => lhs_value % rhs_value,
            Opcode::Xor => lhs_value ^ rhs_value,
            _ => panic!("Invalid operator for math operation"),
        };

        self.registers.set(&lhs.get_register()?, value);

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
        let mut vm = super::Vm::new();
        vm.run(Program {
            instructions: input,
            labels: HashMap::new(),
        })?;
        Ok(vm)
    }

    fn run_l(input: Vec<Instruction>, labels: HashMap<String, usize>) -> Result<Vm, String> {
        let mut vm = super::Vm::new();
        vm.run(Program {
            instructions: input,
            labels,
        })?;
        Ok(vm)
    }

    #[test]
    fn can_create_vm() {
        let vm = super::Vm::new();
        assert_eq!(
            vm,
            super::Vm {
                registers: super::Registers::new(),
                stack: super::Stack::new(),
                call_stack: super::Stack::new(),
                current_program: super::Program {
                    instructions: vec![],
                    labels: HashMap::new()
                }
            }
        );
    }

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
        assert_eq!(vm.registers.ra, 30);

        Ok(())
    }

    use test_case::test_case;

    use super::Vm;

    #[test]
    fn can_mov_value_to_register() -> Result<(), String> {
        let input = vec![Instruction::new(
            Opcode::Mov,
            Operand::R(Register::Ra),
            Operand::N(10),
        )];
        let vm = run(input)?;
        assert_eq!(vm.registers.ra, 10);
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
        assert_eq!(vm.registers.ra, 10);
        assert_eq!(vm.registers.rb, 10);
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
            vm.registers
                .get(&Register::try_from(lhs.to_string()).unwrap()),
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
            vm.registers
                .get(&Register::try_from(lhs.to_string()).unwrap()),
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
        assert_eq!(vm.registers.ra, 10);
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
        assert_eq!(vm.registers.ra, 30);
        assert_eq!(vm.registers.rb, 20);
        assert_eq!(vm.stack.pop(), Some(10));
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
        assert_eq!(vm.registers.ra, 0);
        assert_eq!(vm.registers.rb, 0);
        assert_eq!(vm.registers.rc, 30);
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
        assert_eq!(vm.registers.ra, 0);
        assert_eq!(vm.registers.rb, 0);
        assert_eq!(vm.registers.rc, 30);
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
        assert_eq!(vm.registers.ra, 10);
        assert_eq!(vm.registers.rb, 20);
        assert_eq!(vm.registers.rc, 1);
        assert_ne!(vm.registers.rd, 1);
    }

    #[test]
    fn can_call_and_return() {
        let input = vec![
            Instruction::new_l(Opcode::Call, Operand::L("start".to_string())),
            Instruction::new_l(Opcode::Jmp, Operand::L("end".to_string())),
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(20)),
            Instruction::new(Opcode::Add, Operand::R(Register::Rb), Operand::R(Register::Ra)),
            Instruction::new_e(Opcode::Return),
        ];
        let labels = vec![("start".to_string(), 2), ("end".to_string(), 5)].into_iter().collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(vm.registers.ra, 20);
        assert_eq!(vm.registers.rb, 20);
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
        let labels = vec![("end".to_string(), 6)]
            .into_iter()
            .collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(vm.registers.ra, 10);
        assert_eq!(vm.registers.rb, 10);
        assert_eq!(vm.registers.rc, 0);
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
        let labels = vec![("end".to_string(), 6)]
            .into_iter()
            .collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(vm.registers.ra, 10);
        assert_eq!(vm.registers.rb, 20);
        assert_eq!(vm.registers.rc, 0);
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
        let labels = vec![("end".to_string(), 6)]
            .into_iter()
            .collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(vm.registers.ra, 10);
        assert_eq!(vm.registers.rb, 10);
        assert_ne!(vm.registers.rc, 1);
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
        let labels = vec![("end".to_string(), 6)]
            .into_iter()
            .collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(vm.registers.ra, 10);
        assert_eq!(vm.registers.rb, 20);
        assert_ne!(vm.registers.rc, 1);
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
        let labels = vec![("end".to_string(), 6)]
            .into_iter()
            .collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(vm.registers.ra, 10);
        assert_eq!(vm.registers.rb, 20);
        assert_eq!(vm.registers.rc, 1);
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
        let labels = vec![("end".to_string(), 6)]
            .into_iter()
            .collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(vm.registers.ra, 20);
        assert_eq!(vm.registers.rb, 10);
        assert_eq!(vm.registers.rc, 1);
    }

    #[test]
    fn nop_does_nothing() {
        let input = vec![Instruction::new_e(Opcode::Nop)];
        let vm = run(input).unwrap();
        assert_eq!(vm.registers.ra, 0);
    }

    #[test]
    fn can_halt() {
        let input = vec![
            Instruction::new_e(Opcode::Hlt),
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
        ];
        let vm = run(input).unwrap();
        assert_eq!(vm.registers.ra, 0);
    }

    #[test]
    fn can_assert() {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(10)),
            Instruction::new(Opcode::Assert, Operand::R(Register::Ra), Operand::R(Register::Rb)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(1)),
        ];
        let labels = vec![("end".to_string(), 6)]
            .into_iter()
            .collect();
        let vm = run_l(input, labels).unwrap();
        assert_eq!(vm.registers.ra, 10);
        assert_eq!(vm.registers.rb, 10);
        assert_eq!(vm.registers.rc, 1);
    }

    #[test]
    fn can_assert_failure() {
        let input = vec![
            Instruction::new(Opcode::Mov, Operand::R(Register::Ra), Operand::N(10)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rb), Operand::N(20)),
            Instruction::new(Opcode::Assert, Operand::R(Register::Ra), Operand::R(Register::Rb)),
            Instruction::new(Opcode::Mov, Operand::R(Register::Rc), Operand::N(1)),
        ];
        let labels = vec![("end".to_string(), 6)]
            .into_iter()
            .collect();
        let vm = run_l(input, labels);
        assert_eq!(
            vm,
            Err("Assertion failed at ins 2.".to_string())
        );
    }
}

pub mod instruction;
pub mod lexer;
pub mod opcode;
pub mod operand;
pub mod parser;
pub mod register;
pub mod stack;
pub mod vm;

#[cfg(test)]
mod test {
    use crate::parser::program::Program;

    #[test]
    fn can_add_two_numbers() -> Result<(), String> {
        let mut vm = super::vm::Vm::new();
        let program = Program::from(
            r"
            mov ra, 1
            mov rb, 2
            add ra, rb
        ",
        );
        vm.run(program)?;
        assert_eq!(vm.registers.get(&crate::register::Register::Ra), 3);
        Ok(())
    }

    #[test]
    fn can_xor_two_numbers() {
        let mut vm = super::vm::Vm::new();
        let program = Program::from(
            r"
            mov ra, 3
            mov rb, 5
            xor ra, rb
        ",
        );
        vm.run(program).unwrap();
        assert_eq!(vm.registers.get(&crate::register::Register::Ra), 6);
    }

    #[test]
    fn can_jump() -> Result<(), String> {
        let mut vm = super::vm::Vm::new();
        let program = Program::from(
            r"
            mov r0, 4
            jmp r0
            mov ra, 2   ; this should be skipped
            mov rb, 3   ; this should be skipped
            mov rc, 5
        ",
        );
        vm.run(program)?;
        assert_eq!(vm.registers.get(&crate::register::Register::Ra), 0);
        assert_eq!(vm.registers.get(&crate::register::Register::Rb), 0);
        assert_eq!(vm.registers.get(&crate::register::Register::Rc), 5);
        Ok(())
    }

    #[test]
    fn can_jump_with_labels() -> Result<(), String> {
        let mut vm = super::vm::Vm::new();
        let program = Program::from(
            r"
            jmp start
            mov ra, 2   ; this should be skipped
            mov rb, 3   ; this should be skipped
            _start: mov rc, 5
        ",
        );
        println!("{:?}", program);
        vm.run(program)?;
        assert_eq!(vm.registers.get(&crate::register::Register::Ra), 0);
        assert_eq!(vm.registers.get(&crate::register::Register::Rb), 0);
        assert_eq!(vm.registers.get(&crate::register::Register::Rc), 5);
        Ok(())
    }

    #[test]
    fn can_jump_when_less_than() {
        let mut vm = super::vm::Vm::new();
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
        vm.run(program).unwrap();
        assert_eq!(vm.registers.get(&crate::register::Register::Rc), 10);
    }

    #[test]
    fn can_jump_when_less_than_or_equal() {
        let mut vm = super::vm::Vm::new();
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
        vm.run(program).unwrap();
        assert_eq!(vm.registers.get(&crate::register::Register::Rc), 10);
    }

    #[test]
    fn can_jump_when_greater_than() {
        let mut vm = super::vm::Vm::new();
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
        vm.run(program).unwrap();
        assert_eq!(vm.registers.get(&crate::register::Register::Rc), 10);
    }

    #[test]
    fn can_jump_when_greater_than_or_equal() {
        let mut vm = super::vm::Vm::new();
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
        vm.run(program).unwrap();
        assert_eq!(vm.registers.get(&crate::register::Register::Rc), 10);
    }

    #[test]
    fn can_jump_when_equal() {
        let mut vm = super::vm::Vm::new();
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
        vm.run(program).unwrap();
        assert_eq!(vm.registers.get(&crate::register::Register::Rc), 10);
    }

    #[test]
    fn can_jump_when_not_equal() {
        let mut vm = super::vm::Vm::new();
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
        vm.run(program).unwrap();
        assert_eq!(vm.registers.get(&crate::register::Register::Rc), 10);
    }
}

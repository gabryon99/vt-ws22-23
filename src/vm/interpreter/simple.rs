use crate::vm::{
    opcode::{self, OpCode},
    VM,
};

use super::Interpreter;

pub struct SimpleInterpreter;

impl Interpreter for SimpleInterpreter {
    fn run(&self, vm: &mut VM) {
        loop {
            if vm.halt {
                break;
            }

            let instr = vm.running_program.data[vm.registers.ip as usize];

            match OpCode::try_from(instr).unwrap() {
                OpCode::HALT => self.halt(vm, instr),
                OpCode::CLRA => self.clra(vm, instr),
                OpCode::INC3A => self.inc3a(vm, instr),
                OpCode::DECA => self.deca(vm, instr),
                OpCode::SETL => self.setl(vm, instr),
                OpCode::BACK7 => self.back7(vm, instr),
            }

            // Increase Instruction Pointer of instruction size
            vm.registers.ip += 1;
        }
    }

    fn halt(&self, vm: &mut VM, _instr: u8) {
        vm.halt = true;
    }

    fn clra(&self, vm: &mut VM, _instr: u8) {
        vm.registers.acc = 0;
    }

    fn inc3a(&self, vm: &mut VM, _instr: u8) {
        vm.registers.acc += 3;
    }

    fn deca(&self, vm: &mut VM, _instr: u8) {
        vm.registers.acc -= 1;
    }

    fn setl(&self, vm: &mut VM, _instr: u8) {
        vm.registers.lc = vm.registers.acc;
    }

    fn back7(&self, vm: &mut VM, _instr: u8) {
        vm.registers.lc -= 1;
        if vm.registers.lc >= 0 {
            // Of 8 positions because the IP will be increased
            // after the function's execution
            vm.registers.ip -= 8;
        }
    }
}

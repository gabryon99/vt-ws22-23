use crate::vm::{
    opcode::{OpCode},
    VM,
};

use super::Interpreter;

pub struct SimpleInterpreter;

impl Interpreter for SimpleInterpreter {
    fn run(&self, vm: &VM) {
        use std::time::Instant;
        let now = Instant::now();

        loop {
            if vm.is_halt() {
                break;
            }

            let instr = vm.running_program.data[vm.registers.ip_value() as usize];

            match OpCode::try_from(instr).unwrap() {
                OpCode::HALT => self.halt(vm, instr),
                OpCode::CLRA => self.clra(vm, instr),
                OpCode::INC3A => self.inc3a(vm, instr),
                OpCode::DECA => self.deca(vm, instr),
                OpCode::SETL => self.setl(vm, instr),
                OpCode::BACK7 => self.back7(vm, instr),
            }

            // Increase Instruction Pointer of instruction size
            vm.registers.ip.replace(vm.registers.ip_value() + 1);
        }

        let elapsed = now.elapsed();
        println!("[simple] :: Elapsed: {:.2?}", elapsed);
    }

    fn halt(&self, vm: &VM, _instr: u8) {
        vm.halt.replace(true);
    }

    fn clra(&'_ self, vm: &VM, _instr: u8) {
        vm.registers.acc.replace(0);
    }

    fn inc3a(&'_ self, vm: &VM, _instr: u8) {
        vm.registers.acc.replace(vm.registers.acc_value() + 3);
    }

    fn deca(&'_ self, vm: &VM, _instr: u8) {
        vm.registers.acc.replace(vm.registers.acc_value() - 1);
    }

    fn setl(&'_ self, vm: &VM, _instr: u8) {
        vm.registers.lc.replace(vm.registers.acc_value());
    }

    fn back7(&'_ self, vm: &VM, _instr: u8) {
        vm.registers.lc.replace(vm.registers.lc_value() - 1);
        if vm.registers.lc_value() >= 0 {
            vm.registers.ip.replace(vm.registers.ip_value() - 7);
        }
    }
}

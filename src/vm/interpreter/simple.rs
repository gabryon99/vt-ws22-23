use crate::{vm::{
    opcode::{OpCode},
    VM,
}, measure_time};

use super::Interpreter;

pub struct SimpleInterpreter;

impl Interpreter for SimpleInterpreter {

    fn run(&self, vm: &VM) {
        
        let elapsed_time = measure_time!({
            loop {
                if vm.is_halt() {
                    break;
                }

                let instr = vm.running_program.data[vm.registers.ip_value() as usize];
                // println!("pc={}, acc={}, lc={}: {:?}", vm.registers.ip_value(), vm.registers.acc_value(), vm.registers.lc_value(), OpCode::try_from(instr).unwrap());

                match OpCode::try_from(instr).unwrap() {
                    OpCode::HALT => self.halt(vm, instr),
                    OpCode::CLRA => self.clra(vm, instr),
                    OpCode::INC3A => self.inc3a(vm, instr),
                    OpCode::DECA => self.deca(vm, instr),
                    OpCode::SETL => self.setl(vm, instr),
                    OpCode::BACK7 => self.back7(vm, instr),
                    _ => ()
                }
            }
        });

        vm.running_time.replace(elapsed_time);

    }

    fn halt(&self, vm: &VM, _instr: u8) {
        vm.halt.replace(true);
    }

    fn clra(&'_ self, vm: &VM, _instr: u8) {
        vm.registers.acc.replace(0);
        vm.registers.ip.replace(vm.registers.ip_value() + 1);
    }

    fn inc3a(&'_ self, vm: &VM, _instr: u8) {
        vm.registers.acc.replace(vm.registers.acc_value() + 3);
        vm.registers.ip.replace(vm.registers.ip_value() + 1);
    }

    fn deca(&'_ self, vm: &VM, _instr: u8) {
        vm.registers.acc.replace(vm.registers.acc_value() - 1);
        vm.registers.ip.replace(vm.registers.ip_value() + 1);
    }

    fn setl(&'_ self, vm: &VM, _instr: u8) {
        vm.registers.lc.replace(vm.registers.acc_value());
        vm.registers.ip.replace(vm.registers.ip_value() + 1);
    }

    fn back7(&'_ self, vm: &VM, _instr: u8) {
        vm.registers.lc.replace(vm.registers.lc_value() - 1);
        if vm.registers.lc_value() > 0 {
            vm.registers.ip.replace(vm.registers.ip_value() - 6);
        }
        else {
            vm.registers.ip.replace(vm.registers.ip_value() + 1);
        }
    }

    fn spill(&self, _vm: &VM, _instr: u8) {
        unreachable!()
    }
}

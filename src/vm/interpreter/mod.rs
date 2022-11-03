use super::VM;

pub mod simple;
pub mod threaded;

pub trait Interpreter {
    fn run(&self, vm: &mut VM);
    fn halt(&self, vm: &mut VM, instr: u8);
    fn clra(&self, vm: &mut VM, instr: u8);
    fn inc3a(&self, vm: &mut VM, instr: u8);
    fn deca(&self, vm: &mut VM, instr: u8);
    fn setl(&self, vm: &mut VM, instr: u8);
    fn back7(&self, vm: &mut VM, instr: u8);
}

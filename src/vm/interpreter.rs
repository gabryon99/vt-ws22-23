use super::VM;

pub mod jitted;
pub mod simple;

pub trait Interpreter {
    fn run(&self, vm: &VM);
    fn halt(&self, vm: &VM, instr: u8);
    fn clra(&self, vm: &VM, instr: u8);
    fn inc3a(&self, vm: &VM, instr: u8);
    fn deca(&self, vm: &VM, instr: u8);
    fn setl(&self, vm: &VM, instr: u8);
    fn back7(&self, vm: &VM, instr: u8);
    fn spill(&self, vm: &VM, instr: u8);
}

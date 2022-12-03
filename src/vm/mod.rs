pub mod opcode;
pub mod program;

mod interpreter;

use std::{cell::Cell, borrow::Borrow};

use self::interpreter::Interpreter;
use inkwell::context::Context;
use program::Program;

#[derive(Debug)]
pub enum RunningMode {
    Simple,
    Jitted,
}

#[derive(Debug)]
struct Registers {
    ip: Cell<u32>,  // Instruction Pointer
    acc: Cell<i32>, // Accumulator
    lc: Cell<i32>,  // Loop Counter
}

#[derive(Debug)]
pub struct VM {
    registers: Registers,
    running_program: Program,
    mode: RunningMode,
    halt: Cell<bool>,
}

impl std::fmt::Display for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ip: {}, acc: {}, lc: {}",
            self.registers.ip.get(), self.registers.acc.get(), self.registers.lc.get()
        )
    }
}

impl Registers {
    pub fn ip_value(&self) -> u32 { self.ip.get() }
    pub fn acc_value(&self) -> i32 { self.acc.get() }
    pub fn lc_value(&self) -> i32 { self.lc.get() }
}

impl VM {
    pub fn new(mode: RunningMode, running_program: Program) -> Self {
        Self {
            registers: Registers {
                ip: Cell::new(0),
                acc: Cell::new(running_program.initial_acc),
                lc: Cell::new(running_program.initial_lc),
            },
            halt: Cell::new(false),
            running_program,
            mode,
        }
    }

    fn is_halt(&self) -> bool {
        self.halt.borrow().get()
    }

    pub fn run(&self) {
        match self.mode {
            RunningMode::Simple => interpreter::simple::SimpleInterpreter {}.run(self),
            RunningMode::Jitted => {
                let ctx = Context::create();
                let jitted = interpreter::jitted::JittedInterpreter::new(ctx.borrow());
                jitted.run(self);
            }
        }
    }
}

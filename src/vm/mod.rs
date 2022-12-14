pub mod opcode;
pub mod program;
pub mod utils;

mod interpreter;

use std::{cell::Cell, borrow::Borrow, time::Duration};

use self::interpreter::Interpreter;
use inkwell::{context::Context, OptimizationLevel};
use program::Program;

#[derive(Debug, Clone, serde::Serialize)]
pub enum RunningMode {
    Simple,
    NoOptJitted,
    OptJitted
}

#[derive(Debug, PartialEq)]
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
    pub running_time: Cell<Duration>,
}

impl PartialEq for VM {
    fn eq(&self, other: &Self) -> bool {
        self.registers == other.registers
    }
}

impl std::fmt::Display for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VM(ip: {}, acc: {}, lc: {}, running_time: {:.2?})",
            self.registers.ip.get(), self.registers.acc.get(), self.registers.lc.get(), self.running_time.get()
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
            running_time: Cell::new(Duration::new(0, 0)),
            running_program,
            mode,
        }
    }

    fn is_halt(&self) -> bool {
        self.halt.borrow().get()
    }

    pub fn run(&self) {

        match self.mode {
            RunningMode::Simple => {
                interpreter::simple::SimpleInterpreter {}.run(self);
            },
            RunningMode::NoOptJitted | RunningMode::OptJitted => {

                let opt_level = match self.mode {
                    RunningMode::NoOptJitted => OptimizationLevel::None,
                    RunningMode::OptJitted => OptimizationLevel::Default,
                    _ => unreachable!()
                };

                let ctx = Context::create();
                let jitted = interpreter::jitted::JittedInterpreter::new(ctx.borrow(), opt_level);
                jitted.run(self);
            }
        }
    }
}

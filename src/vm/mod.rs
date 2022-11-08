pub mod opcode;
pub mod program;

mod interpreter;

use self::interpreter::Interpreter;
use program::Program;

#[derive(Debug)]
pub enum RunningMode {
    Simple,
    Threaded,
}

#[derive(Debug)]
struct Registers {
    ip: u32,  // Instruction Pointer
    acc: i32, // Accumulator
    lc: i32,  // Loop Counter
}

#[derive(Debug)]
pub struct VM {
    registers: Registers,
    running_program: Program,
    mode: RunningMode,
    halt: bool,
}

impl std::fmt::Display for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ip: {}, acc: {}, lc: {}",
            self.registers.ip, self.registers.acc, self.registers.lc
        )
    }
}

impl VM {
    pub fn new(mode: RunningMode, running_program: Program) -> Self {
        Self {
            registers: Registers {
                ip: 0,
                acc: running_program.initial_acc,
                lc: running_program.initial_lc,
            },
            halt: false,
            running_program,
            mode,
        }
    }

    pub fn run(&mut self) {
        match self.mode {
            RunningMode::Simple => interpreter::simple::SimpleInterpreter {}.run(self),
            RunningMode::Threaded => todo!(),
        }
    }
}

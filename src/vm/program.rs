use std::{path::PathBuf, collections::VecDeque};

use byteorder::{LittleEndian, ReadBytesExt};

use super::opcode::OpCode;

type Instruction = u8;
type BasicBlock = Vec<Instruction>;

#[derive(Debug, Clone)]
pub struct Program {
    pub data: Vec<Instruction>,
    pub initial_acc: i32,
    pub initial_lc: i32,
    pub filename: Option<String>,
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let no_file = "<no-file>".to_string();
        write!(
            f,
            "Program(filename: `{}`, ACC: {}, LC: {}, Size: {}, Code: [",
            self.filename.as_ref().unwrap_or(&no_file),
            self.initial_acc,
            self.initial_lc,
            self.data.len()
        )
        .unwrap();

        // Print actual code
        if self.data.len() <= 128 {
            write!(f, "[\n").unwrap();
            for (index, instr) in self.data.iter().enumerate() {
                write!(f, "\t{}: {}\n", index + 1, OpCode::try_from(*instr).unwrap()).unwrap();
            }
            write!(f, "])").unwrap();
        }
        else {
            write!(f, "...])").unwrap();
        }

        Ok(())
    }
}

impl Program {
    pub fn new(data: Vec<u8>, initial_acc: i32, initial_lc: i32) -> Self {
        Program {
            data,
            initial_acc,
            initial_lc,
            filename: None,
        }
    }

    pub fn read_from_file(path: PathBuf) -> Self {
        let filename = path.to_str().unwrap().to_string();

        match std::fs::read(path) {
            Ok(file_content) => {
                // The first 4 bytes are the initial value of ACCUMULATOR register
                let mut r_a = &file_content[0..4];
                let initial_acc = r_a.read_i32::<LittleEndian>().unwrap();
                // The second 4 bytes are the initial value of LOOP_COUNTER register
                let mut r_lc = &file_content[4..8];
                let initial_lc = r_lc.read_i32::<LittleEndian>().unwrap();

                let data = file_content[8..].to_vec();
                Program {
                    data,
                    initial_acc,
                    initial_lc,
                    filename: Some(filename),
                }
            }
            Err(e) => {
                eprintln!(
                    "[error] :: an error occurred when reading the program: {}",
                    e
                );
                std::process::exit(-1);
            }
        }
    }

    pub fn build_basic_blocks(&self) -> Vec<BasicBlock> {

        let mut basic_blocks: Vec<BasicBlock> = vec![];
        let mut current_block = vec![];

        // Inject SPILL instructions before Basic Block transformation
        let mut data = VecDeque::new();
        let mut spills = 0;

        for i in 0..self.data.len() {
            
            let instruction = self.data[i];
            data.push_back(instruction);

            match OpCode::try_from(instruction).unwrap() {
                OpCode::BACK7 => {
                    // Inject SPILL 
                    data.insert((i + spills) - 6, OpCode::SPILL.into());
                    spills += 1;
                }
                _ => ()
            }
        }

        for instruction in data {

            current_block.push(instruction);

            match OpCode::try_from(instruction).unwrap() {
                OpCode::BACK7 | OpCode::SPILL => {
                    basic_blocks.push(current_block);
                    current_block = vec![];
                }
                _ => ()
            }
        }

        basic_blocks.push(current_block);

        basic_blocks
    }

}

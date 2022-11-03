use std::path::PathBuf;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::vm::opcode::OpCode;

#[derive(Debug)]
pub struct Program {
    pub data: Vec<u8>,
    pub initial_acc: i32,
    pub initial_lc: i32,
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut i = 1;
        for instr in &self.data {
            write!(f, "{:#06x}:\t{}\n", i, OpCode::from(*instr)).unwrap();
            i += 1;
        }
        write!(
            f,
            "---\nACC: {}, LC: {}, Size: {}\n",
            self.initial_acc,
            self.initial_lc,
            self.data.len()
        )
        .unwrap();
        Ok(())
    }
}

impl Program {
    pub fn read_from_file(path: PathBuf) -> Self {
        if let Ok(file_content) = std::fs::read(path) {
            // The first 4 bytes are the initial value of ACCUMULATOR register
            let mut r_a = &file_content[0..4];
            let initial_acc = r_a.read_i32::<LittleEndian>().unwrap();
            // The second 4 bytes are the initial value of LOOP_COUNTER register
            let mut r_lc = &file_content[4..8];
            let initial_lc = r_lc.read_i32::<LittleEndian>().unwrap();

            let data = file_content[8..].to_vec();
            return Program {
                data,
                initial_acc,
                initial_lc,
            };
        }

        Program {
            data: vec![],
            initial_acc: 0,
            initial_lc: 0,
        }
    }
}

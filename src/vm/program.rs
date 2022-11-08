use std::path::PathBuf;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub struct Program {
    pub data: Vec<u8>,
    pub initial_acc: i32,
    pub initial_lc: i32,
    pub filename: Option<String>,
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let no_file = "<no-file>".to_string();
        write!(
            f,
            "Program(filename: `{}`, ACC: {}, LC: {}, Size: {})",
            self.filename.as_ref().unwrap_or(&no_file),
            self.initial_acc,
            self.initial_lc,
            self.data.len()
        )
        .unwrap();
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
                return Program {
                    data,
                    initial_acc,
                    initial_lc,
                    filename: Some(filename),
                };
            }
            Err(e) => {
                eprintln!(
                    "[error] :: an error occurred when reading the program: {}",
                    e.to_string()
                );
                std::process::exit(-1);
            }
        }
    }
}

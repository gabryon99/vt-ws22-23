use std::path::PathBuf;

use clap::Parser;
use vm::VM;

pub mod vm;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[clap(short, long)]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();
    // Load program from the command line
    let prog = vm::program::Program::read_from_file(args.path);
    println!("{}", prog);

    // Execute the program on a simple VM
    let vm = VM::new(vm::RunningMode::Jitted, prog);
    println!("[info] :: Before execution -> {}", vm);
    vm.run();
    println!("[info] :: After execution -> {}", vm);
}

use lox_bc::vm::{Vm, VmError};
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} run <filename>", args[0]);
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "run" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });
            let mut vm = Vm::new();

            match vm.interpret(&file_contents) {
                Ok(_) => {}
                Err(VmError::CompileError(_compile_error)) => {
                    std::process::exit(65);
                }
                Err(VmError::RuntimeError(_runtime_error)) => {
                    std::process::exit(70);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}

#![allow(unused)]
mod vm;

//
use std::{env, fmt::Error, io::Write, time::Instant};
use vm::opcodes::{OpCode, print_op_from_iter};
use crate::vm::{evaluate, parser::{FileParsingError, parse_file}};

struct ProgramSettings {
    pub display_runtime_time: bool,
}

impl ProgramSettings {
    pub fn new() -> Self {
        ProgramSettings { display_runtime_time: false }
    }
}

fn parse_instruction(instruction_list: &Vec<u8>, index: &mut usize) -> Result<(), Error> {
    let value = match instruction_list.get(*index) {
        Some(val) => val.clone(),
        None => return Err(Error),
    };

    let res = OpCode::try_from(value);

    match res {
        Ok(operator) => {
            *index += 1;
            print_op_from_iter(operator, instruction_list, index);
            
            Ok(())
        },
        Err(_) => {
            Err(Error)
        },
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        let mut new_program_settings = ProgramSettings::new();
        let file_input: String = match args.iter().nth(1) {
            Some(file_name) => file_name.to_owned(),
            None => return,
        };

        for parameter in args {
            match parameter.as_str() {
                "--time" => new_program_settings.display_runtime_time = true,
                _=>{},
            }
        }
        
        let mut program = match parse_file(file_input.to_owned()) {
            Ok(program_instance) => program_instance,
            Err(err) => {
                match err {
                    FileParsingError::FileError(new_err) => println!("File parsing error: {:?}", new_err),
                    t => println!("Program error: {:?}", t),
                }

                return;
            }
        };

        let start = Instant::now();

        if let Err(er) = evaluate::evaluate(&mut program) {
            println!("\x1b[0;31m[Runtime Error]\x1b[0m Error when evaluating program:\n> \x1b[0;31m{:?}\x1b[0m", er);
        }

        match program.std_out.flush() {
            Ok(_) => (),
            Err(e) => println!("Error when flushing to stdout: {}", e),
        }

        let duration = start.elapsed();

        if new_program_settings.display_runtime_time {
            println!("Program ran in: {}us", duration.as_secs_f64() * 1000000.0);
        }
    }
}

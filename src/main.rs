#![allow(unused)]
mod vm;

//
use std::{env, fmt::Error};
use vm::opcodes::{OpCode, print_op_from_iter};
use crate::vm::{evaluate, parser::{FileParsingError, parse_file}};


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
        let file_input: String = match args.iter().nth(1) {
            Some(file_name) => file_name.to_owned(),
            None => return,
        };
        
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

        if let Err(er) = evaluate::evaluate(&mut program) {
            println!("\x1b[0;31m[Runtime Error]\x1b[0m Error when evaluating program:\n> \x1b[0;31m{:?}\x1b[0m", er);
        }
    }
}

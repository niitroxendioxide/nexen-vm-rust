// #![allow(unused)]
mod vm;

//
use std::{env, fmt::Error, io::Write, time::Instant};
use vm::opcodes::{OpCode, print_op_from_iter};
use crate::vm::{evaluate, parser::{FileParsingError, parse_file}, program::Constant};

struct ProgramSettings {
    pub display_runtime_time: bool,
    pub include_flush_time: bool,
    pub preview_bytecode: bool,
}

impl ProgramSettings {
    pub fn new() -> Self {
        ProgramSettings { display_runtime_time: false, preview_bytecode: false, include_flush_time: true }
    }
}

fn parse_instruction(instruction_list: &Vec<u8>, index: &mut i64) -> Result<(), Error> {
    let value = match instruction_list.get(*index as usize) {
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

        let mut iterator = args.iter().peekable();

        while let Some(parameter) = iterator.next() {
            match parameter.as_str() {
                "--time" | "-t" => {
                    new_program_settings.display_runtime_time = true;

                    if let Some(next_param) = iterator.peek() {
                        if next_param.as_str() == "noflush" {
                            iterator.next(); 
                            new_program_settings.include_flush_time = false; 
                        }
                    }
                },
                "--bytes" | "-b" => new_program_settings.preview_bytecode = true,
                _ => {},
            }
}
        
        let mut program = match parse_file(file_input.to_owned()) {
            Ok(program_instance) => program_instance,
            Err(err) => {
                match err {
                    FileParsingError::FileError(new_err) => println!("\n\x1b[1;31m[File Error]\x1b[0m: Error when parsing .nxo file, trace:\n> {:?}", new_err),
                    t => println!("\n\x1b[1;31m[File Error]\x1b[0m: Error when parsing .nxo file, trace:\n> {}", t),
                }

                return;
            }
        };

        if new_program_settings.preview_bytecode {
            //let mut idx = 0;
            let mut fn_idx = 0;
            for function in &program.functions {
                if let Constant::FunctionConstant(body) = function {
                    println!("\n__function_F{fn_idx}:");
                    let mut idx = 0;
                    loop {
                        if let Err(e) = parse_instruction(&body.instructions, &mut idx) {
                            println!("Error reading bytecode: {}", e);
                            break;
                        }

                        if (idx as usize) >= body.instructions.len() {
                            break;
                        }
                    }
                }
                fn_idx += 1;
            }

            println!("\n__start:");
            let callframe = match program.get_call_frame_mut() {
                Ok(v) => v,
                Err(_) => {println!("Bytecode preview is not available"); return;},
            };
            loop {
                if let Err(e) = parse_instruction(&callframe.instructions, &mut callframe.program_counter) {
                    println!("Error reading bytecode: {}", e);
                    break;
                };

                if (callframe.program_counter as usize) >= callframe.instructions.len() {
                    break;
                }
            }

            return;
        }

        let start = Instant::now();

        if let Err(er) = evaluate::evaluate(&mut program) {
            println!("\x1b[0;31m[Runtime Error]\x1b[0m Error when evaluating program:\n\x1b[1;31m> {}\x1b[0m", er);
            if let Ok(cf) = program.get_call_frame_mut() {
                let begin_progc = cf.program_counter;
                match cf.instructions.get(cf.program_counter as usize) {
                    Some(instr) => {
                        if let Ok(op) = OpCode::try_from(*instr) {
                            cf.program_counter += 1;
                            print!("> \x1b[3;30mOn Line:\x1b[0m\n|-> ");
                            print_op_from_iter(op, &cf.instructions, &mut cf.program_counter)
                        }
                    },
                    None => (),
                };

                if cf.function_id >= 0 {
                    println!("|-> In function F{}", cf.function_id);
                }

                println!("|-> Program pointer at: {}", begin_progc);
                println!("> \x1b[3;30mRegisters:\x1b[0m");

                for (idx, value) in program.core_module.registers.iter().enumerate() {
                    if let crate::vm::program::Value::Nil = value {
                        continue;
                    }

                    println!("|-> Register {}: {}", idx, value);
                }

                println!("")
                //println!("> Registers used: {}", program.registers.len());
            }
        }

        let mut duration = start.elapsed();

        match program.std_out.flush() {
            Ok(_) => (),
            Err(e) => println!("Error when flushing to stdout: {}", e),
        }

        if new_program_settings.include_flush_time {
            duration = start.elapsed();
        }
        

        if new_program_settings.display_runtime_time {
            println!("\n\x1b[1;32m[Nexen]\x1b[0m Program ran in: \x1b[1;32m{}us\x1b[0m \x1b[3;36m(microseconds)\x1b[0m", duration.as_secs_f64() * 1000000.0);
        }
    }
}

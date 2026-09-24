use std::{cell::RefCell, fmt::Display, io::Error as IoError, rc::Rc};

use super::modules::{Module};

use super::program::{Program, Constant, FunctionBody};

static LANG_SIGNATURE: u32 = 0x6E786F21;
static LANG_BEGIN: u16 = 0xFFFF;

type ReadRange = std::ops::Range<usize>;

#[derive(Debug)]
pub enum FileParsingError {
    FileError(IoError),
    InvalidCompiledFile,
    NotEnoughData,
    ProgramHasNoEntry,
    InvalidConstantDefined(u8, usize),
    InvalidDeclaredFunction(String),
    InvalidDeclaredModule(String),
}

impl From<IoError> for FileParsingError {
    fn from(err: IoError) -> Self {
        FileParsingError::FileError(err)
    }
}

impl Display for FileParsingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileParsingError::FileError(er) => write!(formatter, "FileError: {:?}", er),
            FileParsingError::InvalidDeclaredFunction(reason) => write!(formatter, "InvalidDeclaredFunction:\n|-> {}", reason),
            FileParsingError::InvalidConstantDefined(byte, idx) => write!(formatter, "InvalidConstantDefined:\n|-> [0x{}], [ptr: {}]", byte, idx),
            FileParsingError::InvalidDeclaredModule(text) => write!(formatter, "InvalidModuleDefined:\n|-> {}", text),
            generic_error => write!(formatter, "{:?}", generic_error),
        }
    }
}

fn read_u32_le(bytes: &[u8], range: ReadRange) -> Result<u32, FileParsingError> {
    let slice = bytes.get(range).ok_or(FileParsingError::NotEnoughData)?;
    let array: [u8; 4] = slice.try_into().map_err(|_| FileParsingError::NotEnoughData)?;
    Ok(u32::from_le_bytes(array))
}

fn read_f64_le(bytes: &[u8], range: ReadRange) -> Result<f64, FileParsingError> {
    let slice = bytes.get(range).ok_or(FileParsingError::NotEnoughData)?;
    let array: [u8; 8] = slice.try_into().map_err(|_| FileParsingError::NotEnoughData)?;
    Ok(f64::from_le_bytes(array))
}

fn read_u16_le(bytes: &[u8], range: ReadRange) -> Result<u16, FileParsingError> {
    let slice = bytes.get(range).ok_or(FileParsingError::NotEnoughData)?;
    let array: [u8; 2] = slice.try_into().map_err(|_| FileParsingError::NotEnoughData)?;
    Ok(u16::from_le_bytes(array))
}

pub fn parse_file(file_name: String) -> Result<Program, FileParsingError> {
    let read_bytes = std::fs::read(file_name)?;

    let magic_constant = read_u32_le(&read_bytes, 0..4)?;
    if magic_constant != LANG_SIGNATURE {
        return Err(FileParsingError::InvalidCompiledFile);
    }

    let version_major = read_u16_le(&read_bytes, 4..6)?;
    let version_minor = read_u16_le(&read_bytes, 6..8)?;
    let version_patch = read_u16_le(&read_bytes, 8..10)?;
    let _program_size = read_u32_le(&read_bytes, 10..14)?;
    let constant_count = read_u32_le(&read_bytes, 14..18)?;
    let function_count = read_u32_le(&read_bytes, 18..22)?;
    let _ = match read_bytes.get(22) {
        Some(v) => *v,
        None => return Err(FileParsingError::NotEnoughData),
    };

    let modules_compiled = read_u32_le(&read_bytes, 23..27)?;
    //println!("Modules compiled: {modules_compiled}");

    let mut idx = 27;
    let mut constant_index = 0;
    let mut function_index = 0;
    let mut module_index = 0;
    let mut constants: Vec<Constant> = Vec::new();
    let mut functions: Vec<Constant> = Vec::new();

    #[allow(unused)]
    let mut modules: Vec<Rc<Module>> = Vec::new();
    let mut program_instructions = Vec::new();

    fn parse_constant(cur_byte: &u8, read_bytes: &Vec<u8>, idx: &mut usize, constants: &mut Vec<Constant>) -> Result<(), FileParsingError> {
        match *cur_byte {
            0x00 => {
                *idx += 1;
                let next_byte = match read_bytes.get(*idx) {
                    Some(byte) => *byte == 0x0,
                    None => return Err( FileParsingError::NotEnoughData )
                };
                *idx += 1;

                let new_val = Constant::BoolConstant(!next_byte);
                constants.push(new_val);
                Ok(())
            },

            0x01 => {
                *idx += 1;
                let value = read_f64_le(&read_bytes, *idx..*idx + 8)?;
                *idx += 8;
                constants.push(Constant::NumberConstant(value));
                Ok(())
            },
            0x02 => {
                *idx += 1;

                let str_len = read_u32_le(&read_bytes, *idx..*idx + 4)?;
                // println!("reading string at idx: {}, with len: {str_len}", *idx-1);
                *idx += 4;

                let end_idx = ((*idx as u32) + str_len) as usize;
                let str_slice = read_bytes.get(*idx..end_idx)
                    .ok_or(FileParsingError::InvalidConstantDefined(*cur_byte, *idx))?;

                constants.push(Constant::from(str_slice));
                *idx += str_len as usize;
                
                Ok(())
            },
            0x04 => {
                *idx += 1;
                let arr_len = read_u32_le(&read_bytes, *idx..*idx + 4)? as usize;
                *idx += 4;

                let mut array_vals: Vec<Constant> = Vec::with_capacity(arr_len);
                for _ in 0..arr_len {
                    let cur_byte = read_bytes.get(*idx).ok_or( FileParsingError::NotEnoughData )?;
                    parse_constant(&cur_byte, &read_bytes, &mut *idx, &mut array_vals)?;
                }

                let used_rcrefcell = Rc::from(RefCell::from(array_vals));
                constants.push(Constant::ArrayConstant(used_rcrefcell));

                Ok(())
            },
            0x07 | 0x08 => {
                *idx += 1;
                let next_byte = match read_bytes.get(*idx) {
                    Some(byte) => *byte,
                    None => return Err( FileParsingError::NotEnoughData )
                };
                *idx += 1;

                if *cur_byte == 0x08 {
                    constants.push(Constant::RegisterRefConstant(next_byte));
                } else {
                    constants.push(Constant::StringRefConstant(next_byte));
                }
                Ok(())
            }
            _ => return Err( FileParsingError::InvalidConstantDefined(*cur_byte, *idx) ),
        }
    }

    fn parse_function(cur_byte: &u8, read_bytes: &Vec<u8>, idx: &mut usize, functions: &mut Vec<Constant>) -> Result<(), FileParsingError> {
        if *cur_byte != 0x03 {
            let str = format!("Function tag is incorrect ({cur_byte}). At index: {idx}");
            return Err( FileParsingError::InvalidDeclaredFunction(str) )
        }

        *idx += 1;
        let fn_len = read_u32_le(&read_bytes, *idx..*idx+4).map_err(|_| FileParsingError::InvalidDeclaredFunction("Function has no length".to_string()) )? as usize;
        *idx += 4;
        let arg_count = read_bytes.get(*idx).ok_or(FileParsingError::InvalidDeclaredFunction("Argument count is not specified".to_string()))?;
        *idx += 1;
        let reg_count = read_bytes.get(*idx).ok_or(FileParsingError::InvalidDeclaredFunction("Function has no register count specified".to_string()))?;
        *idx += 1;

        let end_idx = *idx + fn_len;
        let fn_slice = match read_bytes.get(*idx..end_idx) {
            Some(slice) => slice,
            None => {
                let len = read_bytes.len();
                let str = format!("Function length outside of range, length: {fn_len}, from {idx} to {end_idx}, byte length: {len}");
                return Err( FileParsingError::InvalidDeclaredFunction(str) );
            }
        };//[idx..end_idx];

        let new_fn = FunctionBody::new(fn_len as u32, *arg_count, *reg_count, &fn_slice);
        functions.push(Constant::FunctionConstant(new_fn));
        
        *idx += fn_len;
        Ok(())
    }


    /*
        parse file contents!
     */
    loop {
        let cur_byte = read_bytes.get(idx).ok_or( FileParsingError::NotEnoughData )?;
        if constant_index < constant_count && constant_count > 0 {
            match parse_constant(cur_byte, &read_bytes, &mut idx, &mut constants) {
                Ok(_) => {constant_index += 1; continue},
                Err(e) => return Err(e),
            }
        }

        if function_index < function_count && function_count > 0 {
            match parse_function(cur_byte, &read_bytes, &mut idx, &mut functions) {
                Ok(_) => {function_index += 1; continue},
                Err(e) => return Err(e),
            }
        }

        if module_index < modules_compiled && modules_compiled > 0 {
            if *cur_byte != 0x09 {
                let str = format!("Module Tag is incorrect or missing ({cur_byte}). At index: {idx}");
                return Err( FileParsingError::InvalidDeclaredModule(str) )
            }

            idx += 1;
            let export_count = *read_bytes.get(idx)
                .ok_or(FileParsingError::InvalidDeclaredModule("Argument count is not specified".to_string()) )? as usize;

            idx += 1;
            let module_size = read_u32_le(&read_bytes, idx..idx+4)
                .map_err(|_| FileParsingError::InvalidDeclaredModule("Module has no size".to_string()) )? as usize;
            
            idx += 4;
            let func_count = read_u32_le(&read_bytes, idx..idx+4)
                .map_err(|_| FileParsingError::InvalidDeclaredModule("Module has no function count".to_string()) )? as usize;

            idx += 4;
            println!("Module: {module_index} has {export_count} exports, {func_count} functions & {module_size} bytes of __start");

            let mut reg_vec: Vec<u8> = Vec::with_capacity(export_count);
            for exp_idx in 0..export_count {
                let reg = *read_bytes.get(idx + (exp_idx as usize) + 1)
                    .ok_or(FileParsingError::InvalidDeclaredModule(
                        format!("No value for export EX{}", idx + exp_idx as usize),
                    ) )?;

                reg_vec.push(reg);
                idx += 2;
            }

            //let mut mod_func_idx = 0;
            let mut module_functions: Vec<Constant> = Vec::with_capacity(func_count);
            for _ in 0..func_count {
                let cur_byte = read_bytes.get(idx).ok_or( FileParsingError::NotEnoughData )?;
                match parse_function(cur_byte, &read_bytes, &mut idx, &mut module_functions) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
            }

            let module_body: Vec<u8> = match read_bytes.get(idx.. idx + module_size) {
                Some(slice) => slice.into(),
                None => {
                    let len = read_bytes.len();
                    let str = format!(
                        "Module body outside of range, reading {module_size} bytes, from {idx} to {}, actual byte length: {len}", 
                        idx + module_size
                    );
                    return Err( FileParsingError::InvalidDeclaredModule(str) );
                }
            };
            
            idx += module_size;
            modules.push(Rc::from(Module::new(reg_vec, Rc::from(module_body), module_functions)));

            module_index+=1;
            continue;
        }
        
        let next_two = read_u16_le(&read_bytes, idx..idx+2)?;
        if next_two == LANG_BEGIN {
            let start = idx+2;

            /*for cons in &constants {
                println!("{cons:?}");
            } */

            program_instructions.extend_from_slice(&read_bytes[start..]);

            break;
        }

        for i in 0..function_index {
            if let Some(v) = functions.get(i as usize) {
                println!("{}", v);   
            }
        }

        return Err( FileParsingError::ProgramHasNoEntry )
    }

    let new_program = Program::new(
        version_major, 
        version_minor, 
        version_patch, 
        Rc::new(program_instructions), 
        functions, 
        constants, 
        modules
    );

    Ok(new_program)
}
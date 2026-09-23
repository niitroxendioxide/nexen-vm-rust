use std::{cell::RefCell, fmt::Display, io::Error as IoError, rc::Rc};

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
            FileParsingError::InvalidDeclaredFunction(reason) => write!(formatter, "InvalidDeclaredFunction: {}", reason),
            FileParsingError::InvalidConstantDefined(byte, idx) => write!(formatter, "InvalidConstantDefined: [0x{}], [ptr: {}]", byte, idx),
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
    let registers_used = match read_bytes.get(22) {
        Some(v) => *v,
        None => return Err(FileParsingError::NotEnoughData),
    };

    let mut idx = 23;
    let mut constant_index = 0;
    let mut function_index = 0;
    let mut constants: Vec<Constant> = Vec::new();
    let mut functions: Vec<Constant> = Vec::new();
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
                *idx += 4;

                let end_idx = ((*idx as u32) + str_len) as usize;
                let str_slice = &read_bytes[*idx..end_idx];
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

    loop {
        let cur_byte = read_bytes.get(idx).ok_or( FileParsingError::NotEnoughData )?;
        if constant_index < constant_count && constant_count > 0 {
            match parse_constant(cur_byte, &read_bytes, &mut idx, &mut constants) {
                Ok(_) => {constant_index += 1; continue},
                Err(e) => return Err(e),
            }
        }

        if function_index < function_count && function_count > 0 {
            if *cur_byte != 0x03 {
                println!("Data: c_count{constant_index}, c_given{constant_count}");
                let str = format!("Function tag is incorrect ({cur_byte}). At index: {idx}");
                return Err( FileParsingError::InvalidDeclaredFunction(str) )
            }

            idx += 1;
            let fn_len = read_u32_le(&read_bytes, idx..idx+4).map_err(|_| FileParsingError::InvalidDeclaredFunction("Function has no length".to_string()) )? as usize;
            idx += 4;
            let arg_count = read_bytes.get(idx).ok_or(FileParsingError::InvalidDeclaredFunction("Argument count is not specified".to_string()))?;
            idx += 1;
            let reg_count = read_bytes.get(idx).ok_or(FileParsingError::InvalidDeclaredFunction("Function has no register count specified".to_string()))?;
            idx += 1;

            let end_idx = idx + fn_len;
            let fn_slice = match read_bytes.get(idx..end_idx) {
                Some(slice) => slice,
                None => {
                    let len = read_bytes.len();
                    let str = format!("Function length outside of range, length: {fn_len}, from {idx} to {end_idx}, byte length: {len}");
                    return Err( FileParsingError::InvalidDeclaredFunction(str) );
                }
            };//[idx..end_idx];

            let new_fn = FunctionBody::new(fn_len as u32, *arg_count, *reg_count, &fn_slice);
            functions.push(Constant::FunctionConstant(new_fn));
            
            function_index += 1;
            idx += fn_len;

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

    let new_program = Program::new(version_major, version_minor, version_patch, Rc::new(program_instructions), functions, constants, registers_used);

    Ok(new_program)
}
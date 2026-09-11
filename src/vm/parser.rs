use std::io::{Error as IoError};

use super::program::{Program, Constant};

static LANG_SIGNATURE: u32 = 0x6E786F21;
static LANG_BEGIN: u16 = 0xFFFF;

type ReadRange = std::ops::Range<usize>;

#[derive(Debug)]
pub enum FileParsingError {
    FileError(IoError),
    InvalidCompiledFile,
    NotEnoughData,
    ProgramHasNoEntry,
    InvalidConstantDefined,
}

impl From<IoError> for FileParsingError {
    fn from(err: IoError) -> Self {
        FileParsingError::FileError(err)
    }
}

fn read_u32_le(bytes: &[u8], range: ReadRange) -> Result<u32, FileParsingError> {
    let slice = bytes.get(range).ok_or(FileParsingError::NotEnoughData)?;
    let array: [u8; 4] = slice.try_into().map_err(|_| FileParsingError::NotEnoughData)?;
    Ok(u32::from_le_bytes(array))
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
    let _program_size = read_u32_le(&read_bytes, 8..12)?;
    let constant_count = read_u32_le(&read_bytes, 12..16)?;

    let mut idx = 16;
    let mut constant_index = 0;
    let mut constants: Vec<Constant> = Vec::new();
    let mut program_instructions = Vec::new();

    loop {
        if constant_index < constant_count && constant_count > 0 {
            if let Some(byte) = read_bytes.get(idx) {
                if *byte != 0x02 {
                    break;
                } 

                idx += 1;

                let str_len = read_u32_le(&read_bytes, idx..idx + 4)?;
                idx += 4;

                let end_idx = ((idx as u32) + str_len) as usize;
                let str_slice = &read_bytes[idx..end_idx];
                constants.push(Constant::from(str_slice));

                idx += (str_len - 1) as usize;
            } else {
                return Err( FileParsingError::InvalidConstantDefined );
            }

            constant_index += 1;
            idx += 1;
            
            continue;
        };
        
        let next_two = read_u16_le(&read_bytes, idx..idx+2)?;
        if next_two == LANG_BEGIN {
            let start = idx+2;
            program_instructions.extend_from_slice(&read_bytes[start..]);

            break;
        }

        return Err( FileParsingError::ProgramHasNoEntry )
    }

    let new_program = Program::new(version_major, version_minor, program_instructions, constants);
    //println!("{}", new_program);

    Ok(new_program)
}
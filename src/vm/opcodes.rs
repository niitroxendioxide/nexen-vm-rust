use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    OpVoid = 0x00,
    OpLoadConst = 0x01,
    OpStoreLocal = 0x02,
    OpLoadLocal = 0x03,
    OpAdd = 0x04,
    OpSub = 0x05,
    OpMul = 0x06,
    OpDiv = 0x07,
    OpPushNum = 0x08,
    OpPushU16 = 0x09,
    OpPushU8 = 0x0A,
    OpPush1 = 0x0B,
    OpPush0 = 0x0C,
    OpPushScope = 0x0D,
    OpPopScope = 0x0E,
    OpJump = 0x0F,
    OpEq = 0x10,
    OpNotEq = 0x11,
    OpLessThan = 0x12,
    OpGreaterThan = 0x13,
    OpLessEqualThan = 0x14,
    OpGreaterEqualThan = 0x15,
    OpJumpIfTrue = 0x16,
    OpJumpIfFalse = 0x17,
    OpFunctionCall = 0x18,
    OpReturn = 0x19,
    OpNativeFnCall = 0x1A,
}

#[derive(Debug)]
pub struct OpInvalid;

impl TryFrom<u8> for OpCode {
    type Error = OpInvalid;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(OpCode::OpVoid),
            0x01 => Ok(OpCode::OpLoadConst),
            0x02 => Ok(OpCode::OpStoreLocal),
            0x03 => Ok(OpCode::OpLoadLocal),
            0x04 => Ok(OpCode::OpAdd),
            0x05 => Ok(OpCode::OpSub),
            0x06 => Ok(OpCode::OpMul),
            0x07 => Ok(OpCode::OpDiv),
            0x08 => Ok(OpCode::OpPushNum),
            0x09 => Ok(OpCode::OpPushU16),
            0x0A => Ok(OpCode::OpPushU8),
            0x0B => Ok(OpCode::OpPush1),
            0x0C => Ok(OpCode::OpPush0),
            0x0D => Ok(OpCode::OpPushScope),
            0x0E => Ok(OpCode::OpPopScope),
            0x0F => Ok(OpCode::OpJump),
            0x10 => Ok(OpCode::OpEq),
            0x11 => Ok(OpCode::OpNotEq),
            0x12 => Ok(OpCode::OpLessThan),
            0x13 => Ok(OpCode::OpGreaterThan),
            0x14 => Ok(OpCode::OpLessEqualThan),
            0x15 => Ok(OpCode::OpGreaterEqualThan),
            0x16 => Ok(OpCode::OpJumpIfTrue),
            0x17 => Ok(OpCode::OpJumpIfFalse),
            0x18 => Ok(OpCode::OpFunctionCall),
            0x19 => Ok(OpCode::OpReturn),
            0x1A => Ok(OpCode::OpNativeFnCall),
            _ => Err(OpInvalid)
        }
    }
}

pub fn print_op_from_iter(operator: OpCode, instruction_list: &Vec<u8>, index: &mut usize) {
    match operator {
        OpCode::OpPush0 | OpCode::OpPush1 | OpCode::OpAdd | OpCode::OpSub | OpCode::OpDiv | OpCode::OpMul 
        | OpCode::OpEq | OpCode::OpNotEq | OpCode::OpGreaterEqualThan | OpCode::OpGreaterThan 
        | OpCode::OpLessEqualThan | OpCode::OpLessThan | OpCode::OpPushScope | OpCode::OpPopScope | OpCode::OpVoid | OpCode::OpReturn => {
            println!("{:?}", operator);
        },

        // 1 byte ahead
        OpCode::OpPushU8 | OpCode::OpLoadLocal | OpCode::OpLoadConst | OpCode::OpStoreLocal | OpCode::OpFunctionCall => {
            let next_val = match instruction_list.get(*index) {
                Some(val) => val,
                None => return,
            };

            *index += 1;
            println!("{:?} {}", operator, next_val);
        },

        // 2 bytes ahead
        OpCode::OpPushU16 | OpCode::OpJump | OpCode::OpJumpIfFalse | OpCode::OpJumpIfTrue | OpCode::OpNativeFnCall => {
            let byte1 = match instruction_list.get(*index) {
                Some(val) => val,
                None => return,
            };

            *index += 1;
            let byte2 = match instruction_list.get(*index) {
                Some(val) => val,
                None => return,
            };

            *index += 1;

            if (operator == OpCode::OpNativeFnCall) {
                println!("{:?} {}, {}", operator, *byte1, *byte2);
            } else {
                let new_val = u16::from_le_bytes([*byte1, *byte2]);
                println!("{:?} {}", operator, new_val);
            }

        }, 

        // 8 bytes ahead
        OpCode::OpPushNum => {
            let mut new_vec: Vec<u8> = Vec::new();
            for _ in 0..8 {
                let byte1 = match instruction_list.get(*index) {
                    Some(val) => val,
                    None => return,
                };

                *index += 1;
                new_vec.push(*byte1);
            }

            let fixed_length_array: [u8; 8] = match new_vec.try_into() {
                Ok(fixed) => fixed,
                Err(_) => return,
            };

            let new_val = f64::from_le_bytes(fixed_length_array);
            println!("{:?} {}", operator, new_val);
        },
    }
}
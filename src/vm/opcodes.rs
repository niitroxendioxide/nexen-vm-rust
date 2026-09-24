use std::{convert::TryFrom, fmt::Display};

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
    //OpPushScope = 0x0D,
    //OpPopScope = 0x0E,
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
    OpPushArray = 0x1B,
    OpPushDict = 0x1C,
    OpDictSet = 0x1D,
    OpNewStruct = 0x1E,
    OpLoadField = 0x1F,
    OpLoadIndex = 0x20,
    OpLoadGlob = 0x21,
    OpLoadMod = 0x22,
    OpCallReg = 0x23,
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
            //0x0D => Ok(OpCode::OpPushScope),
            //0x0E => Ok(OpCode::OpPopScope),
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
            0x1B => Ok(OpCode::OpPushArray),
            0x1C => Ok(OpCode::OpPushDict),
            0x1D => Ok(OpCode::OpDictSet),
            0x1E => Ok(OpCode::OpNewStruct),
            0x1F => Ok(OpCode::OpLoadField),
            0x20 => Ok(OpCode::OpLoadIndex),
            0x21 => Ok(OpCode::OpLoadGlob),
            0x22 => Ok(OpCode::OpLoadMod),
            0x23 => Ok(OpCode::OpCallReg),
            // 0x21 => Ok(),
            _ => Err(OpInvalid)
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::OpVoid => write!(f, "NO_OP"),
            OpCode::OpAdd => write!(f, "ADD"),
            OpCode::OpSub => write!(f, "SUB"),
            OpCode::OpMul => write!(f, "MUL"),
            OpCode::OpDiv => write!(f, "DIV"),
            OpCode::OpEq => write!(f, "EQ"),
            OpCode::OpNotEq => write!(f, "NEQ"),
            OpCode::OpLessEqualThan => write!(f, "LEQT"),
            OpCode::OpGreaterEqualThan => write!(f, "GEQT"),
            OpCode::OpGreaterThan => write!(f, "GT"),
            OpCode::OpLessThan => write!(f, "LT"),
            OpCode::OpFunctionCall => write!(f, "CALL"),
            OpCode::OpLoadConst => write!(f, "LOAD_CONST"),
            OpCode::OpLoadLocal => write!(f, "LOAD_LOCAL"),
            OpCode::OpStoreLocal => write!(f, "STR"),
            OpCode::OpPush0 => write!(f, "PUSH_0"),
            OpCode::OpPush1 => write!(f, "PUSH_1"),
            OpCode::OpPushU8 => write!(f, "PUSH_U8"),
            OpCode::OpPushU16 => write!(f, "PUSH_U16"),
            OpCode::OpPushNum => write!(f, "PUSH_F64"),
            OpCode::OpReturn => write!(f, "RET"),
            OpCode::OpNativeFnCall => write!(f, "CALL"),
            OpCode::OpJump => write!(f, "JUMP"),
            OpCode::OpJumpIfFalse => write!(f, "JNEQ"),
            OpCode::OpJumpIfTrue => write!(f, "JEQ"),
            OpCode::OpLoadField => write!(f, "LOAD_FIELD"),
            OpCode::OpNewStruct => write!(f, "NEW_STRUCT"),
            OpCode::OpDictSet => write!(f, "DICT_SET"),
            OpCode::OpLoadIndex => write!(f, "LOAD_IDX"),
            OpCode::OpPushArray => write!(f, "PUSH_ARR"),
            OpCode::OpLoadGlob => write!(f, "LOAD_GLOB"),
            OpCode::OpLoadMod => write!(f, "LOAD_MOD"),
            OpCode::OpCallReg => write!(f, "CALL_REG"),

            #[allow(unreachable_patterns)]
            _ => write!(f, "{:?}", self),
        }
    }
}

pub fn print_op_from_iter(operator: OpCode, instruction_list: &Vec<u8>, index: &mut i64) {
    match operator {
        OpCode::OpVoid => println!("{}", operator),
        OpCode::OpPush0 | OpCode::OpPush1 | OpCode::OpCallReg | OpCode::OpReturn | OpCode::OpPushArray| OpCode::OpDictSet | OpCode::OpPushDict => {
            let reg1 = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;
            println!("\x1b[1;30m{}\x1b[0m R{}", operator, reg1);
        },

        OpCode::OpAdd | OpCode::OpSub | OpCode::OpDiv | OpCode::OpMul
        | OpCode::OpEq | OpCode::OpNotEq | OpCode::OpGreaterEqualThan | OpCode::OpGreaterThan 
        | OpCode::OpLessEqualThan | OpCode::OpLessThan => {
            let reg1 = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;
            
            let reg2 = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;

            let reg3 = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;
            println!("\x1b[1;30m{}\x1b[0m R{}, R{}, R{}", operator, reg1, reg2, reg3);
        }

        OpCode::OpNewStruct => {
            let register = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;

            let size = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;

            println!("\x1b[1;30m{}\x1b[0m R{}, [\x1b[1;33m{}\x1b[0m]", operator, register, size);
        },

        OpCode::OpLoadField => {
            let reg_dest = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;

            let field_reg = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;

            let field_id = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;

            println!("\x1b[1;30m{}\x1b[0m R{}, R{}, [\x1b[1;33m{}\x1b[0m]", operator, reg_dest, field_reg, field_id);
        },

        OpCode::OpLoadIndex => {
            let reg_dest = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;

            let field_reg = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;

            let field_id = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;

            println!("\x1b[1;30m{}\x1b[0m, R{}, R{}, R{}", operator, reg_dest, field_reg, field_id);
        },

        // 1 byte reg + 4 byte ahead
        OpCode::OpLoadMod => {
            let register = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;
            
            let next_val = match instruction_list.get(*index as usize .. *index as usize + 4) {
                Some(val) => u32::from_le_bytes(val.try_into().unwrap()),
                None => return,
            };

            *index += 4;
            println!("\x1b[1;30m{}\x1b[0m R{}, [\x1b[1;33m{}\x1b[0m]", operator, register, next_val);
        }

        // 1 byte reg + 1 byte
        OpCode::OpPushU8 | OpCode::OpLoadGlob | OpCode::OpLoadLocal | OpCode::OpLoadConst | OpCode::OpStoreLocal => {
            let register = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;
            
            let next_val = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;
            println!("\x1b[1;30m{}\x1b[0m R{}, [\x1b[1;33m{}\x1b[0m]", operator, register, next_val);
        },

        OpCode::OpNativeFnCall => {
            let reg = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;
            let b1 = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };
            *index += 1;

            let b2 = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };
            *index += 1;

            println!("\x1b[1;30m{}\x1b[0m R{}, [\x1b[1;33m{}\x1b[0m], [\x1b[1;33m{}\x1b[0m]", operator, reg, b1, b2);
        }

        OpCode::OpJump => {
            let range = *index as usize..*index as usize+4;
            let i32val = match instruction_list.get(range) {
                Some(val) => i32::from_le_bytes(val.try_into().unwrap()),
                None => return,
            }; 
            *index += 4;
            println!("\x1b[1;30m{}\x1b[0m [\x1b[1;33m{}\x1b[0m]", operator, i32val);
        }

        OpCode::OpJumpIfFalse | OpCode::OpJumpIfTrue => {
            let reg = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;
            let range = *index as usize..*index as usize+4;
            let i32val = match instruction_list.get(range) {
                Some(val) => i32::from_le_bytes(val.try_into().unwrap()),
                None => return,
            }; 

            *index += 4;
            println!("\x1b[1;30m{}\x1b[0m R{}, [\x1b[1;33m{}\x1b[0m]", operator, reg, i32val);
        }

        // 2 bytes ahead
        OpCode::OpPushU16 => {
            let reg = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };
            *index += 1;

            let byte1 = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;
            let byte2 = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;

            let new_val = u16::from_le_bytes([*byte1, *byte2]);
            println!("\x1b[1;30m{}\x1b[0m R{}, {}", operator, reg, new_val);
        }, 

        OpCode::OpFunctionCall => {
            let reg = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };

            *index += 1;
            let u32val = match instruction_list.get(*index as usize..*index as usize+4) {
                Some(val) => u32::from_le_bytes(val.try_into().unwrap()),
                None => return,
            }; 

            *index += 4;
            println!("\x1b[1;30m{}\x1b[0m R{}, F{}", operator, reg, u32val);
        }

        // 8 bytes ahead
        OpCode::OpPushNum => {
            let reg = match instruction_list.get(*index as usize) {
                Some(val) => val,
                None => return,
            };
            *index += 1;
            let mut new_vec: Vec<u8> = Vec::new();
            for _ in 0..8 {
                let byte1 = match instruction_list.get(*index as usize) {
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
            println!("\x1b[1;30m{}\x1b[0m R{}, {}", operator, reg, new_val);
        },
    }
}
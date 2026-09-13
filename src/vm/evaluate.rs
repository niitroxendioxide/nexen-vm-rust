use std::cell::RefCell;
use std::rc::Rc;

use crate::vm::program::{CallFrame, Constant, Value};
use crate::vm::stdlib;

use super::program::Program;
use super::opcodes::OpCode;

#[derive(Debug)]
pub enum EvaluateError {
    OutOfRange,
    InvalidOperation,
    NoValueProvided,
    StackEndReached,
    UndefinedConstant,
    UndefinedLocalVariable,
    UndefinedFunction,
    NoCallFrameAvailable,
    NotAFunction,
    CrossValueOperation,
    UnsupportedOperation
}

pub fn evaluate(program: &mut Program) -> Result<(), EvaluateError> {
    program.set_running(true);

    while (program.is_running()) {
        let call_frame_idx = program.get_call_frame_idx();
        let current_byte = match program.get_call_frame_mut()?.advance() {
            Some(val) => val,
            None => {
                program.set_running(false);
                break;
            }
        };

        let oper = match OpCode::try_from(current_byte) {
            Ok(val) => val,
            Err(t) => {
                return Err(EvaluateError::InvalidOperation);
            },
        };

        match oper {
            OpCode::OpAdd | OpCode::OpDiv | OpCode::OpEq | OpCode::OpMul | OpCode::OpSub => {
                let mut call_frame = program.get_call_frame_mut()?;
                let right = program.pop()?;
                let left = program.pop()?;

                if oper == OpCode::OpEq {

                } else {
                    let (lval, rval) = match (left, right) {
                        (Value::Number(l), Value::Number(r)) => (l, r),
                        _ => return Err(EvaluateError::CrossValueOperation),
                    };

                    let mut result = lval;
                    match oper {
                        OpCode::OpAdd => {result = lval + rval},
                        OpCode::OpSub => {result = lval - rval},
                        OpCode::OpDiv => {result = lval / rval},
                        OpCode::OpMul => {result = lval * rval},
                        _ => return Err( EvaluateError::UnsupportedOperation )
                    }

                    program.push(Value::Number(result));
                }
            }

            OpCode::OpPushNum => {
                let num_pushed = program.get_call_frame_mut()?.read_f64()?;
                let value = Value::Number(num_pushed);
                program.push(value);
            },

            OpCode::OpPushU8 => {
                let num_pushed = program.get_call_frame_mut()?.read_u8()?;
                let value = Value::Number( f64::from(num_pushed) );
                program.push(value);
            },

            OpCode::OpPushU16 => {
                let num_pushed = program.get_call_frame_mut()?.read_u16()?;
                let value = Value::Number( f64::from(num_pushed) );
                program.push(value);
            },

            OpCode::OpPush0 | OpCode::OpPush1 => {
                let mut call_frame = program.get_call_frame_mut()?;
                let used_val = if oper == OpCode::OpPush1 { true } else { false };
                let value = Value::Bool(used_val);
                program.push(value);
            },

            OpCode::OpLoadConst => {
                let arg = program.get_call_frame_mut()?.read_u8()? as usize;
                let value = program.load_constant(arg)?;
                program.push(value);
            },

            OpCode::OpJump => {
                let mut call_frame = program.get_call_frame_mut()?;
                let arg = call_frame.read_u16()?;
                call_frame.program_counter += arg as usize;
            },

            OpCode::OpStoreLocal => {
                let value = program.pop()?;
                let register = program.get_call_frame_mut()?.read_u8()? as usize;
                //println!("Set Register {} [{}] to: {}", register, call_frame_idx, value);
                program.set_local(register, value);
            }

            OpCode::OpLoadLocal => {
                let arg = program.get_call_frame_mut()?.read_u8()? as usize;
                let value = program.get_local(arg)?.clone();
                program.push(value);
            }

            OpCode::OpReturn => {
                let mut call_frame = program.get_call_frame_mut()?;
                let value_returned = program.pop()?;
                program.call_stack.pop();

                let mut prev_frame = program.get_call_frame_mut()?;
                //println!("Value is: {}", value_returned);
                program.push(value_returned);

            }

            OpCode::OpNativeFnCall => {
                let native_idx = program.get_call_frame_mut()?.read_u8()? as usize;
                let arg_count = program.get_call_frame_mut()?.read_u8()? as usize;

                let stack_len = program.stack.len();
                let mut output = &mut program.std_out;
                let args = match program.stack.get_mut((stack_len-arg_count .. stack_len) ) {
                    Some(slice) => {
                        slice.reverse();
                        slice
                    },
                    None => return  Err( EvaluateError::StackEndReached ),
                };

                match native_idx {
                    0x00 => {
                        stdlib::out::print(output, args, arg_count as u8);
                    },
                    _ => return Err( EvaluateError::UndefinedFunction ),
                }
            }

            OpCode::OpFunctionCall => {
                let fn_idx = program.get_call_frame_mut()?.read_u32()? as usize;
                let (arg_count, instructions, fn_reg_count) = match program.load_function(fn_idx)? {
                    Constant::FunctionConstant(body) => ( 
                        body.argument_count as usize,
                        body.instructions.clone(),
                        body.registers_used as usize,
                    ),
                    _ => return Err( EvaluateError::NotAFunction ),
                };

                let stack_base = program.stack.len();
                let reg_base = program.registers.len();
                let new_size = reg_base + fn_reg_count;
                program.registers.resize(new_size, Value::Number(0.0));

                let mut new_call_frame = CallFrame::new(instructions, stack_base, reg_base);

                let mut index = 0;
                for val in (0..arg_count).rev() {
                    let val = program.pop()?;
                    program.set_local(index + reg_base, val);
                    index += 1;
                }

                program.call_stack.push(new_call_frame);
            }

            _ => {}//println!("\x1b[3;30mEvaluating\x1b[0m \x1b[1;29mByte<{:#04x}>\x1b[0m", current_byte),
        }
    }
    Ok(())
}
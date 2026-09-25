use std::cell::RefCell;
use std::fmt::Display;
use std::io::{BufWriter, Write};
use std::rc::Rc;

use crate::vm::program::{CallFrame, Constant, VMStruct, Value};
use crate::vm::stdlib;

use super::program::Program;
use super::opcodes::OpCode;

#[derive(Debug)]
pub enum EvaluateError {
    OutOfRange,
    NoModuleActive,
    InvalidOperation,
    CircularDependency,
    ArrayIndexNaN,
    InstructionNotImplemented,
    UnknownInstruction,
    #[allow(unused)]
    StackEndReached,
    InvalidRegisterIndex,
    UndefinedConstant(u8),
    UndefinedLocalVariable,
    UndefinedModule(usize),
    UndefinedFunction,
    NoCallFrameAvailable,
    NotAFunction,
    CrossValueOperation(OpCode, u8, u8),
    UnsupportedOperation,
    RustIOError(String),

    // NoValueProvided,
    #[allow(unused)]
    RustStackOverflow,
}

pub enum ProgramExitCode {
    ProgramRanSuccesfully,
}

impl Display for EvaluateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluateError::RustIOError(str) => write!(f, "IOError: {str}"),
            EvaluateError::UndefinedConstant(val) => write!(f, "UndefinedConstant: {val}"),
            EvaluateError::CrossValueOperation(op, reg1, reg2) => write!(f, "Cross Value Operation: {}, R{}, R{}",
            op, reg1, reg2),
            EvaluateError::UndefinedModule(val) => write!(f, "Module [{val}] is undefined."),
            generic => write!(f, "{:?}", generic),
        }
    }
}

pub fn is_true(val: &Value) -> bool {
    match val {
        Value::Bool(state) => *state == true,
        Value::Nil => false,
        _ => true,
    }
}

pub fn evaluate(program: &mut Program) -> Result<ProgramExitCode, EvaluateError> {
    program.set_running(true);

    while program.is_running() {
        // let call_frame_idx = program.get_call_frame_idx();
        let current_byte = match program.get_call_frame_mut()?.advance() {
            Some(val) => val,
            None => {
                if program.call_stack.len() > 1 {
                    program.leave_module_compilation()?;
                    continue;
                }

                println!("Left at idx: {}", program.get_cur_mod_idx());
                program.set_running(false);
                break;
            }
        };

        let oper = match OpCode::try_from(current_byte) {
            Ok(val) => val,
            Err(_) => {
                return Err(EvaluateError::InvalidOperation);
            },
        };

        fn leave_on_unsupported(program: &mut Program) -> Result<(), EvaluateError> {
            program.get_call_frame_mut()?.program_counter -= 1;
            return Err(EvaluateError::InstructionNotImplemented)
        }

        match oper {
            OpCode::OpAdd | OpCode::OpDiv | OpCode::OpEq | OpCode::OpMul | OpCode::OpSub | OpCode::OpGreaterEqualThan | OpCode::OpGreaterThan | OpCode::OpLessThan | OpCode::OpLessEqualThan => {
                let target_reg = program.get_call_frame_mut()?.read_u8()? as usize;
                let left_reg = program.get_call_frame_mut()?.read_u8()? as usize;
                let right_reg = program.get_call_frame_mut()?.read_u8()? as usize;
                //let byte_end = program.get_call_frame_mut()?.program_counter as usize - 1;
                // println!("Got registers, used positions: {}, {}, {}, {}", byte_end - 3, byte_end - 2, byte_end - 1, byte_end);

                if oper == OpCode::OpEq {
                    let lval = program.get_local(left_reg)?.clone();
                    let rval = program.get_local(right_reg)?.clone();

                    let mut res = false;
                    match (lval, rval) {
                        (Value::String(str1), Value::String(str2)) => {
                            res = str1 == str2;
                        },
                        (Value::Number(n1), Value::Number(n2)) => {
                            res = n1 == n2;
                        },
                        (Value::Bool(n1), Value::Bool(n2)) => {
                            res = n1 == n2;
                        },
                        (Value::Array(refptr1), Value::Array(refptr2)) => {
                            res = Rc::ptr_eq(&refptr1, &refptr2);
                        },
                        _=>(),
                    }

                    program.set_local(target_reg, Value::Bool(res))?;
                } else {
                    let lval = match program.get_local(left_reg)? {
                        Value::Number(n) => n, 
                        _ => return Err(EvaluateError::CrossValueOperation(oper, left_reg as u8, right_reg as u8) ),
                    };

                    let rval = match program.get_local(right_reg)? {
                        Value::Number(n) => n,
                        _ => return Err(EvaluateError::CrossValueOperation(oper, left_reg as u8, right_reg as u8) ),
                    };

                    // println!("Left {}, Right: {}, Op: {:?}", lval, rval, oper);

                    let mut is_bool = false;
                    let result = match oper {
                        OpCode::OpAdd => {lval + rval},
                        OpCode::OpSub => {lval - rval},
                        OpCode::OpDiv => {lval / rval},
                        OpCode::OpMul => {lval * rval},
                        OpCode::OpGreaterEqualThan => {is_bool = true; if lval >= rval { 1.0 } else { 0.0 }},
                        OpCode::OpGreaterThan => {is_bool = true; if lval > rval { 1.0 } else { 0.0 }},
                        OpCode::OpLessEqualThan => {is_bool = true; if lval <= rval { 1.0 } else { 0.0 }},
                        OpCode::OpLessThan => {is_bool = true; if lval < rval { 1.0 } else { 0.0 }},
                        _ => return Err( EvaluateError::UnsupportedOperation )
                    };

                    let new_val = match is_bool {
                        true => Value::Bool(result == 1.0),
                        false => Value::Number(result),
                    };
                    program.set_local(target_reg, new_val)?;
                }
            }

            OpCode::OpPushNum => {
                let reg_pushed = program.get_call_frame_mut()?.read_u8()? as usize;
                let num_pushed = program.get_call_frame_mut()?.read_f64()?;
                let value = Value::Number(num_pushed);
                //program.push(value);
                program.set_local(reg_pushed, value)?;
            },

            OpCode::OpPushU8 => {
                let reg_pushed = program.get_call_frame_mut()?.read_u8()? as usize;
                let num_pushed = program.get_call_frame_mut()?.read_u8()?;
                let value = Value::Number( f64::from(num_pushed) );
                //program.push(value);
                program.set_local(reg_pushed, value)?;
            },

            OpCode::OpPushU16 => {
                let reg_pushed = program.get_call_frame_mut()?.read_u8()? as usize;
                let num_pushed = program.get_call_frame_mut()?.read_u16()?;
                let value = Value::Number( f64::from(num_pushed) );
                //program.push(value);
                program.set_local(reg_pushed, value)?;
            },

            OpCode::OpPush0 | OpCode::OpPush1 => {
                let reg_pushed = program.get_call_frame_mut()?.read_u8()? as usize;
                let used_val = if oper == OpCode::OpPush1 { true } else { false };
                let value = Value::Bool(used_val);
                program.set_local(reg_pushed, value)?;
                //program.push(value);
            },

            OpCode::OpLoadConst => {
                // println!("At index: {}", program.get_call_frame_mut()?.program_counter - 1);
                let register = program.get_call_frame_mut()?.read_u8()? as usize;
                let arg = program.get_call_frame_mut()?.read_u8()? as usize;
                let value = program.load_constant(arg)?;
                
                program.set_local(register, value)?;
            },

            OpCode::OpJump => {
                let call_frame = program.get_call_frame_mut()?;
                let arg = call_frame.read_i32()? as i128;
                call_frame.program_counter += arg;

                /*println!("Jumped back by: {}", arg);
                if let Some(v) = call_frame.instructions.get(call_frame.program_counter as usize) {
                    println!("At byte {}, pos: {}\n", v, call_frame.program_counter);
                } */
            },

            OpCode::OpJumpIfFalse | OpCode::OpJumpIfTrue => {
                let reg = program.get_call_frame_mut()?.read_u8()? as usize;
                let arg = program.get_call_frame_mut()?.read_i32()? as i128;
                let value = program.get_local(reg)?;
                let jump = if oper == OpCode::OpJumpIfFalse { is_true(&value) == false } else { is_true(&value) == true };
                if jump {
                    let call_frame = program.get_call_frame_mut()?;
                    call_frame.program_counter += arg;
                }
            },

            OpCode::OpStoreLocal => leave_on_unsupported(program)?,

            OpCode::OpLoadLocal => {
                let reg = program.get_call_frame_mut()?.read_u8()? as usize;
                let arg = program.get_call_frame_mut()?.read_u8()? as usize;
                let value = program.get_local(arg)?.clone();
                program.set_local(reg, value)?;
            }

            OpCode::OpReturn => {
                let call_frame = program.get_call_frame_mut()?;
                let reg_returned = call_frame.read_u8()? as usize;
                let reg_overwritten = call_frame.reg_ret;
                let value = program.get_local(reg_returned)?.clone();
                program.call_stack.pop();

                program.set_local(reg_overwritten, value)?;
            }

            OpCode::OpNativeFnCall => {
                let start_reg  = program.get_call_frame_mut()?.read_u8()? as usize;
                let native_fn_idx = program.get_call_frame_mut()?.read_u8()? as usize;
                let arg_count  = program.get_call_frame_mut()?.read_u8()? as usize;

                //let caller_reg_base = program.get_call_frame_mut()?.reg_base;

                let absolute_arg_start = start_reg;
                let absolute_arg_end = absolute_arg_start + arg_count;

                /*let args = program.get_mut_slice(absolute_arg_start .. absolute_arg_end)?; match program.registers.get_mut(absolute_arg_start..absolute_arg_end) {
                    Some(slice) => slice,
                    None => return Err(EvaluateError::InvalidRegisterIndex),
                }; */

                program.with_mut_slice(absolute_arg_start .. absolute_arg_end, |program_ref, slice| {
                    let mut temp_output = BufWriter::new(std::io::stdout());
                
                    let result = match native_fn_idx {
                        0x00 => {
                            stdlib::out::print(&mut temp_output, slice, arg_count as u8)?
                        },
                        _ => return Err( EvaluateError::UndefinedFunction ),
                    };

                    let cloned = &mut program_ref.std_out;
                    if let Err(e) = cloned.write_all(temp_output.buffer()) {
                        return Err( EvaluateError::RustIOError(format!("Buffer write failed, reason: {}", e)) )
                    };

                    //let current_frame_mut = program.get_call_frame_mut()?;
                    program_ref.set_local(start_reg, result)?;
                    Ok(())
                })??;
            }

            OpCode::OpFunctionCall => {
                let start_reg = program.get_call_frame_mut()?.read_u8()? as usize;
                let fn_idx = program.get_call_frame_mut()?.read_u32()? as usize;
                let (_arg_count, instructions, _fn_reg_count) = match program.load_function(fn_idx)? {
                    Constant::FunctionConstant(body) => ( 
                        body.argument_count as usize,
                        body.instructions.clone(),
                        body.registers_used as usize,
                    ),
                    _ => return Err( EvaluateError::NotAFunction ),
                };

                let caller_reg_base = program.get_call_frame_mut()?.reg_base;
                let new_reg_base = caller_reg_base + start_reg;

                /*if new_reg_base + fn_reg_count > program.registers.len() {
                    return Err(EvaluateError::RustStackOverflow);
                } */

                let new_call_frame = CallFrame::new(instructions, 0, new_reg_base, start_reg);
                
                program.call_stack.push(new_call_frame);
            }

            OpCode::OpLoadField => {
                let dest_reg  = program.get_call_frame_mut()?.read_u8()? as usize;
                let struct_reg  = program.get_call_frame_mut()?.read_u8()? as usize;
                let field_loaded  = program.get_call_frame_mut()?.read_u8()? as usize;
                let accessed_str = program.get_local(struct_reg)?;
                //println!("accessed: {}", accessed_str);

                let value = match accessed_str {
                    Value::Struct(str_ref) => {
                        match str_ref.borrow_mut().values.get(field_loaded) {
                            Some(v) => v.clone(),
                            None => return Err( EvaluateError::InvalidRegisterIndex )
                        }
                    },
                    Value::Array(arr_ref) => {
                        match arr_ref.borrow().get(field_loaded) {
                            Some(v) => v.clone(),
                            None => Value::Nil
                        }
                    },
                    Value::Module(module_ref) => {
                        let mapped_reg = *module_ref.borrow_mut().exports.get(field_loaded).ok_or( EvaluateError::OutOfRange )? as usize;
                        let local_cloned = module_ref.borrow_mut().get_local(mapped_reg)?.clone();

                        println!("returning: {}", local_cloned);
                        local_cloned
                    }
                    /* */
                    _ => {
                        program.get_call_frame_mut()?.program_counter -= 4;
                        return Err( EvaluateError::InvalidOperation )
                    },
                };

                program.set_local(dest_reg, value)?;
            }

            OpCode::OpLoadMod => {
                let dest_reg  = program.get_call_frame_mut()?.read_u8()? as usize;
                let module_idx  = program.get_call_frame_mut()?.read_u32()? as usize;

                //println!("\n\x1b[1;31m[TODO WARNING]\x1b[0m\nOP_LOAD_MOD SHOULD ALSO LOAD THE MODULE AND EXECUTE IT IF IT HASN'T BEEN.\n\x1b[1;31m[TODO WARNING]\x1b[0m\n");
                program.load_mod(dest_reg, module_idx)?;
                //println!("module?: {}", program.get_local(dest_reg)?);
            }

            OpCode::OpLoadGlob => leave_on_unsupported(program)?,
            OpCode::OpCallReg => leave_on_unsupported(program)?,
            OpCode::OpVoid => leave_on_unsupported(program)?,

            OpCode::OpLoadIndex => {
                let dest_reg  = program.get_call_frame_mut()?.read_u8()? as usize;
                let struct_reg  = program.get_call_frame_mut()?.read_u8()? as usize;
                let reg_val_loaded  = program.get_call_frame_mut()?.read_u8()? as usize;
                let idx = match program.get_local(reg_val_loaded)? {
                    Value::Number(t) => t,
                    _=> return Err( EvaluateError::ArrayIndexNaN ),
                } as usize;
                let value = match program.get_local(struct_reg)? {
                    Value::Array(arr_ref) => {
                        match arr_ref.borrow().get(idx) {
                            Some(v) => v.clone(),
                            None => Value::Nil
                        }
                    },
                    _ => return Err( EvaluateError::InvalidOperation ),
                };

                program.set_local(dest_reg, value)?;
            }

            OpCode::OpNewStruct => {
                let start_reg  = program.get_call_frame_mut()?.read_u8()? as usize;
                let count  = program.get_call_frame_mut()?.read_u8()? as usize;

                let mut vec: Vec<Value> = Vec::with_capacity(count);
                for reg_idx in start_reg..start_reg + count {
                    let value = program.take_local(reg_idx as usize)?;
                    vec.push(value);
                }

                let vm_struct = VMStruct { values: vec };
                let struct_val = Value::Struct(Rc::from(RefCell::from(vm_struct)));
                program.set_local(start_reg, struct_val)?;
            }

            OpCode::OpPushArray => {
                let start_reg  = program.get_call_frame_mut()?.read_u8()? as usize;
                let size  = program.get_call_frame_mut()?.read_u32()? as usize;
                let mut vec: Vec<Value> = Vec::with_capacity(size);

                for reg_idx in start_reg..start_reg + size {
                    let value = program.take_local(reg_idx as usize)?;
                    vec.push(value);
                }

                let vm_array = Value::Array(Rc::from(RefCell::from(vec)));
                program.set_local(start_reg, vm_array)?;
            }

            _ => {
                program.get_call_frame_mut()?.program_counter -= 1;
                return Err(EvaluateError::UnknownInstruction)
            }//println!("\x1b[3;30mEvaluating\x1b[0m \x1b[1;29mByte<{:#04x}>\x1b[0m", current_byte),
        }
    }

    Ok(ProgramExitCode::ProgramRanSuccesfully)
}
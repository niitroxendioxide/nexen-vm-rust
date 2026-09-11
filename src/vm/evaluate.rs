use crate::vm::program::Value;

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
}

pub fn evaluate(program: &mut Program) -> Result<(), EvaluateError> {
    if program.program_counter >= program.instructions.len() {
        return Ok(());
    }

    let current_byte = unsafe {
        *program.instructions.get_unchecked(program.program_counter)
    };

    program.program_counter += 1;

    let oper = match OpCode::try_from(current_byte) {
        Ok(val) => val,
        Err(t) => {
            return Err(EvaluateError::InvalidOperation);
        },
    };

    match oper {
        OpCode::OpPushNum => {
            let num_pushed = program.read_f64()?;
            let value = Value::Number(num_pushed);
            program.push_to_stack(value);
        },

        OpCode::OpPushU8 => {
            let num_pushed = program.read_u8()?;
            let value = Value::Number( f64::from(num_pushed) );
            program.push_to_stack(value);
        },

        OpCode::OpPushU16 => {
            let num_pushed = program.read_u16()?;
            let value = Value::Number( f64::from(num_pushed) );
            program.push_to_stack(value);
        },

        OpCode::OpPush0 | OpCode::OpPush1 => {
            let used_val = if oper == OpCode::OpPush1 { true } else { false };
            let value = Value::Bool(used_val);
            program.push_to_stack(value);
        },

        OpCode::OpLoadConst => {
            let arg = program.read_u8()?;
            let value = program.load_constant(arg as usize)?;
            program.push_to_stack(value);
        },

        OpCode::OpJump => {
            let arg = program.read_u16()?;
            program.program_counter += arg as usize;
        },

        OpCode::OpStoreLocal => {
            let value = program.pop_stack()?;
            let register = program.read_u8()?;
            let scope = &mut program.current_scope;
            println!("Set register: {} to value: {}", register, value);
            scope.set(register, value);
        }

        OpCode::OpLoadLocal => {
            let arg = program.read_u8()?;
            let scope = &mut program.current_scope;
            let value = scope.get(arg)?;
            program.push_to_stack(value);
        }

        _ => println!("\x1b[3;30mEvaluating\x1b[0m \x1b[1;29mByte<{:#04x}>\x1b[0m", current_byte),
    }

    evaluate(program)?;
    Ok(())
}
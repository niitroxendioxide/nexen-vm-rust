use crate::vm::program::{Value};
use crate::vm::evaluate::{EvaluateError};
use std::io::{BufWriter, Error, Stdout, Write};
use std::rc::Rc;

pub fn to_io_error(er: Error) -> EvaluateError { EvaluateError::RustIOError(er.to_string()) }

#[allow(unused)]
pub fn tostring(value: &Value) -> Value {
    match value {
        Value::Bool(val) => Value::String(val.to_string().into()),
        Value::Number(val) => Value::String(val.to_string().into()),
        Value::String(t) => Value::String(t.clone()),
        Value::Nil => Value::String(Rc::from("nil")),
        Value::Struct(vec) => {
            let ptrfm = format!("{:p}", vec.as_ptr());
            Value::String(Rc::from(ptrfm))
        },
        Value::Array(vec) => {
            let ptrfm = format!("{:p}", vec.as_ptr());
            Value::String(Rc::from(ptrfm))
        }, 
    }
}

fn print_value<W: Write>(output: &mut W, val: &Value) -> Result<(), EvaluateError> {
    match val {
        Value::Number(n) => write!(output, "{} ", n).map_err(to_io_error),
        Value::String(s) => write!(output, "{} ", s).map_err(to_io_error),
        Value::Bool(b)   => write!(output, "{} ", b).map_err(to_io_error),
        Value::Nil       => output.write_all(b"nil ").map_err(to_io_error),
        
        #[allow(unreachable_patterns)]
        generic => {
            let str_representation = tostring(generic); 
            print_value(output, &str_representation)?;
            Ok(())
        }
    }
}

pub fn print(output: &mut BufWriter<Stdout>, values: &[Value], arg_count: u8) -> Result<Value, EvaluateError> {
    if values.len() < arg_count as usize {
        return Err( EvaluateError::UnsupportedOperation )
    }
    
    for i in 0 .. arg_count as usize {
        let val = match values.get(i) {
            Some(v) => v,
            None => {write!(output, " ").map_err(to_io_error)?; continue;},
        };

        print_value(output, val)?;
    }

    write!(output, "\n").map_err(to_io_error)?;

    Ok(Value::Nil)
}
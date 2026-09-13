use crate::vm::program::{Value, Program};
use std::io::{BufWriter, Error, Stdout, Write};

pub fn tostring(value: &Value) -> Value {
    match value {
        Value::Bool(val) => Value::String(val.to_string()),
        Value::Number(val) => Value::String(val.to_string()),
        Value::String(t) => Value::String(t.to_string()),
    }
}

pub fn print(output: &mut BufWriter<Stdout>, values: &[Value], arg_count: u8) -> Result<(), Error> {
    for i in (0 .. arg_count as usize) {
        let val = match values.get(i) {
            Some(v) => v,
            None => {write!(output, " ")?; continue;},
        };

        let str_val = match tostring(val) {
            Value::String(str) => str,
            _ => panic!("Impossible"),
        };

        write!(output, "{} ", str_val);
    }

    write!(output, "\n");

    Ok(())
}
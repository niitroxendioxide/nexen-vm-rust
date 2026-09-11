use std::{collections::HashMap, fmt::Display};
use super::evaluate::EvaluateError;

type ReadRange = std::ops::Range<usize>;
pub trait FromLeBytes<const N: usize>: Sized {
    fn from_le_bytes(bytes: [u8; N]) -> Self;
}

impl FromLeBytes<8> for f64 {
    fn from_le_bytes(bytes: [u8; 8]) -> Self { f64::from_le_bytes(bytes) }
}

impl FromLeBytes<4> for u32 {
    fn from_le_bytes(bytes: [u8; 4]) -> Self { u32::from_le_bytes(bytes) }
}

impl FromLeBytes<2> for u16 {
    fn from_le_bytes(bytes: [u8; 2]) -> Self { u16::from_le_bytes(bytes) }
}

impl FromLeBytes<2> for i16 {
    fn from_le_bytes(bytes: [u8; 2]) -> Self { i16::from_le_bytes(bytes) }
}

fn read_le<T: FromLeBytes<N>, const N: usize>(bytes: &[u8], range: ReadRange) -> Result<T, EvaluateError> {
    let slice = bytes.get(range).ok_or(EvaluateError::OutOfRange)?;
    let array: [u8; N] = slice.try_into().map_err(|_| EvaluateError::OutOfRange)?;
    
    Ok(T::from_le_bytes(array))
}


#[repr(u8)]
#[derive(Debug)]
pub enum Constant {
    _BoolConstant = 0x0,
    _NumberConstant = 0x1,
    StringConstant(String) = 0x02,
}

#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
}

impl From<&[u8]> for Constant {
    fn from(value: &[u8]) -> Self {
        let new_value = String::from_utf8(value.to_vec()).unwrap_or_default();
        Constant::StringConstant(new_value)
    }
}

impl From<&Constant> for Value {
    fn from(value: &Constant) -> Self {
        match value {
            Constant::StringConstant(str_val) => Value::String(str_val.to_owned()),
            _=> Value::Number(0.0),
        }
    }
}

pub struct Stack {
    pub values: Vec<Value>,
}

pub struct Scope {
    pub locals: HashMap<u8, Value>,
    pub parent: Option<Box<Scope>>,
}

pub struct Program {
    pub version_major: u16,
    pub version_minor: u16,
    pub constant_count: usize,
    pub constants: Vec<Constant>,
    pub instructions: Vec<u8>,
    pub program_counter: usize,
    pub current_scope: Box<Scope>,
    pub current_stack: Stack,
}

impl Display for Program {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Program \x1b[0;33m<Version=\x1b[1;33m[{}.{}]\x1b[0;33m, Constants=\x1b[1;33m[{}]\x1b[0;33m, ProgramLength=\x1b[1;33m[{}b]\x1b[0;33m>\x1b[0m", 
        self.version_major, self.version_minor, self.constant_count, self.instructions.len())?;
        Ok(())
    }
}

impl Display for Value {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(raw_val) => write!(formatter, "\x1b[0;33mRuntime<{}>\x1b[0m", raw_val)?,
            Value::String(raw_str) => write!(formatter, "\x1b[0;33mRuntime<\"{}\">\x1b[0m", raw_str)?,
            Value::Bool(raw_bool) => write!(formatter, "\x1b[0;33mRuntime<{}>\x1b[0m", raw_bool)?,
        }
        Ok(())
    }
}


impl Display for Constant {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::StringConstant(val) => write!(formatter, "\x1b[2;32mString\x1b[0;32m<{}>\x1b[2;32m\x1b[0m", val)?,
            _ => write!(formatter, "{:?}", self)?,
        }
        Ok(())
    }
}

impl Stack {
    pub fn new() -> Self {
        Stack { values: vec![] }
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.values.pop()
    }
}


impl Scope {
    pub fn new(parent: Option<Box<Scope>>) -> Self {
        Scope {
            locals: HashMap::new(),
            parent,
        }
    }

    pub fn set(&mut self, idx: u8, value: Value) {
        self.locals.insert(idx, value);
        
        return;
    }

    pub fn get(&mut self, idx: u8) -> Result<Value, EvaluateError> {
        match self.locals.get(&idx) {
            Some(val) => Ok(val.clone()),
            None => Err( EvaluateError::UndefinedLocalVariable ),
        }
    }
}

impl Program {
    pub fn new(version_major: u16, version_minor: u16, instructions: Vec<u8>, constants: Vec<Constant>) -> Self {
        Program { 
            version_major, 
            version_minor, 
            constant_count: constants.len(), 
            constants,
            instructions, 
            program_counter: 0,
            current_scope: Box::new(Scope::new(None)),
            current_stack: Stack::new(),
        }
    }

    pub fn load_constant(&self, index: usize) -> Result<Value, EvaluateError> {
        match self.constants.get(index) {
            Some(constant) => {
                Ok(Value::from(constant))
            },
            None => Err(EvaluateError::UndefinedConstant),
        }
    }

    pub fn push_to_stack(&mut self, value: Value) {
        println!("Pushed value: {} to stack", value);
        self.current_stack.push(value);
    }

    pub fn pop_stack(&mut self) -> Result<Value, EvaluateError> {
        match self.current_stack.pop() {
            Some(val) => Ok(val),
            None => Err(EvaluateError::StackEndReached)
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, EvaluateError> {
        let value = read_le(&self.instructions, self.program_counter .. self.program_counter+4);
        self.program_counter+=4;
        value
    }

    pub fn read_f64(&mut self) -> Result<f64, EvaluateError> {
        let value = read_le(&self.instructions, self.program_counter .. self.program_counter+8);
        self.program_counter+=8;
        value
    }

    pub fn read_i16(&mut self) -> Result<i16, EvaluateError> {
        let value = read_le(&self.instructions, self.program_counter .. self.program_counter+2);
        self.program_counter+=2;
        value
    }

    pub fn read_u16(&mut self) -> Result<u16, EvaluateError> {
        let value = read_le(&self.instructions, self.program_counter .. self.program_counter+2);
        self.program_counter+=2;
        value
    }

    pub fn read_u8(&mut self) -> Result<u8, EvaluateError> {
        let value = match self.instructions.get(self.program_counter) {
            Some(val) => Ok(*val),
            None => Err(EvaluateError::OutOfRange),
        };
        self.program_counter+=1;
        value
    }
}
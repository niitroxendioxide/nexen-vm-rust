use std::{cell::RefCell, collections::HashMap, fmt::Display, io::{BufWriter, Stdout}, rc::Rc};
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

impl FromLeBytes<4> for i32 {
    fn from_le_bytes(bytes: [u8; 4]) -> Self { i32::from_le_bytes(bytes) }
}

fn read_le<T: FromLeBytes<N>, const N: usize>(bytes: &[u8], range: ReadRange) -> Result<T, EvaluateError> {
    let slice = bytes.get(range).ok_or(EvaluateError::OutOfRange)?;
    let array: [u8; N] = slice.try_into().map_err(|_| EvaluateError::OutOfRange)?;
    
    Ok(T::from_le_bytes(array))
}

#[derive(Debug, Clone)]
pub struct FunctionBody {
    pub length: u32,
    pub argument_count: u8,
    pub registers_used: u8,
    pub instructions: Rc<Vec<u8>>,
}

#[derive(Debug)]
pub struct VMStruct {
    pub values: Vec<Value>,
}

impl FunctionBody {
    pub fn new(length: u32, argument_count: u8, registers_used: u8, instructions: &[u8]) -> Self {
        FunctionBody { length, argument_count, instructions: Rc::new(Vec::from(instructions)), registers_used }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClassBody {
    pub name: Value,
    pub fields: Vec<Value>,
    pub methods: Vec<Value>,
}

#[allow(dead_code)]
impl ClassBody {
    pub fn new(name: Value, fields: Vec<Value>, methods: Vec<Value>) -> Self {
        ClassBody { name, fields, methods }
    }

    pub fn get_name_as_str(&self) -> Rc<str> {
        match &self.name {
            Value::String(str_ref) => str_ref.clone(),
            _ => panic!("Unreachable"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Constant {
    BoolConstant(bool) = 0x0,
    NumberConstant(f64) = 0x1,
    StringConstant(Rc<str>) = 0x02,
    FunctionConstant(FunctionBody) = 0x03,
    ArrayConstant(Rc<RefCell<Vec<Constant>>>) = 0x04,
    StringRefConstant(u8) = 0x07,
    RegisterRefConstant(u8) = 0x08,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Value {
    // Short(u16),
    Number(f64),
    String(Rc<str>),
    Bool(bool),
    Array(Rc<RefCell<Vec<Value>>>),
    Dict(Rc<RefCell<HashMap<Value, Value>>>),
    Struct(Rc<RefCell<VMStruct>>),
    Class(Rc<RefCell<ClassBody>>),
    Nil,
}

impl From<&[u8]> for Constant {
    fn from(value: &[u8]) -> Self {
        let converted_string = String::from_utf8(value.to_vec()).unwrap_or_default();
        let rc_string: Rc<str> = converted_string.into();
        Constant::StringConstant(rc_string)
    }
}

impl Into<String> for Value {
    fn into(self) -> String {
        match self {
            Value::Array(_) => "<array>".to_string(),
            Value::Dict(_) => "<dict>".to_string(),
            Value::Struct(_) => "<struct>".to_string(),
            Value::Class(_) => "<class>".to_string(),
            Value::Bool(val) => val.to_string(),
            Value::Nil => "nil".to_string(),
            Value::Number(f) => f.to_string(),
            Value::String(str) => String::from(str.clone().to_string()),
        }
    }
}

fn value_from_constant(value: &Constant, program: &mut Program) -> Result<Value, EvaluateError> {
    match value {
        Constant::StringConstant(str_val) => Ok(Value::String(str_val.clone())),
        Constant::BoolConstant(bool) => Ok(Value::Bool(*bool)),
        Constant::NumberConstant(num) => Ok(Value::Number(*num)),
        Constant::RegisterRefConstant(byte_ref) => {
            Ok(program.get_local(*byte_ref as usize)?.clone())
        },
        Constant::StringRefConstant(byte_ref) => {
            Ok(program.load_constant(*byte_ref as usize)?)
        },
        Constant::ArrayConstant(elements) => {
            let mut array_vec: Vec<Value> = Vec::with_capacity(elements.borrow().len());
            for static_element in elements.borrow().iter() {
                let val = value_from_constant(static_element, program)?;
                array_vec.push(val);
            }

            Ok(Value::Array(Rc::from(RefCell::from(array_vec))))
        }
        _=> Ok(Value::Nil),
    }
}



/*pub struct Scope {
    pub locals: HashMap<u8, Value>,
    pub parent: Option<Rc<RefCell<Scope>>>,
    pub current_stack: Stack,
} */

pub struct CallFrame {
    pub program_counter: i64,
    pub instructions: Rc<Vec<u8>>,
    //pub scope: Rc<RefCell<Scope>>,
    pub reg_base: usize,
    pub reg_ret: usize,
    pub function_id: i64,
}

pub struct Program {
    pub version_major: u16,
    pub version_minor: u16,
    pub version_patch: u16,
    pub constant_count: usize,
    pub constants: Vec<Constant>,
    pub functions: Vec<Constant>,
    pub call_stack: Vec<CallFrame>,
    pub stack: Vec<Value>,
    pub registers: Vec<Value>,
    // pub actual_pointer: i128,
    pub std_out: BufWriter<Stdout>,
    //pub current_scope: Rc<RefCell<Scope>>,
    pub running_state: bool,
}

impl Display for Program {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let program_len = match self.call_stack.first() {
            Some(v) => v.instructions.len(),
            None => 0,
        };
        write!(formatter, "Nexen Program \x1b[0;33m<Version: \x1b[1;33m{}.{}.{}\x1b[0;33m, Constants: \x1b[1;33m{}\x1b[0;33m, Program Length: \x1b[1;33m{}B\x1b[0;33m>\x1b[0m", 
        self.version_major, self.version_minor, self.version_patch, self.constant_count, program_len)?;
        Ok(())
    }
}

impl Display for Value {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(raw_val) => write!(formatter, "\x1b[0;33mRuntime<Number, {}>\x1b[0m", raw_val)?,
            Value::String(raw_str) => write!(formatter, "\x1b[0;33mRuntime<String, \"{}\">\x1b[0m", raw_str)?,
            Value::Bool(raw_bool) => write!(formatter, "\x1b[0;33mRuntime<Bool, {}>\x1b[0m", raw_bool)?,
            Value::Array(refv) => write!(formatter, "\x1b[0;33mRuntime<Array[{}]>\x1b[0m", refv.borrow().len())?,
            Value::Struct(_) => write!(formatter, "\x1b[0;33mRuntime<Struct>\x1b[0m")?,
            Value::Dict(refv) => write!(formatter, "\x1b[0;33mRuntime<Dict[{}]>\x1b[0m", refv.borrow().len())?,
            Value::Class(_) => write!(formatter, "\x1b[0;33mRuntime<Class>\x1b[0m")?,
            Value::Nil => write!(formatter, "\x1b[0;33mRuntime<Nil>\x1b[0m")?,
        }
        Ok(())
    }
}


impl Display for Constant {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::StringConstant(val) => write!(formatter, "\x1b[2;32mString\x1b[0;32m<{}>\x1b[2;32m\x1b[0m", val)?,
            Constant::FunctionConstant(body) => write!(formatter, "\x1b[2;34mFunction\x1b[0;34m<len: {}, args: {}, body: {:?}>\x1b[2;34m\x1b[0m", body.length, body.argument_count, body.instructions)?,
            _ => write!(formatter, "{:?}", self)?,
        }
        Ok(())
    }
}

impl CallFrame {
    pub fn new(instructions: Rc<Vec<u8>>, function_id: i64, reg_base: usize, reg_ret: usize) -> Self {
        CallFrame { program_counter: 0, instructions, function_id, reg_base, reg_ret }
    }

    pub fn read_u32(&mut self) -> Result<u32, EvaluateError> {
        let range = self.program_counter as usize .. (self.program_counter as usize)+4;
        let value = read_le(&self.instructions, range);
        self.program_counter+=4;
        value
    }

    pub fn read_f64(&mut self) -> Result<f64, EvaluateError> {
        let range = self.program_counter as usize .. (self.program_counter as usize)+8;
        let value = read_le(&self.instructions, range);
        self.program_counter+=8;
        value
    }

    pub fn _read_i16(&mut self) -> Result<i16, EvaluateError> {
        let range = self.program_counter as usize .. (self.program_counter as usize)+2;
        let value = read_le(&self.instructions, range);
        self.program_counter+=2;
        value
    }

    pub fn read_u16(&mut self) -> Result<u16, EvaluateError> {
        let range = self.program_counter as usize .. (self.program_counter as usize)+2;
        let value = read_le(&self.instructions, range);
        self.program_counter+=2;
        value
    }

    pub fn read_i32(&mut self) -> Result<i32, EvaluateError> {
        let range = self.program_counter as usize .. (self.program_counter as usize)+4;
        let value = read_le(&self.instructions, range);
        self.program_counter+=4;
        value
    }

    pub fn read_u8(&mut self) -> Result<u8, EvaluateError> {
        let value = match self.instructions.get(self.program_counter as usize) {
            Some(val) => Ok(*val),
            None => Err(EvaluateError::OutOfRange),
        };
        self.program_counter+=1;
        value
    }

    pub fn advance(&mut self) -> Option<u8> {
        if self.program_counter as usize >= self.instructions.len() {
            return None;
        }

        let current_byte = unsafe {*self.instructions.get_unchecked(self.program_counter as usize)};

        self.program_counter += 1;
        Some(current_byte)
    }
}

impl Program {
    pub fn new(version_major: u16, version_minor: u16, version_patch: u16, instructions: Rc<Vec<u8>>, functions: Vec<Constant>, constants: Vec<Constant>, _registers_used: u8) -> Self {
        let first_call_frame = CallFrame::new(instructions, -1, 0, 0);
        let mut call_stack_vec = Vec::with_capacity(50);
        let mut registers = Vec::with_capacity(256);
        registers.resize(256, Value::Nil);
        call_stack_vec.push(first_call_frame);
        
        Program { 
            version_major, 
            version_minor, 
            version_patch,
            constant_count: constants.len(), 
            constants,
            functions,
            call_stack: call_stack_vec,
            running_state: false,
            registers: registers,
            stack: Vec::with_capacity(64),
            std_out: BufWriter::new(std::io::stdout())
        }
    }

    pub fn load_constant(&mut self, index: usize) -> Result<Value, EvaluateError> {
        let const_idx = self.constants.get(index).cloned();
        match const_idx {
            Some(constant) => value_from_constant(&constant, self),
            None => Err( EvaluateError::UndefinedConstant(index as u8) ),
        }
    }

    pub fn load_function(&self, index: usize) -> Result<&Constant, EvaluateError> {
        match self.functions.get(index) {
            Some(constant) => {
                Ok(constant)
            },
            None => Err(EvaluateError::UndefinedFunction),
        }
    }

    pub fn get_call_frame_mut(&mut self) -> Result<&mut CallFrame, EvaluateError> {
        match self.call_stack.last_mut() {
            Some(cframe) => Ok(cframe),
            None => Err(EvaluateError::NoCallFrameAvailable),
        }
    }

    pub fn _get_call_frame_idx(&mut self) -> usize {
        self.call_stack.len() - 1
    }

    pub fn is_running(&mut self) -> bool {
        self.running_state
    }

    pub fn set_running(&mut self, state: bool) {
        self.running_state = state;
    }

    pub fn _push(&mut self, value: Value) -> Result<(), EvaluateError> {
        self.stack.push(value);
        //self.scope.borrow_mut().push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Value, EvaluateError> {
        match self.stack.pop() {
            Some(val) => Ok(val),
            None => Err( EvaluateError::StackEndReached ),
        }
        //self.scope.borrow_mut().pop()
    }

    pub fn set_local(&mut self, idx: usize, value: Value) -> Result<(), EvaluateError> {
        let reg_base = self.get_call_frame_mut()?.reg_base;
        if (reg_base + idx) >= self.registers.len() {
            self.registers.reserve(128);
        }

        self.registers[reg_base + idx] = value;
        Ok(())
        //self.scope.borrow_mut().set((idx + self.base) as u8, value);
    }

    pub fn get_local(&mut self, idx: usize) -> Result<&Value, EvaluateError> {
        let reg_base = self.get_call_frame_mut()?.reg_base;
        match self.registers.get(reg_base + idx) {
            Some(val) => Ok(val),
            None => Err( EvaluateError::UndefinedLocalVariable )
        }
        //self.scope.borrow_mut().get((idx + self.base) as u8)
    }

    #[allow(unused)]
    pub fn get_call_frame(&self) -> Result<&CallFrame, EvaluateError> {
        self.call_stack.last().ok_or(EvaluateError::NoCallFrameAvailable)
    }

    #[allow(unused)]
    pub fn get_local_ref(&self, idx: usize) -> Result<&Value, EvaluateError> {
        let reg_base = self.get_call_frame()?.reg_base;
        self.registers.get(reg_base + idx).ok_or(EvaluateError::UndefinedLocalVariable)
    }

    pub fn take_local(&mut self, idx: usize) -> Result<Value, EvaluateError> {
        let reg_base = self.get_call_frame_mut()?.reg_base;
        match self.registers.get_mut(reg_base + idx) {
            Some(val) => Ok(std::mem::replace(val, Value::Nil)),
            None => Err( EvaluateError::UndefinedLocalVariable )
        }
        //self.scope.borrow_mut().get((idx + self.base) as u8)
    }
}
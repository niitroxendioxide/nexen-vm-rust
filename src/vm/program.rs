use std::{cell::RefCell, collections::HashMap, fmt::Display, io::{BufWriter, Stdout}, ops::Range, rc::Rc};
use crate::vm::modules::{ModuleState, ProgramModule};

use super::evaluate::EvaluateError;
use super::modules::Module;

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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClassBody {
    pub name: Value,
    pub fields: Vec<Value>,
    pub methods: Vec<Value>,
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Constant {
    BoolConstant(bool) = 0x0,
    NumberConstant(f64) = 0x1,
    StringConstant(ProgramString) = 0x02,
    FunctionConstant(FunctionBodyRef) = 0x03,
    ArrayConstant(ArrayConstantRef) = 0x04,
    StringRefConstant(u8) = 0x07,
    RegisterRefConstant(u8) = 0x08,
}

/* Quick type definitions */
pub type ProgramString = Rc<str>;
pub type ArrayConstantRef = Rc<RefCell<Vec<Constant>>>;
pub type Instructions = Rc<Vec<u8>>;
pub type CanError = Result<(), EvaluateError>;
pub type ProgramAccessed<T> = Result<T, EvaluateError>;
pub type ClassBodyRef = Rc<RefCell<ClassBody>>;
pub type StructRef = Rc<RefCell<VMStruct>>;
pub type DictRef = Rc<RefCell<HashMap<Value, Value>>>;
pub type ArrayRef = Rc<RefCell<Vec<Value>>>;
pub type FunctionBodyRef = Rc<FunctionBody>;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Value {
    // Short(u16),
    Number(f64),
    String(ProgramString),
    Bool(bool),
    Array(ArrayRef),
    Dict(DictRef),
    Struct(StructRef),
    Class(ClassBodyRef),
    Module(ProgramModule),
    Function(FunctionBodyRef, usize),
    Nil,
}

pub struct CallFrame {
    pub program_counter: i128,
    pub instructions: Rc<Vec<u8>>,
    //pub scope: Rc<RefCell<Scope>>,
    pub reg_base: usize,
    pub reg_ret: usize,
    pub mod_ctx_id: usize,
    //pub function_id: i64,
}

pub struct Program {
    pub version_major: u16,
    pub version_minor: u16,
    pub version_patch: u16,
    pub constant_count: usize,
    pub constants: Vec<Constant>,
    pub call_stack: Vec<CallFrame>,
    pub module_idx: Vec<usize>,
    pub modules: Vec<ProgramModule>,
    // pub actual_pointer: i128,
    pub std_out: BufWriter<Stdout>,
    //pub current_scope: Rc<RefCell<Scope>>,
    pub running_state: bool,
}


impl FunctionBody {
    pub fn new(length: u32, argument_count: u8, registers_used: u8, instructions: &[u8]) -> Self {
        FunctionBody { length, argument_count, instructions: Rc::new(Vec::from(instructions)), registers_used }
    }
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
            Value::Module(md) => format!("<module {:p}>", md),
            Value::Function(_, _) => "function".to_string(),
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
        },
        Constant::FunctionConstant(func) => Ok(Value::Function(func.clone(), program.modules.len())),

        #[allow(unreachable_patterns)]
        _=> Ok(Value::Nil),
    }
}



/*pub struct Scope {
    pub locals: HashMap<u8, Value>,
    pub parent: Option<Rc<RefCell<Scope>>>,
    pub current_stack: Stack,
} */

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
            Value::Module(_) => write!(formatter, "\x1b[0;33mRuntime<Module>\x1b[0m")?,
            Value::Function(_, _) => write!(formatter, "\x1b[0;33mRuntime<Function>\x1b[0m")?,
            Value::Nil => write!(formatter, "\x1b[0;33mRuntime<Nil>\x1b[0m")?,
        }
        Ok(())
    }
}


impl Display for Constant {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::StringConstant(val) => write!(formatter, "\x1b[2;32mString\x1b[0;32m<{}>\x1b[2;32m\x1b[0m", val)?,
            Constant::FunctionConstant(body) => write!(formatter, "\x1b[2;34mFunction\x1b[0;34m<len: {}, args: {}, body: {:?}>\x1b[2;34m\x1b[0m", 
            body.length, body.argument_count, body.instructions)?,
            _ => write!(formatter, "{:?}", self)?,
        }
        Ok(())
    }
}

impl CallFrame {
    pub fn new(
        instructions: Rc<Vec<u8>>, 
        program_counter: i128, 
        reg_base: usize, 
        reg_ret: usize,
        mod_ctx_id: usize,
    ) -> Self {
        CallFrame { program_counter, instructions, reg_base, reg_ret, mod_ctx_id }
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
    pub fn new(
        version_major: u16, 
        version_minor: u16, 
        version_patch: u16, 
        instructions: Instructions, 
        functions: Vec<Constant>, 
        constants: Vec<Constant>, 
        modules: Vec<ProgramModule>,
    ) -> Self {
        let module_idx_core = modules.len();
        let core_module = Module::new_ref(vec![], instructions.clone(), functions, module_idx_core);
        
        let mut cur_program = Program { 
            version_major, 
            version_minor, 
            version_patch,
            constant_count: constants.len(), 
            constants,
            call_stack: Vec::with_capacity(50),
            running_state: false,
            module_idx: vec![module_idx_core],
            modules,
            std_out: BufWriter::new(std::io::stdout()),
        };

        cur_program.modules.push(core_module);
        match cur_program.push_call_frame(instructions, 0, 0, 0, module_idx_core) {
            Ok(_) => cur_program,
            Err(e) => {
                println!("Error when loading mainframe: {e}");
                panic!("Program main-callframe couldn't be loaded.");
            }
        }
    }

    pub fn load_constant(&mut self, index: usize) -> ProgramAccessed<Value>/*Result<Value, EvaluateError>*/ {
        let const_idx = self.constants.get(index).cloned();
        match const_idx {
            Some(constant) => value_from_constant(&constant, self),
            None => Err( EvaluateError::UndefinedConstant(index as u8) ),
        }
    }

    pub fn load_function(&mut self, index: usize) -> ProgramAccessed<Constant> {
        self.get_current_module()?.borrow_mut().load_function(index)
    }

    pub fn get_call_frame_mut(&mut self) -> ProgramAccessed<&mut CallFrame> {
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

    pub fn get_cur_mod_idx(&mut self) -> usize {
        *self.module_idx.last().expect("Module should never be empty.")
    }

    pub fn get_current_module(&mut self) -> ProgramAccessed<ProgramModule> {
        let index = self.get_cur_mod_idx();
        match self.modules.get(index) {
            Some(v) => Ok(v.clone()),
            None => Err(EvaluateError::NoModuleActive)
        }
    }

    pub fn set_local(&mut self, idx: usize, value: Value) -> CanError {
        let reg_base = self.get_call_frame_mut()?.reg_base;
        self.get_current_module()?.borrow_mut().set_local(reg_base + idx, value)?;
        //self.core_module.set_local(reg_base + idx, value)?;
        Ok(())
    }

    pub fn get_local(&mut self, idx: usize) -> ProgramAccessed<Value> {
        let reg_base = self.get_call_frame_mut()?.reg_base;
        self.get_current_module()?.borrow_mut().get_local(reg_base + idx).cloned()
    }

    /*#[allow(unused)]
    pub fn get_call_frame(&self) -> Result<&CallFrame, EvaluateError> {
        self.call_stack.last().ok_or(EvaluateError::NoCallFrameAvailable)
    }
    
    #[allow(unused)]
    pub fn get_local_ref(&self, idx: usize) -> Result<&Value, EvaluateError> {
        let reg_base = self.get_call_frame()?.reg_base;
        self.core_module.get_local_ref(reg_base + idx)
    } */

    pub fn with_mut_slice<F, T>(&mut self, idx: Range<usize>, f: F) -> ProgramAccessed<T>
    where
        F: FnOnce(&mut Program, &mut [Value]) -> T,
    {
        let reg_base = self.get_call_frame_mut()?.reg_base;
        let new_range = idx.start + reg_base..idx.end + reg_base;

        let modctx = self.get_current_module()?;
        
        let mut borrowed = modctx.borrow_mut();

        let slice = borrowed.get_mut_slice(new_range)
            .map_err(|_| EvaluateError::InvalidRegisterIndex)?;

        Ok(f(self, slice))
    }

    pub fn load_global(&mut self, idx: usize) -> ProgramAccessed<Value> {
        let callframe_context = self.get_call_frame_mut()?.mod_ctx_id;
        let context_module = self.modules.get(callframe_context).ok_or(EvaluateError::NoModuleActive)?;
        let mut mut_borrow = context_module.borrow_mut();
        let variable = mut_borrow.get_local(idx)?;

        Ok(variable.clone())
    }

    /*pub fn get_mut_slice(&mut self, idx: Range<usize>) -> Result<&mut [Value], EvaluateError> {
            let reg_base = self.get_call_frame_mut()?.reg_base;
            let new_range = idx.start + reg_base..idx.end + reg_base;

            self.core_module.get_mut_slice(new_range)
    }*/

    pub fn take_local(&mut self, idx: usize) -> ProgramAccessed<Value> {
        let reg_base = self.get_call_frame_mut()?.reg_base;
        self.get_current_module()?.borrow_mut().take_local(reg_base + idx)
        //self.scope.borrow_mut().get((idx + self.base) as u8)
    }

    pub fn leave_module_compilation(&mut self) -> CanError {
        if self.call_stack.len() < 2 {
            return Err( EvaluateError::StackEndReached );
        }

        self.get_current_module()?.borrow_mut().set_state(ModuleState::Finished);
        self.module_idx.pop();
        self.call_stack.pop();
        Ok(())
    }

    pub fn load_mod(&mut self, var_idx: usize, module_idx: usize) -> CanError {
        let (module_instructions, module_obj) = match self.modules.get(module_idx) {
            Some(module_obj) => {
                let state = module_obj.borrow().state;

                if state == ModuleState::Unvisited {
                    module_obj.borrow_mut().set_state(ModuleState::Compiling);
                    let instructions = module_obj.borrow_mut().body.clone();
                    (Some(instructions), module_obj.clone())
                } else if state == ModuleState::Compiling {
                    return Err(EvaluateError::CircularDependency);
                } else {
                    (None, module_obj.clone())
                }
            }
            None => return Err(EvaluateError::UndefinedModule(module_idx)),
        };

        self.set_local(var_idx, Value::Module(module_obj))?;

        if let Some(instructions) = module_instructions {
            self.module_idx.push(module_idx);
            self.push_call_frame(instructions, 0, 0, 0, module_idx)?;
        }

        Ok(())
    }

    pub fn push_call_frame(&mut self, instructions: Instructions, pc: i128, reg_base: usize, reg_ret: usize, mod_ctx_id: usize) -> Result<(), EvaluateError> {
        let new_frame = CallFrame::new(instructions, pc, reg_base, reg_ret, mod_ctx_id);
        self.call_stack.push(new_frame);

        Ok(())
    }

    /*pub fn load_mod(&mut self, var_idx: usize, module_idx: usize) -> Result<(), EvaluateError> {
        match self.modules.get(module_idx) {
            Some(module_obj) => {
                self.set_local(var_idx, Value::Module(module_obj.clone()))?;

                Ok(())
            },
            None => return Err( EvaluateError::UndefinedModule(module_idx) )
        }
    }*/
}
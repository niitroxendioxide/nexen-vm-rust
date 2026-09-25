use std::cell::RefCell;
use std::rc::Rc;

use std::ops::Range;

use crate::vm::evaluate::{EvaluateError};
use crate::vm::program::{Constant, Instructions, Value};

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum ModuleState {
    Compiling,
    Finished,
    Unvisited,
}


#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Module {
    pub registers: Vec<Value>,
    pub exports: Vec<u8>,
    pub functions: Vec<Constant>,
    pub body: Instructions,
    pub state: ModuleState,
}

pub type ProgramModule = Rc<RefCell<Module>>;

impl Module {
    pub fn new_ref(exports: Vec<u8>, body: Rc<Vec<u8>>, functions: Vec<Constant>) -> ProgramModule {
        let mut registers = Vec::with_capacity(256);
        registers.resize(256, Value::Nil);

        Rc::from(RefCell::from(Module { registers, exports, body, functions, state: ModuleState::Unvisited }))
    }

    pub fn set_state(&mut self, state: ModuleState) {
        self.state = state;
    }

    pub fn set_local(&mut self, idx: usize, value: Value) -> Result<(), EvaluateError> {
        if (idx) >= self.registers.len() {
            //self.registers.reserve(128);
            let new_len = self.registers.len() + 128;
            self.registers.resize(new_len, Value::Nil);
        }

        self.registers[idx] = value;
        Ok(())
        //self.scope.borrow_mut().set((idx + self.base) as u8, value);
    }

    pub fn get_local(&mut self, idx: usize) -> Result<&Value, EvaluateError> {
        //let reg_base = self.get_call_frame_mut()?.reg_base;
        match self.registers.get(idx) {
            Some(val) => Ok(val),
            None => Err( EvaluateError::UndefinedLocalVariable )
        }
        //self.scope.borrow_mut().get((idx + self.base) as u8)
    }

    #[allow(unused)]
    pub fn get_local_ref(&self, idx: usize) -> Result<&Value, EvaluateError> {
        self.registers.get(idx).ok_or(EvaluateError::UndefinedLocalVariable)
    }

    pub fn get_mut_slice(&mut self, idx: Range<usize>) -> Result<&mut [Value], EvaluateError> {
        match self.registers.get_mut(idx) {
            Some(v) => Ok(v),
            None => return Err(EvaluateError::InvalidRegisterIndex),
        }
    }

    pub fn take_local(&mut self, idx: usize) -> Result<Value, EvaluateError> {
        match self.registers.get_mut(idx) {
            Some(val) => Ok(std::mem::replace(val, Value::Nil)),
            None => Err( EvaluateError::UndefinedLocalVariable )
        }
        //self.scope.borrow_mut().get((idx + self.base) as u8)
    }
}
#![allow(unused)]
use std::io::SeekFrom;

use crate::{
    locals::Locals,
    model::FuncType,
    stack::Stack,
    value::{self, Value},
};
use anyhow::{anyhow, Result};

struct CallStack {
    funcs: Vec<FuncStack>,
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack {
            funcs: vec![FuncStack::new()],
        }
    }

    pub fn commit(&mut self) {
        self.funcs.last_mut().unwrap().commit();
    }

    pub fn rollback(&mut self) {
        self.funcs.last_mut().unwrap().rollback();
    }

    pub fn get_func(&mut self) -> Result<&mut FuncStack> {
        self.funcs.last_mut().ok_or(anyhow!("No function in stack"))
    }

    pub fn add_func(&mut self, ty: &FuncType) -> Result<()> {
        let mut func_state = FuncStack::new();
        let func_stack = self.get_func()?;
        for param in ty.params.iter().rev() {
            let val = func_stack.pop()?;
            val.is_same_type(&param.val_type)?;
            func_state.locals.grow(param.id.clone(), val)?;
        }
        self.funcs.push(func_state);

        Ok(())
    }

    pub fn remove_func(&mut self, ty: &FuncType, requires_empty: bool) -> Result<()> {
        let mut func_stack = self.funcs.pop().ok_or(anyhow!("No function in stack"))?;
        let mut values = vec![];
        for result in ty.results.iter().rev() {
            let value = func_stack.pop()?;
            value.is_same_type(&result)?;
            values.push(value);
        }

        if requires_empty && !func_stack.is_empty()? {
            return Err(anyhow!("Too many returns"));
        }

        let func_stack = self.get_func()?;
        while values.len() > 0 {
            func_stack.push(values.pop().unwrap());
        }

        Ok(())
    }
}

struct FuncStack {
    blocks: Vec<Stack>,
    pub locals: Locals,
}

impl FuncStack {
    fn new() -> FuncStack {
        FuncStack {
            blocks: vec![Stack::new()],
            locals: Locals::new(),
        }
    }

    fn commit(&mut self) {
        self.blocks.last_mut().unwrap().commit();
        self.locals.commit();
    }

    fn rollback(&mut self) {
        self.blocks.last_mut().unwrap().rollback();
        self.locals.rollback();
    }

    fn get_latest_block(&mut self) -> Result<&mut Stack> {
        self.blocks.last_mut().ok_or(anyhow!("No block in stack"))
    }

    fn pop(&mut self) -> Result<Value> {
        self.get_latest_block()?.pop()
    }

    fn is_empty(&mut self) -> Result<bool> {
        Ok(self.get_latest_block()?.is_empty())
    }

    fn push(&mut self, value: Value) -> Result<()> {
        self.get_latest_block()?.push(value);
        Ok(())
    }
}

#[cfg(test)]
#[path = "./call_stack_test.rs"]
mod call_stack_test;

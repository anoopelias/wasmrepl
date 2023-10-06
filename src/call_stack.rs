use crate::{locals::Locals, model::FuncType, stack::Stack, value::Value};
use anyhow::{anyhow, Result};

pub struct CallStack {
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

    pub fn get_func_stack(&mut self) -> Result<&mut FuncStack> {
        self.funcs.last_mut().ok_or(anyhow!("No function in stack"))
    }

    pub fn add_func_stack(&mut self, ty: &FuncType) -> Result<()> {
        let mut func_state = FuncStack::new();
        let func_stack = self.get_func_stack()?;
        for param in ty.params.iter().rev() {
            let val = func_stack.pop()?;
            val.is_same_type(&param.val_type)?;
            func_state.locals.grow(param.id.clone(), val)?;
        }
        self.funcs.push(func_state);

        Ok(())
    }

    pub fn remove_func_stack(&mut self, ty: &FuncType, requires_empty: bool) -> Result<()> {
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

        let func_stack = self.get_func_stack()?;
        while values.len() > 0 {
            func_stack.push(values.pop().unwrap())?;
        }

        Ok(())
    }

    pub fn add_block_stack(&mut self, ty: &FuncType) -> Result<()> {
        self.get_func_stack()?.add_block_stack(ty)
    }

    pub fn remove_block_stack(&mut self, ty: &FuncType, requires_empty: bool) -> Result<()> {
        self.get_func_stack()?
            .remove_block_stack(ty, requires_empty)
    }

    pub fn to_string(&self) -> String {
        self.funcs.last().unwrap().to_string()
    }

    pub fn len(&self) -> usize {
        self.funcs.len()
    }
}

pub struct FuncStack {
    blocks: Vec<Stack>,
    pub locals: Locals,
}

impl FuncStack {
    pub fn new() -> FuncStack {
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

    pub fn pop(&mut self) -> Result<Value> {
        self.get_latest_block()?.pop()
    }

    fn is_empty(&mut self) -> Result<bool> {
        Ok(self.get_latest_block()?.is_empty())
    }

    pub fn push(&mut self, value: Value) -> Result<()> {
        self.get_latest_block()?.push(value);
        Ok(())
    }

    fn add_block_stack(&mut self, ty: &FuncType) -> Result<()> {
        let mut block_state = Stack::new();
        let mut values = vec![];
        for param in ty.params.iter().rev() {
            let val = self.pop()?;
            val.is_same_type(&param.val_type)?;
            values.push(val);
        }

        while values.len() > 0 {
            block_state.push(values.pop().unwrap());
        }
        self.blocks.push(block_state);

        Ok(())
    }

    fn remove_block_stack(&mut self, ty: &FuncType, requires_empty: bool) -> Result<()> {
        let mut block_stack = self.blocks.pop().ok_or(anyhow!("No block in stack"))?;
        let mut values = vec![];
        for result in ty.results.iter().rev() {
            let value = block_stack.pop()?;
            value.is_same_type(&result)?;
            values.push(value);
        }

        if requires_empty && !block_stack.is_empty() {
            return Err(anyhow!("Too many returns"));
        }

        while values.len() > 0 {
            self.push(values.pop().unwrap())?;
        }

        Ok(())
    }

    pub fn peek(&mut self) -> Result<Value> {
        self.get_latest_block()?.peek()
    }

    pub fn to_string(&self) -> String {
        self.blocks.last().unwrap().to_string()
    }

    #[allow(unused)]
    pub fn to_soft_string(&self) -> Result<String> {
        self.blocks.last().unwrap().to_soft_string()
    }
}

#[cfg(test)]
#[path = "./call_stack_test.rs"]
mod call_stack_test;

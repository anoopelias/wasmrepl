use anyhow::{Error, Result};
use wast::{
    core::Instruction,
    token::{Id, Index},
};

use crate::executor::State;

pub struct Handler<'a> {
    state: &'a mut State,
}

impl<'a> Handler<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Handler { state }
    }

    fn i32_const(&mut self, value: i32) -> Result<()> {
        self.state.stack.push(value);
        Ok(())
    }

    fn drop(&mut self) -> Result<()> {
        self.state.stack.pop()?;
        Ok(())
    }

    fn i32_clz(&mut self) -> Result<()> {
        let value = self.state.stack.pop()?;
        self.state.stack.push(value.leading_zeros() as i32);
        Ok(())
    }

    fn i32_ctz(&mut self) -> Result<()> {
        let value = self.state.stack.pop()?;
        self.state.stack.push(value.trailing_zeros() as i32);
        Ok(())
    }

    fn i32_add(&mut self) -> Result<()> {
        let a = self.state.stack.pop()?;
        let b = self.state.stack.pop()?;
        self.state.stack.push(a.wrapping_add(b));
        Ok(())
    }

    fn i32_sub(&mut self) -> Result<()> {
        let a = self.state.stack.pop()?;
        let b = self.state.stack.pop()?;
        self.state.stack.push(b.wrapping_sub(a));
        Ok(())
    }

    fn i32_mul(&mut self) -> Result<()> {
        let a = self.state.stack.pop()?;
        let b = self.state.stack.pop()?;
        self.state.stack.push(a.wrapping_mul(b));
        Ok(())
    }

    fn i32_div_s(&mut self) -> Result<()> {
        let value1 = self.state.stack.pop()?;
        let value2 = self.state.stack.pop()?;
        if value1 == 0 {
            return Err(Error::msg("Division by zero"));
        }
        self.state.stack.push(value2 / value1);
        Ok(())
    }

    fn local_get(&mut self, index: u32) -> Result<()> {
        let value = self.state.locals.get(index as usize)?;
        self.state.stack.push(value);
        Ok(())
    }

    fn local_get_by_id(&mut self, id: &Id) -> Result<()> {
        let val = self.state.locals.get_by_id(id.name())?;
        self.state.stack.push(val);
        Ok(())
    }

    fn local_set(&mut self, index: u32) -> Result<()> {
        let value = self.state.stack.pop()?;
        self.state.locals.set(index as usize, value)
    }

    fn local_set_by_id(&mut self, id: &Id) -> Result<()> {
        let value = self.state.stack.pop()?;
        self.state.locals.set_by_id(id.name(), value)
    }

    pub fn handle(&mut self, instr: &Instruction) -> Result<()> {
        match instr {
            Instruction::I32Const(value) => self.i32_const(*value),
            Instruction::Drop => self.drop(),
            Instruction::I32Clz => self.i32_clz(),
            Instruction::I32Ctz => self.i32_ctz(),
            Instruction::I32Add => self.i32_add(),
            Instruction::I32Sub => self.i32_sub(),
            Instruction::I32Mul => self.i32_mul(),
            Instruction::I32DivS => self.i32_div_s(),
            Instruction::LocalGet(Index::Num(index, _)) => self.local_get(*index),
            Instruction::LocalGet(Index::Id(id)) => self.local_get_by_id(id),
            Instruction::LocalSet(Index::Num(index, _)) => self.local_set(*index),
            Instruction::LocalSet(Index::Id(id)) => self.local_set_by_id(id),
            _ => Err(Error::msg("Unknown instruction")),
        }
    }
}

#[cfg(test)]
#[path = "./handler_test.rs"]
mod handler_test;

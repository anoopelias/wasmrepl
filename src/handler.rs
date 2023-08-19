use anyhow::{Error, Result};
use wast::{
    core::Instruction,
    token::{Id, Index},
};

use crate::{executor::State, value::Value};

pub struct Handler<'a> {
    state: &'a mut State,
}

impl<'a> Handler<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Handler { state }
    }

    fn constant(&mut self, value: Value) -> Result<()> {
        self.state.stack.push(value);
        Ok(())
    }

    fn i32_const(&mut self, value: i32) -> Result<()> {
        self.constant(value.into())
    }

    fn i64_const(&mut self, value: i64) -> Result<()> {
        self.constant(value.into())
    }

    fn drop(&mut self) -> Result<()> {
        self.state.stack.pop()?;
        Ok(())
    }

    fn i32_clz(&mut self) -> Result<()> {
        let value: i32 = self.state.stack.pop()?.try_into()?;
        self.state.stack.push((value.leading_zeros() as i32).into());
        Ok(())
    }

    fn i64_clz(&mut self) -> Result<()> {
        let value: i64 = self.state.stack.pop()?.try_into()?;
        self.state.stack.push((value.leading_zeros() as i64).into());
        Ok(())
    }

    fn i32_ctz(&mut self) -> Result<()> {
        let value: i32 = self.state.stack.pop()?.try_into()?;
        let value = (value.trailing_zeros() as i32).into();
        self.state.stack.push(value);
        Ok(())
    }

    fn i32_add(&mut self) -> Result<()> {
        let a: i32 = self.state.stack.pop()?.try_into()?;
        let b: i32 = self.state.stack.pop()?.try_into()?;
        self.state.stack.push(a.wrapping_add(b).into());
        Ok(())
    }

    fn i32_sub(&mut self) -> Result<()> {
        let a: i32 = self.state.stack.pop()?.try_into()?;
        let b: i32 = self.state.stack.pop()?.try_into()?;
        self.state.stack.push(b.wrapping_sub(a).into());
        Ok(())
    }

    fn i32_mul(&mut self) -> Result<()> {
        let a: i32 = self.state.stack.pop()?.try_into()?;
        let b: i32 = self.state.stack.pop()?.try_into()?;
        self.state.stack.push(a.wrapping_mul(b).into());
        Ok(())
    }

    fn i32_div_s(&mut self) -> Result<()> {
        let a: i32 = self.state.stack.pop()?.try_into()?;
        let b: i32 = self.state.stack.pop()?.try_into()?;
        if a == 0 {
            return Err(Error::msg("Division by zero"));
        }
        self.state.stack.push((b / a).into());
        Ok(())
    }

    fn local_get(&mut self, index: u32) -> Result<()> {
        let value = self.state.locals.get(index as usize)?;
        self.state.stack.push(value.clone());
        Ok(())
    }

    fn local_get_by_id(&mut self, id: &Id) -> Result<()> {
        let val = self.state.locals.get_by_id(id.name())?;
        self.state.stack.push(val.clone());
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
            Instruction::I64Const(value) => self.i64_const(*value),
            Instruction::Drop => self.drop(),
            Instruction::I32Clz => self.i32_clz(),
            Instruction::I64Clz => self.i64_clz(),
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

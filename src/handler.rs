use anyhow::{Error, Result};
use wast::{
    core::Instruction,
    token::{Id, Index},
};

use crate::{
    executor::State,
    value::{match_type, Value},
};

pub struct Handler<'a> {
    state: &'a mut State,
}

impl<'a> Handler<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Handler { state }
    }

    fn pop_i32(&mut self) -> Result<Value> {
        let val = self.state.stack.pop()?;
        match_type!(val, Value::I32)?;
        Ok(val)
    }

    fn pop_i64(&mut self) -> Result<Value> {
        let val = self.state.stack.pop()?;
        match_type!(val, Value::I64)?;
        Ok(val)
    }

    fn constant(&mut self, value: Value) -> Result<()> {
        self.state.stack.push(value);
        Ok(())
    }

    fn drop(&mut self) -> Result<()> {
        self.state.stack.pop()?;
        Ok(())
    }

    fn clz(&mut self, val: Value) -> Result<()> {
        self.state.stack.push(val.leading_zeros());
        Ok(())
    }

    fn ctz(&mut self, val: Value) -> Result<()> {
        self.state.stack.push(val.trailing_zeros());
        Ok(())
    }

    fn add(&mut self, a: Value, b: Value) -> Result<()> {
        self.state.stack.push(a.add(&b)?);
        Ok(())
    }

    fn sub(&mut self, a: Value, b: Value) -> Result<()> {
        self.state.stack.push(a.sub(&b)?);
        Ok(())
    }

    fn mul(&mut self, a: Value, b: Value) -> Result<()> {
        self.state.stack.push(a.mul(&b)?);
        Ok(())
    }

    fn div_s(&mut self, a: Value, b: Value) -> Result<()> {
        self.state.stack.push(b.div_s(&a)?);
        Ok(())
    }

    fn i32_const(&mut self, value: i32) -> Result<()> {
        self.constant(value.into())
    }

    fn i32_clz(&mut self) -> Result<()> {
        let val = self.pop_i32()?;
        self.clz(val)
    }

    fn i32_ctz(&mut self) -> Result<()> {
        let val = self.pop_i32()?;
        self.ctz(val)
    }

    fn i32_add(&mut self) -> Result<()> {
        let a = self.pop_i32()?;
        let b = self.pop_i32()?;
        self.add(a, b)
    }

    fn i32_sub(&mut self) -> Result<()> {
        let a = self.pop_i32()?;
        let b = self.pop_i32()?;
        self.sub(a, b)
    }

    fn i32_mul(&mut self) -> Result<()> {
        let a = self.pop_i32()?;
        let b = self.pop_i32()?;

        self.mul(a, b)
    }

    fn i32_div_s(&mut self) -> Result<()> {
        let a = self.pop_i32()?;
        let b = self.pop_i32()?;

        self.div_s(a, b)
    }

    fn i64_const(&mut self, value: i64) -> Result<()> {
        self.constant(value.into())
    }

    fn i64_clz(&mut self) -> Result<()> {
        let val = self.pop_i64()?;
        self.clz(val)
    }

    fn i64_ctz(&mut self) -> Result<()> {
        let val = self.pop_i64()?;
        self.ctz(val)
    }

    fn i64_add(&mut self) -> Result<()> {
        let a = self.pop_i64()?;
        let b = self.pop_i64()?;
        self.add(a, b)
    }

    fn i64_sub(&mut self) -> Result<()> {
        let a = self.pop_i64()?;
        let b = self.pop_i64()?;
        self.sub(a, b)
    }

    fn i64_mul(&mut self) -> Result<()> {
        let a = self.pop_i64()?;
        let b = self.pop_i64()?;
        self.mul(a, b)
    }

    fn i64_div_s(&mut self) -> Result<()> {
        let a = self.pop_i64()?;
        let b = self.pop_i64()?;
        self.div_s(a, b)
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
            Instruction::Drop => self.drop(),
            Instruction::I32Clz => self.i32_clz(),
            Instruction::I32Ctz => self.i32_ctz(),
            Instruction::I32Add => self.i32_add(),
            Instruction::I32Sub => self.i32_sub(),
            Instruction::I32Mul => self.i32_mul(),
            Instruction::I32DivS => self.i32_div_s(),
            Instruction::I64Const(value) => self.i64_const(*value),
            Instruction::I64Clz => self.i64_clz(),
            Instruction::I64Ctz => self.i64_ctz(),
            Instruction::I64Add => self.i64_add(),
            Instruction::I64Sub => self.i64_sub(),
            Instruction::I64Mul => self.i64_mul(),
            Instruction::I64DivS => self.i64_div_s(),
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

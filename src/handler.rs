use crate::ops::FloatOps;
use crate::ops::IntOps;
use crate::ops::NumOps;
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

    fn drop(&mut self) -> Result<()> {
        self.state.stack.pop()?;
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
            Instruction::F32Const(value) => self.f32_const(f32::from_bits(value.bits)),
            Instruction::F32Add => self.f32_add(),
            Instruction::F32Sub => self.f32_sub(),
            Instruction::F32Mul => self.f32_mul(),
            Instruction::F32Div => self.f32_div(),
            Instruction::F64Const(value) => self.f64_const(f64::from_bits(value.bits)),
            Instruction::F64Add => self.f64_add(),
            Instruction::F64Sub => self.f64_sub(),
            Instruction::F64Mul => self.f64_mul(),
            Instruction::F64Div => self.f64_div(),
            Instruction::LocalGet(Index::Num(index, _)) => self.local_get(*index),
            Instruction::LocalGet(Index::Id(id)) => self.local_get_by_id(id),
            Instruction::LocalSet(Index::Num(index, _)) => self.local_set(*index),
            Instruction::LocalSet(Index::Id(id)) => self.local_set_by_id(id),
            _ => Err(Error::msg("Unknown instruction")),
        }
    }
}

macro_rules! pop {
    ($fname:ident, $ty:ty) => {
        impl<'a> Handler<'a> {
            fn $fname(&mut self) -> Result<$ty> {
                let val: $ty = self.state.stack.pop()?.try_into()?;
                Ok(val)
            }
        }
    };
}

pop!(i32_pop, i32);
pop!(i64_pop, i64);
pop!(f32_pop, f32);
pop!(f64_pop, f64);

macro_rules! constant {
    ($fname:ident, $ty:ty) => {
        impl<'a> Handler<'a> {
            fn $fname(&mut self, value: $ty) -> Result<()> {
                self.state.stack.push(value.into());
                Ok(())
            }
        }
    };
}

constant!(i32_const, i32);
constant!(i64_const, i64);
constant!(f32_const, f32);
constant!(f64_const, f64);

macro_rules! impl_binary_op {
    ($fname:ident, $popper:ident, $op:ident) => {
        impl<'a> Handler<'a> {
            fn $fname(&mut self) -> Result<()> {
                let a = self.$popper()?;
                let b = self.$popper()?;
                self.state.stack.push(b.$op(a).into());
                Ok(())
            }
        }
    };
}

impl_binary_op!(i32_add, i32_pop, add);
impl_binary_op!(i32_sub, i32_pop, sub);
impl_binary_op!(i32_mul, i32_pop, mul);

impl_binary_op!(i64_add, i64_pop, add);
impl_binary_op!(i64_sub, i64_pop, sub);
impl_binary_op!(i64_mul, i64_pop, mul);

impl_binary_op!(f32_add, f32_pop, add);
impl_binary_op!(f32_sub, f32_pop, sub);
impl_binary_op!(f32_mul, f32_pop, mul);
impl_binary_op!(f32_div, f32_pop, div);

impl_binary_op!(f64_add, f64_pop, add);
impl_binary_op!(f64_sub, f64_pop, sub);
impl_binary_op!(f64_mul, f64_pop, mul);
impl_binary_op!(f64_div, f64_pop, div);

macro_rules! impl_binary_res_op {
    ($fname:ident, $popper:ident, $op:ident) => {
        impl<'a> Handler<'a> {
            fn $fname(&mut self) -> Result<()> {
                let a = self.$popper()?;
                let b = self.$popper()?;
                self.state.stack.push(b.$op(a)?.into());
                Ok(())
            }
        }
    };
}

impl_binary_res_op!(i32_div_s, i32_pop, div);
impl_binary_res_op!(i64_div_s, i64_pop, div);

macro_rules! impl_unary_op {
    ($fname:ident, $popper:ident, $op:ident) => {
        impl<'a> Handler<'a> {
            fn $fname(&mut self) -> Result<()> {
                let a = self.$popper()?;
                self.state.stack.push(a.$op().into());
                Ok(())
            }
        }
    };
}

impl_unary_op!(i32_clz, i32_pop, clz);
impl_unary_op!(i32_ctz, i32_pop, ctz);

impl_unary_op!(i64_clz, i64_pop, clz);
impl_unary_op!(i64_ctz, i64_pop, ctz);

#[cfg(test)]
#[path = "./handler_test.rs"]
mod handler_test;

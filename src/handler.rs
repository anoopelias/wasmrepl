use std::ops::BitAnd;
use std::ops::BitOr;

use crate::ops::FloatOps;
use crate::ops::IntOps;
use crate::ops::NumOps;
use anyhow::{Error, Result};
use std::ops::{BitXor, Shl};
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
            Instruction::I32Popcnt => self.i32_popcnt(),
            Instruction::I32Add => self.i32_add(),
            Instruction::I32Sub => self.i32_sub(),
            Instruction::I32Mul => self.i32_mul(),
            Instruction::I32DivS => self.i32_div_s(),
            Instruction::I32DivU => self.i32_div_u(),
            Instruction::I32RemS => self.i32_rem_s(),
            Instruction::I32RemU => self.i32_rem_u(),
            Instruction::I32And => self.i32_and(),
            Instruction::I32Or => self.i32_or(),
            Instruction::I32Xor => self.i32_xor(),
            Instruction::I32Shl => self.i32_shl(),
            Instruction::I32ShrS => self.i32_shr_s(),
            Instruction::I32ShrU => self.i32_shr_u(),
            Instruction::I32Rotl => self.i32_rotl(),
            Instruction::I32Rotr => self.i32_rotr(),
            Instruction::I64Const(value) => self.i64_const(*value),
            Instruction::I64Clz => self.i64_clz(),
            Instruction::I64Ctz => self.i64_ctz(),
            Instruction::I64Popcnt => self.i64_popcnt(),
            Instruction::I64Add => self.i64_add(),
            Instruction::I64Sub => self.i64_sub(),
            Instruction::I64Mul => self.i64_mul(),
            Instruction::I64DivS => self.i64_div_s(),
            Instruction::I64DivU => self.i64_div_u(),
            Instruction::I64RemS => self.i64_rem_s(),
            Instruction::I64RemU => self.i64_rem_u(),
            Instruction::I64And => self.i64_and(),
            Instruction::I64Or => self.i64_or(),
            Instruction::I64Xor => self.i64_xor(),
            Instruction::I64Shl => self.i64_shl(),
            Instruction::I64ShrS => self.i64_shr_s(),
            Instruction::I64ShrU => self.i64_shr_u(),
            Instruction::I64Rotl => self.i64_rotl(),
            Instruction::I64Rotr => self.i64_rotr(),
            Instruction::F32Const(value) => self.f32_const(f32::from_bits(value.bits)),
            Instruction::F32Abs => self.f32_abs(),
            Instruction::F32Neg => self.f32_neg(),
            Instruction::F32Ceil => self.f32_ceil(),
            Instruction::F32Floor => self.f32_floor(),
            Instruction::F32Trunc => self.f32_trunc(),
            Instruction::F32Nearest => self.f32_nearest(),
            Instruction::F32Sqrt => self.f32_sqrt(),
            Instruction::F32Add => self.f32_add(),
            Instruction::F32Sub => self.f32_sub(),
            Instruction::F32Mul => self.f32_mul(),
            Instruction::F32Div => self.f32_div(),
            Instruction::F32Min => self.f32_min(),
            Instruction::F32Max => self.f32_max(),
            Instruction::F32Copysign => self.f32_copysign(),
            Instruction::F64Const(value) => self.f64_const(f64::from_bits(value.bits)),
            Instruction::F64Abs => self.f64_abs(),
            Instruction::F64Neg => self.f64_neg(),
            Instruction::F64Ceil => self.f64_ceil(),
            Instruction::F64Floor => self.f64_floor(),
            Instruction::F64Trunc => self.f64_trunc(),
            Instruction::F64Nearest => self.f64_nearest(),
            Instruction::F64Sqrt => self.f64_sqrt(),
            Instruction::F64Add => self.f64_add(),
            Instruction::F64Sub => self.f64_sub(),
            Instruction::F64Mul => self.f64_mul(),
            Instruction::F64Div => self.f64_div(),
            Instruction::F64Min => self.f64_min(),
            Instruction::F64Max => self.f64_max(),
            Instruction::F64Copysign => self.f64_copysign(),
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
    ($fname:ident, $pop:ident, $op:ident) => {
        impl<'a> Handler<'a> {
            fn $fname(&mut self) -> Result<()> {
                let a = self.$pop()?;
                let b = self.$pop()?;
                self.state.stack.push(b.$op(a).into());
                Ok(())
            }
        }
    };
}

impl_binary_op!(i32_add, i32_pop, add);
impl_binary_op!(i32_sub, i32_pop, sub);
impl_binary_op!(i32_mul, i32_pop, mul);
impl_binary_op!(i32_and, i32_pop, bitand);
impl_binary_op!(i32_or, i32_pop, bitor);
impl_binary_op!(i32_xor, i32_pop, bitxor);
impl_binary_op!(i32_shl, i32_pop, shl);
impl_binary_op!(i32_shr_s, i32_pop, shr_s);
impl_binary_op!(i32_shr_u, i32_pop, shr_u);
impl_binary_op!(i32_rotl, i32_pop, rotl);
impl_binary_op!(i32_rotr, i32_pop, rotr);

impl_binary_op!(i64_add, i64_pop, add);
impl_binary_op!(i64_sub, i64_pop, sub);
impl_binary_op!(i64_mul, i64_pop, mul);
impl_binary_op!(i64_and, i64_pop, bitand);
impl_binary_op!(i64_or, i64_pop, bitor);
impl_binary_op!(i64_xor, i64_pop, bitxor);
impl_binary_op!(i64_shl, i64_pop, shl);
impl_binary_op!(i64_shr_s, i64_pop, shr_s);
impl_binary_op!(i64_shr_u, i64_pop, shr_u);
impl_binary_op!(i64_rotl, i64_pop, rotl);
impl_binary_op!(i64_rotr, i64_pop, rotr);

impl_binary_op!(f32_add, f32_pop, add);
impl_binary_op!(f32_sub, f32_pop, sub);
impl_binary_op!(f32_mul, f32_pop, mul);
impl_binary_op!(f32_div, f32_pop, div);
impl_binary_op!(f32_min, f32_pop, min);
impl_binary_op!(f32_max, f32_pop, max);
impl_binary_op!(f32_copysign, f32_pop, copysign);

impl_binary_op!(f64_add, f64_pop, add);
impl_binary_op!(f64_sub, f64_pop, sub);
impl_binary_op!(f64_mul, f64_pop, mul);
impl_binary_op!(f64_div, f64_pop, div);
impl_binary_op!(f64_min, f64_pop, min);
impl_binary_op!(f64_max, f64_pop, max);
impl_binary_op!(f64_copysign, f64_pop, copysign);

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

impl_binary_res_op!(i32_div_s, i32_pop, div_s);
impl_binary_res_op!(i32_div_u, i32_pop, div_u);
impl_binary_res_op!(i32_rem_s, i32_pop, rem_s);
impl_binary_res_op!(i32_rem_u, i32_pop, rem_u);

impl_binary_res_op!(i64_div_s, i64_pop, div_s);
impl_binary_res_op!(i64_div_u, i64_pop, div_u);
impl_binary_res_op!(i64_rem_s, i64_pop, rem_s);
impl_binary_res_op!(i64_rem_u, i64_pop, rem_u);

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
impl_unary_op!(i32_popcnt, i32_pop, popcnt);

impl_unary_op!(i64_clz, i64_pop, clz);
impl_unary_op!(i64_ctz, i64_pop, ctz);
impl_unary_op!(i64_popcnt, i64_pop, popcnt);

impl_unary_op!(f32_abs, f32_pop, abs);
impl_unary_op!(f32_neg, f32_pop, neg);
impl_unary_op!(f32_ceil, f32_pop, ceil);
impl_unary_op!(f32_floor, f32_pop, floor);
impl_unary_op!(f32_trunc, f32_pop, trunc);
impl_unary_op!(f32_nearest, f32_pop, round);
impl_unary_op!(f32_sqrt, f32_pop, sqrt);

impl_unary_op!(f64_abs, f64_pop, abs);
impl_unary_op!(f64_neg, f64_pop, neg);
impl_unary_op!(f64_ceil, f64_pop, ceil);
impl_unary_op!(f64_floor, f64_pop, floor);
impl_unary_op!(f64_trunc, f64_pop, trunc);
impl_unary_op!(f64_nearest, f64_pop, round);
impl_unary_op!(f64_sqrt, f64_pop, sqrt);

#[cfg(test)]
#[path = "./handler_test.rs"]
mod handler_test;

use anyhow::Result;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::{BitXor, Shl};

use crate::call_stack::FuncStack;
use crate::model::BlockType;
use crate::model::Expression;
use crate::model::{Index, Instruction};
use crate::ops::FloatOps;
use crate::ops::IntOps;
use crate::ops::NumOps;
use crate::response::Control;
use crate::response::Response;

pub struct Handler<'a> {
    stack: &'a mut FuncStack,
}

impl<'a> Handler<'a> {
    pub fn new(state: &'a mut FuncStack) -> Self {
        Handler { stack: state }
    }

    fn drop(&mut self) -> Result<Response> {
        self.stack.pop()?;
        Ok(Response::new())
    }

    fn local_get(&mut self, index: &Index) -> Result<Response> {
        let value = self.stack.locals.get(index)?;
        self.stack.push(value.clone())?;
        Ok(Response::new())
    }

    fn local_set(&mut self, index: &Index) -> Result<Response> {
        let value = self.stack.pop()?;
        self.stack.locals.set(index, value)?;
        Ok(Response::new())
    }

    fn local_tee(&mut self, index: &Index) -> Result<Response> {
        let value = self.stack.peek()?;
        self.stack.locals.set(index, value)?;
        Ok(Response::new())
    }

    fn return_instr(&mut self) -> Result<Response> {
        Ok(Response::new_ctrl(Control::Return))
    }

    fn nop(&mut self) -> Result<Response> {
        Ok(Response::new())
    }

    fn call_func(&mut self, index: Index) -> Result<Response> {
        Ok(Response::new_ctrl(Control::ExecFunc(index)))
    }

    fn if_instr(
        &mut self,
        block_type: BlockType,
        if_block: Option<Expression>,
        else_block: Option<Expression>,
    ) -> Result<Response> {
        let value = self.stack.pop()?;
        if value.is_true() {
            Ok(Response::new_ctrl(Control::ExecBlock(
                block_type,
                if_block.unwrap(),
            )))
        } else {
            Ok(Response::new_ctrl(Control::ExecBlock(
                block_type,
                else_block.unwrap(),
            )))
        }
    }

    fn block(&mut self, block_type: BlockType, block: Option<Expression>) -> Result<Response> {
        Ok(Response::new_ctrl(Control::ExecBlock(
            block_type,
            block.unwrap(),
        )))
    }

    fn branch(&mut self, index: Index) -> Result<Response> {
        Ok(Response::new_ctrl(Control::Branch(index)))
    }

    fn handle_loop(
        &mut self,
        block_type: BlockType,
        block: Option<Expression>,
    ) -> Result<Response> {
        Ok(Response::new_ctrl(Control::ExecLoop(
            block_type,
            block.unwrap(),
        )))
    }

    pub fn handle(&mut self, instr: Instruction) -> Result<Response> {
        match instr {
            Instruction::I32Const(value) => self.i32_const(value),
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
            Instruction::I32Eqz => self.i32_eqz(),
            Instruction::I32Eq => self.i32_eq(),
            Instruction::I32Ne => self.i32_ne(),
            Instruction::I32LtS => self.i32_lt_s(),
            Instruction::I32LtU => self.i32_lt_u(),
            Instruction::I32GtS => self.i32_gt_s(),
            Instruction::I32GtU => self.i32_gt_u(),
            Instruction::I32LeS => self.i32_le_s(),
            Instruction::I32LeU => self.i32_le_u(),
            Instruction::I32GeS => self.i32_ge_s(),
            Instruction::I32GeU => self.i32_ge_u(),
            Instruction::I64Const(value) => self.i64_const(value),
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
            Instruction::I64Eqz => self.i64_eqz(),
            Instruction::I64Eq => self.i64_eq(),
            Instruction::I64Ne => self.i64_ne(),
            Instruction::I64LtS => self.i64_lt_s(),
            Instruction::I64LtU => self.i64_lt_u(),
            Instruction::I64GtS => self.i64_gt_s(),
            Instruction::I64GtU => self.i64_gt_u(),
            Instruction::I64LeS => self.i64_le_s(),
            Instruction::I64LeU => self.i64_le_u(),
            Instruction::I64GeS => self.i64_ge_s(),
            Instruction::I64GeU => self.i64_ge_u(),
            Instruction::F32Const(value) => self.f32_const(value),
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
            Instruction::F32Eq => self.f32_eq(),
            Instruction::F32Ne => self.f32_ne(),
            Instruction::F32Lt => self.f32_lt(),
            Instruction::F32Gt => self.f32_gt(),
            Instruction::F32Le => self.f32_le(),
            Instruction::F32Ge => self.f32_ge(),
            Instruction::F64Const(value) => self.f64_const(value),
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
            Instruction::F64Eq => self.f64_eq(),
            Instruction::F64Ne => self.f64_ne(),
            Instruction::F64Lt => self.f64_lt(),
            Instruction::F64Gt => self.f64_gt(),
            Instruction::F64Le => self.f64_le(),
            Instruction::F64Ge => self.f64_ge(),
            Instruction::LocalGet(index) => self.local_get(&index),
            Instruction::LocalSet(index) => self.local_set(&index),
            Instruction::LocalTee(index) => self.local_tee(&index),
            Instruction::Return => self.return_instr(),
            Instruction::Nop => self.nop(),
            Instruction::Call(index) => self.call_func(index),
            Instruction::If(bt, ib, eb) => self.if_instr(bt, ib, eb),
            Instruction::Else => unreachable!(),
            Instruction::End => unreachable!(),
            Instruction::Block(bt, b) => self.block(bt, b),
            Instruction::Br(index) => self.branch(index),
            Instruction::Loop(bt, b) => self.handle_loop(bt, b),
        }
    }
}

macro_rules! pop {
    ($fname:ident, $ty:ty) => {
        impl<'a> Handler<'a> {
            fn $fname(&mut self) -> Result<$ty> {
                let val: $ty = self.stack.pop()?.try_into()?;
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
            fn $fname(&mut self, value: $ty) -> Result<Response> {
                self.stack.push(value.into())?;
                Ok(Response::new())
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
            fn $fname(&mut self) -> Result<Response> {
                let a = self.$pop()?;
                let b = self.$pop()?;
                self.stack.push(b.$op(a).into())?;
                Ok(Response::new())
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
impl_binary_op!(i32_eq, i32_pop, eq);
impl_binary_op!(i32_ne, i32_pop, ne);
impl_binary_op!(i32_lt_s, i32_pop, lt_s);
impl_binary_op!(i32_lt_u, i32_pop, lt_u);
impl_binary_op!(i32_gt_s, i32_pop, gt_s);
impl_binary_op!(i32_gt_u, i32_pop, gt_u);
impl_binary_op!(i32_le_s, i32_pop, le_s);
impl_binary_op!(i32_le_u, i32_pop, le_u);
impl_binary_op!(i32_ge_s, i32_pop, ge_s);
impl_binary_op!(i32_ge_u, i32_pop, ge_u);

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
impl_binary_op!(i64_eq, i64_pop, eq);
impl_binary_op!(i64_ne, i64_pop, ne);
impl_binary_op!(i64_lt_s, i64_pop, lt_s);
impl_binary_op!(i64_lt_u, i64_pop, lt_u);
impl_binary_op!(i64_gt_s, i64_pop, gt_s);
impl_binary_op!(i64_gt_u, i64_pop, gt_u);
impl_binary_op!(i64_le_s, i64_pop, le_s);
impl_binary_op!(i64_le_u, i64_pop, le_u);
impl_binary_op!(i64_ge_s, i64_pop, ge_s);
impl_binary_op!(i64_ge_u, i64_pop, ge_u);

impl_binary_op!(f32_add, f32_pop, add);
impl_binary_op!(f32_sub, f32_pop, sub);
impl_binary_op!(f32_mul, f32_pop, mul);
impl_binary_op!(f32_div, f32_pop, div);
impl_binary_op!(f32_min, f32_pop, min);
impl_binary_op!(f32_max, f32_pop, max);
impl_binary_op!(f32_copysign, f32_pop, copysign);
impl_binary_op!(f32_eq, f32_pop, eq);
impl_binary_op!(f32_ne, f32_pop, ne);
impl_binary_op!(f32_lt, f32_pop, lt);
impl_binary_op!(f32_gt, f32_pop, gt);
impl_binary_op!(f32_le, f32_pop, le);
impl_binary_op!(f32_ge, f32_pop, ge);

impl_binary_op!(f64_add, f64_pop, add);
impl_binary_op!(f64_sub, f64_pop, sub);
impl_binary_op!(f64_mul, f64_pop, mul);
impl_binary_op!(f64_min, f64_pop, min);
impl_binary_op!(f64_div, f64_pop, div);
impl_binary_op!(f64_max, f64_pop, max);
impl_binary_op!(f64_copysign, f64_pop, copysign);
impl_binary_op!(f64_eq, f64_pop, eq);
impl_binary_op!(f64_ne, f64_pop, ne);
impl_binary_op!(f64_lt, f64_pop, lt);
impl_binary_op!(f64_gt, f64_pop, gt);
impl_binary_op!(f64_le, f64_pop, le);
impl_binary_op!(f64_ge, f64_pop, ge);

macro_rules! impl_binary_res_op {
    ($fname:ident, $popper:ident, $op:ident) => {
        impl<'a> Handler<'a> {
            fn $fname(&mut self) -> Result<Response> {
                let a = self.$popper()?;
                let b = self.$popper()?;
                self.stack.push(b.$op(a)?.into())?;
                Ok(Response::new())
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
            fn $fname(&mut self) -> Result<Response> {
                let a = self.$popper()?;
                self.stack.push(a.$op().into())?;
                Ok(Response::new())
            }
        }
    };
}

impl_unary_op!(i32_clz, i32_pop, clz);
impl_unary_op!(i32_ctz, i32_pop, ctz);
impl_unary_op!(i32_popcnt, i32_pop, popcnt);
impl_unary_op!(i32_eqz, i32_pop, eqz);

impl_unary_op!(i64_clz, i64_pop, clz);
impl_unary_op!(i64_ctz, i64_pop, ctz);
impl_unary_op!(i64_popcnt, i64_pop, popcnt);
impl_unary_op!(i64_eqz, i64_pop, eqz);

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

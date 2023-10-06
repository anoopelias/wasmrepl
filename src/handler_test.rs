use crate::call_stack::FuncStack;
use crate::response::{Control, Response};
use crate::value::Value;
use anyhow::Result;

use crate::model::{Expression, FuncType, Index, Instruction, Local, ValType};
use crate::test_utils::{
    test_block, test_block_type, test_func_type, test_if, test_index, test_local, test_loop,
};

use super::Handler;

fn exec_instr_handler(instr: Instruction, stack: &mut FuncStack) -> Result<Response> {
    let mut handler = Handler::new(stack);
    handler.handle(instr)
}

#[test]
fn test_i32_const() {
    let mut stack = FuncStack::new();
    exec_instr_handler(Instruction::I32Const(42), &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 42.into());
}

#[test]
fn test_i64_const() {
    let mut stack = FuncStack::new();
    exec_instr_handler(Instruction::I64Const(52), &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 52i64.into());
}

#[test]
fn test_drop() {
    let mut stack = FuncStack::new();
    stack.push(42.into()).unwrap();
    exec_instr_handler(Instruction::Drop, &mut stack).unwrap();
    assert!(stack.pop().is_err());
}

#[test]
fn test_drop_error() {
    let mut stack = FuncStack::new();
    assert!(exec_instr_handler(Instruction::Drop, &mut stack).is_err());
}

#[test]
fn test_i32_clz() {
    let mut stack = FuncStack::new();
    stack.push(1023.into()).unwrap();
    exec_instr_handler(Instruction::I32Clz, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 22.into());
}

#[test]
fn test_i32_clz_error() {
    let mut stack = FuncStack::new();
    assert!(exec_instr_handler(Instruction::I32Clz, &mut stack).is_err());
}

#[test]
fn test_i32_clz_type_error() {
    let mut stack = FuncStack::new();
    stack.push(1023i64.into()).unwrap();
    assert!(exec_instr_handler(Instruction::I32Clz, &mut stack).is_err());
}

#[test]
fn test_i32_ctz() {
    let mut stack = FuncStack::new();
    stack.push(1024.into()).unwrap();
    exec_instr_handler(Instruction::I32Ctz, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 10.into());
}

#[test]
fn test_i32_ctz_error() {
    let mut stack = FuncStack::new();
    assert!(exec_instr_handler(Instruction::I32Ctz, &mut stack).is_err());
}

#[test]
fn test_i32_popcnt() {
    let mut stack = FuncStack::new();
    stack.push(1023.into()).unwrap();
    exec_instr_handler(Instruction::I32Popcnt, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 10.into());
}

#[test]
fn test_i32_add() {
    let mut stack = FuncStack::new();
    stack.push(1.into()).unwrap();
    stack.push(2.into()).unwrap();
    exec_instr_handler(Instruction::I32Add, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 3.into());
}

#[test]
fn test_i32_add_error() {
    let mut stack = FuncStack::new();
    stack.push(1.into()).unwrap();
    assert!(exec_instr_handler(Instruction::I32Add, &mut stack).is_err());
}

#[test]
fn test_i32_add_type_error() {
    let mut stack = FuncStack::new();
    stack.push(1i64.into()).unwrap();
    stack.push(2.into()).unwrap();
    assert!(exec_instr_handler(Instruction::I32Add, &mut stack).is_err());
}

#[test]
fn test_i32_sub() {
    let mut stack = FuncStack::new();
    stack.push(2.into()).unwrap();
    stack.push(1.into()).unwrap();
    exec_instr_handler(Instruction::I32Sub, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), Value::from(1));
}

#[test]
fn test_i32_sub_overflow() {
    let mut stack = FuncStack::new();
    stack.push(i32::MAX.into()).unwrap();
    stack.push(Value::from(-1)).unwrap();
    exec_instr_handler(Instruction::I32Sub, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), Value::from(-2147483648));
}

#[test]
fn test_i32_sub_error() {
    let mut stack = FuncStack::new();
    stack.push(1.into()).unwrap();
    assert!(exec_instr_handler(Instruction::I32Sub, &mut stack).is_err());
}

#[test]
fn test_i32_mul() {
    let mut stack = FuncStack::new();
    stack.push(2.into()).unwrap();
    stack.push(3.into()).unwrap();
    exec_instr_handler(Instruction::I32Mul, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 6.into());
}

#[test]
fn test_i32_mul_overflow() {
    let mut stack = FuncStack::new();
    stack.push(i32::MAX.into()).unwrap();
    stack.push(3.into()).unwrap();
    exec_instr_handler(Instruction::I32Mul, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 2147483645.into());
}

#[test]
fn test_i32_mul_error() {
    let mut stack = FuncStack::new();
    stack.push(1.into()).unwrap();
    assert!(exec_instr_handler(Instruction::I32Mul, &mut stack).is_err());
}

#[test]
fn test_i32_div_s() {
    let mut stack = FuncStack::new();
    stack.push(i32::MIN.into()).unwrap();
    stack.push(2.into()).unwrap();
    exec_instr_handler(Instruction::I32DivU, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0x40000000.into());
}

#[test]
fn test_i32_div_s_error() {
    let mut stack = FuncStack::new();
    stack.push(1.into()).unwrap();
    assert!(exec_instr_handler(Instruction::I32DivS, &mut stack).is_err());
}

#[test]
fn test_i32_div_s_type_error() {
    let mut stack = FuncStack::new();
    stack.push(1i64.into()).unwrap();
    stack.push(2.into()).unwrap();
    assert!(exec_instr_handler(Instruction::I32DivS, &mut stack).is_err());
}

#[test]
fn test_i32_div_u() {
    let mut stack = FuncStack::new();
    stack.push(7.into()).unwrap();
    stack.push(3.into()).unwrap();
    exec_instr_handler(Instruction::I32DivU, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 2.into());
}

#[test]
fn test_i32_rem_s() {
    let mut stack = FuncStack::new();
    stack.push(7.into()).unwrap();
    stack.push(3.into()).unwrap();
    exec_instr_handler(Instruction::I32RemS, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 1.into());
}

#[test]
fn test_i32_rem_u() {
    let mut stack = FuncStack::new();
    stack.push(i32::MIN.into()).unwrap();
    stack.push((-1).into()).unwrap();
    exec_instr_handler(Instruction::I32RemU, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), i32::MIN.into());
}

#[test]
fn test_i32_and() {
    let mut stack = FuncStack::new();
    stack.push(0b1010.into()).unwrap();
    stack.push(0b1100.into()).unwrap();
    exec_instr_handler(Instruction::I32And, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0b1000.into());
}

#[test]
fn test_i32_or() {
    let mut stack = FuncStack::new();
    stack.push(0b1010.into()).unwrap();
    stack.push(0b1100.into()).unwrap();
    exec_instr_handler(Instruction::I32Or, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0b1110.into());
}

#[test]
fn test_i32_xor() {
    let mut stack = FuncStack::new();
    stack.push(0b1010.into()).unwrap();
    stack.push(0b1100.into()).unwrap();
    exec_instr_handler(Instruction::I32Xor, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0b0110.into());
}

#[test]
fn test_i32_shl() {
    let mut stack = FuncStack::new();
    stack.push(0b1010.into()).unwrap();
    stack.push(2.into()).unwrap();
    exec_instr_handler(Instruction::I32Shl, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0b101000.into());
}

#[test]
fn test_i32_shr_s() {
    let mut stack = FuncStack::new();
    stack.push(i32::MIN.into()).unwrap();
    stack.push(1.into()).unwrap();
    exec_instr_handler(Instruction::I32ShrS, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (-1073741824).into());
}

#[test]
fn test_i32_shr_u() {
    let mut stack = FuncStack::new();
    stack.push(i32::MIN.into()).unwrap();
    stack.push(1.into()).unwrap();
    exec_instr_handler(Instruction::I32ShrU, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0x40000000.into());
}

#[test]
fn test_i32_rotl() {
    let mut stack = FuncStack::new();
    stack.push(0x10008002.into()).unwrap();
    stack.push(2.into()).unwrap();
    exec_instr_handler(Instruction::I32Rotl, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0x40020008.into());
}

#[test]
fn test_i32_rotr() {
    let mut stack = FuncStack::new();
    stack.push(0x40020008.into()).unwrap();
    stack.push(2.into()).unwrap();
    exec_instr_handler(Instruction::I32Rotr, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0x10008002.into());
}

#[test]
fn test_i64_clz() {
    let mut stack = FuncStack::new();
    stack.push(1023i64.into()).unwrap();
    exec_instr_handler(Instruction::I64Clz, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 54i64.into());
}

#[test]
fn test_i64_ctz() {
    let mut stack = FuncStack::new();
    stack.push(1024i64.into()).unwrap();
    exec_instr_handler(Instruction::I64Ctz, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 10i64.into());
}

#[test]
fn test_i64_popcnt() {
    let mut stack = FuncStack::new();
    stack.push(1023i64.into()).unwrap();
    exec_instr_handler(Instruction::I64Popcnt, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 10i64.into());
}

#[test]
fn test_i64_add() {
    let mut stack = FuncStack::new();
    stack.push(1i64.into()).unwrap();
    stack.push(2i64.into()).unwrap();
    exec_instr_handler(Instruction::I64Add, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 3i64.into());
}

#[test]
fn test_i64_sub() {
    let mut stack = FuncStack::new();
    stack.push(2i64.into()).unwrap();
    stack.push(1i64.into()).unwrap();
    exec_instr_handler(Instruction::I64Sub, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 1i64.into());
}

#[test]
fn test_i64_mul() {
    let mut stack = FuncStack::new();
    stack.push(2i64.into()).unwrap();
    stack.push(3i64.into()).unwrap();
    exec_instr_handler(Instruction::I64Mul, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 6i64.into());
}

#[test]
fn test_i64_div_s() {
    let mut stack = FuncStack::new();
    stack.push(7i64.into()).unwrap();
    stack.push(3i64.into()).unwrap();
    exec_instr_handler(Instruction::I64DivS, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 2i64.into());
}

#[test]
fn test_i64_div_u() {
    let mut stack = FuncStack::new();
    stack.push(i64::MIN.into()).unwrap();
    stack.push(2i64.into()).unwrap();
    exec_instr_handler(Instruction::I64DivU, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0x4000000000000000i64.into());
}

#[test]
fn test_i64_rem_s() {
    let mut stack = FuncStack::new();
    stack.push(7i64.into()).unwrap();
    stack.push(3i64.into()).unwrap();
    exec_instr_handler(Instruction::I64RemS, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 1i64.into());
}

#[test]
fn test_i64_rem_u() {
    let mut stack = FuncStack::new();
    stack.push(i64::MIN.into()).unwrap();
    stack.push((-1i64).into()).unwrap();
    exec_instr_handler(Instruction::I64RemU, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), i64::MIN.into());
}

#[test]
fn test_i64_and() {
    let mut stack = FuncStack::new();
    stack.push(0b1010i64.into()).unwrap();
    stack.push(0b1100i64.into()).unwrap();
    exec_instr_handler(Instruction::I64And, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0b1000i64.into());
}

#[test]
fn test_i64_or() {
    let mut stack = FuncStack::new();
    stack.push(0b1010i64.into()).unwrap();
    stack.push(0b1100i64.into()).unwrap();
    exec_instr_handler(Instruction::I64Or, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0b1110i64.into());
}

#[test]
fn test_i64_xor() {
    let mut stack = FuncStack::new();
    stack.push(0b1010i64.into()).unwrap();
    stack.push(0b1100i64.into()).unwrap();
    exec_instr_handler(Instruction::I64Xor, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0b0110i64.into());
}

#[test]
fn test_i64_shl() {
    let mut stack = FuncStack::new();
    stack.push(0b1010i64.into()).unwrap();
    stack.push(2i64.into()).unwrap();
    exec_instr_handler(Instruction::I64Shl, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0b101000i64.into());
}

#[test]
fn test_i64_shr_u() {
    let mut stack = FuncStack::new();
    stack.push(i64::MIN.into()).unwrap();
    stack.push(1i64.into()).unwrap();
    exec_instr_handler(Instruction::I64ShrU, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0x4000000000000000i64.into());
}

#[test]
fn test_i64_shr_s() {
    let mut stack = FuncStack::new();
    stack.push(i64::MIN.into()).unwrap();
    stack.push(1i64.into()).unwrap();
    exec_instr_handler(Instruction::I64ShrS, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (-4611686018427387904i64).into());
}

#[test]
fn test_i64_rotl() {
    let mut stack = FuncStack::new();
    stack.push(0x10008002i64.into()).unwrap();
    stack.push(2i64.into()).unwrap();
    exec_instr_handler(Instruction::I64Rotl, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0x40020008i64.into());
}

#[test]
fn test_i64_rotr() {
    let mut stack = FuncStack::new();
    stack.push(0x40020008i64.into()).unwrap();
    stack.push(2i64.into()).unwrap();
    exec_instr_handler(Instruction::I64Rotr, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0x10008002i64.into());
}

#[test]
fn test_f32_const() {
    let mut stack = FuncStack::new();
    exec_instr_handler(Instruction::F32Const(3.14), &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 3.14f32.into());
}

#[test]
fn test_f32_abs() {
    let mut stack = FuncStack::new();
    stack.push((-2.5f32).into()).unwrap();
    exec_instr_handler(Instruction::F32Abs, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 2.5f32.into());
}

#[test]
fn test_f32_neg() {
    let mut stack = FuncStack::new();
    stack.push(2.5f32.into()).unwrap();
    exec_instr_handler(Instruction::F32Neg, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (-2.5f32).into());
}

#[test]
fn test_f32_ceil() {
    let mut stack = FuncStack::new();
    stack.push((-2.5f32).into()).unwrap();
    exec_instr_handler(Instruction::F32Ceil, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (-2.0f32).into());
}

#[test]
fn test_f32_floor() {
    let mut stack = FuncStack::new();
    stack.push(2.5f32.into()).unwrap();
    exec_instr_handler(Instruction::F32Floor, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (2.0f32).into());
}

#[test]
fn test_f32_trunc() {
    let mut stack = FuncStack::new();
    stack.push(2.5f32.into()).unwrap();
    exec_instr_handler(Instruction::F32Trunc, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (2.0f32).into());
}

#[test]
fn test_f32_nearest() {
    let mut stack = FuncStack::new();
    stack.push(2.5f32.into()).unwrap();
    exec_instr_handler(Instruction::F32Nearest, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (3.0f32).into());
}

#[test]
fn test_f32_sqrt() {
    let mut stack = FuncStack::new();
    stack.push(4.0f32.into()).unwrap();
    exec_instr_handler(Instruction::F32Sqrt, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (2.0f32).into());
}

#[test]
fn test_f32_add() {
    let mut stack = FuncStack::new();
    stack.push(1.0f32.into()).unwrap();
    stack.push(2.0f32.into()).unwrap();
    exec_instr_handler(Instruction::F32Add, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 3.0f32.into());
}

#[test]
fn test_f32_sub() {
    let mut stack = FuncStack::new();
    stack.push(2.0f32.into()).unwrap();
    stack.push(1.5f32.into()).unwrap();
    exec_instr_handler(Instruction::F32Sub, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0.5f32.into());
}

#[test]
fn test_f32_mul() {
    let mut stack = FuncStack::new();
    stack.push(2.5f32.into()).unwrap();
    stack.push(2.0f32.into()).unwrap();
    exec_instr_handler(Instruction::F32Mul, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 5.0f32.into());
}

#[test]
fn test_f32_div() {
    let mut stack = FuncStack::new();
    stack.push(7.0f32.into()).unwrap();
    stack.push(2.0f32.into()).unwrap();
    exec_instr_handler(Instruction::F32Div, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 3.5f32.into());
}

#[test]
fn test_f32_min() {
    let mut stack = FuncStack::new();
    stack.push(2.0f32.into()).unwrap();
    stack.push(3.0f32.into()).unwrap();
    exec_instr_handler(Instruction::F32Min, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 2.0f32.into());
}

#[test]
fn test_f32_max() {
    let mut stack = FuncStack::new();
    stack.push(2.0f32.into()).unwrap();
    stack.push(3.0f32.into()).unwrap();
    exec_instr_handler(Instruction::F32Max, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 3.0f32.into());
}

#[test]
fn test_f32_copysign() {
    let mut stack = FuncStack::new();
    stack.push(2.0f32.into()).unwrap();
    stack.push((-3.0f32).into()).unwrap();
    exec_instr_handler(Instruction::F32Copysign, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (-2.0f32).into());
}

#[test]
fn test_f64_const() {
    let mut stack = FuncStack::new();
    exec_instr_handler(Instruction::F64Const(3.14), &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 3.14f64.into());
}

#[test]
fn test_f64_abs() {
    let mut stack = FuncStack::new();
    stack.push((-2.5f64).into()).unwrap();
    exec_instr_handler(Instruction::F64Abs, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 2.5f64.into());
}

#[test]
fn test_f64_neg() {
    let mut stack = FuncStack::new();
    stack.push(2.5f64.into()).unwrap();
    exec_instr_handler(Instruction::F64Neg, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (-2.5f64).into());
}

#[test]
fn test_f64_ceil() {
    let mut stack = FuncStack::new();
    stack.push(2.5f64.into()).unwrap();
    exec_instr_handler(Instruction::F64Ceil, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (3.0f64).into());
}

#[test]
fn test_f64_floor() {
    let mut stack = FuncStack::new();
    stack.push(2.5f64.into()).unwrap();
    exec_instr_handler(Instruction::F64Floor, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (2.0f64).into());
}

#[test]
fn test_f64_trunc() {
    let mut stack = FuncStack::new();
    stack.push(2.5f64.into()).unwrap();
    exec_instr_handler(Instruction::F64Trunc, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (2.0f64).into());
}

#[test]
fn test_f64_nearest() {
    let mut stack = FuncStack::new();
    stack.push(2.5f64.into()).unwrap();
    exec_instr_handler(Instruction::F64Nearest, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (3.0f64).into());
}

#[test]
fn test_f64_sqrt() {
    let mut stack = FuncStack::new();
    stack.push(4.0f64.into()).unwrap();
    exec_instr_handler(Instruction::F64Sqrt, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (2.0f64).into());
}

#[test]
fn test_f64_add() {
    let mut stack = FuncStack::new();
    stack.push(1.0f64.into()).unwrap();
    stack.push(2.0f64.into()).unwrap();
    exec_instr_handler(Instruction::F64Add, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 3.0f64.into());
}

#[test]
fn test_f64_sub() {
    let mut stack = FuncStack::new();
    stack.push(2.0f64.into()).unwrap();
    stack.push(1.5f64.into()).unwrap();
    exec_instr_handler(Instruction::F64Sub, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 0.5f64.into());
}

#[test]
fn test_f64_mul() {
    let mut stack = FuncStack::new();
    stack.push(2.5f64.into()).unwrap();
    stack.push(2.0f64.into()).unwrap();
    exec_instr_handler(Instruction::F64Mul, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 5.0f64.into());
}

#[test]
fn test_f64_div() {
    let mut stack = FuncStack::new();
    stack.push(12.0f64.into()).unwrap();
    stack.push(3.0f64.into()).unwrap();
    exec_instr_handler(Instruction::F64Div, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 4.0f64.into());
}

#[test]
fn test_f64_min() {
    let mut stack = FuncStack::new();
    stack.push(2.0f64.into()).unwrap();
    stack.push(3.0f64.into()).unwrap();
    exec_instr_handler(Instruction::F64Min, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 2.0f64.into());
}

#[test]
fn test_f64_max() {
    let mut stack = FuncStack::new();
    stack.push(2.0f64.into()).unwrap();
    stack.push(3.0f64.into()).unwrap();
    exec_instr_handler(Instruction::F64Max, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 3.0f64.into());
}

#[test]
fn test_f64_copysign() {
    let mut stack = FuncStack::new();
    stack.push(2.0f64.into()).unwrap();
    stack.push((-3.0f64).into()).unwrap();
    exec_instr_handler(Instruction::F64Copysign, &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), (-2.0f64).into());
}

#[test]
fn test_local_get() {
    let mut stack = FuncStack::new();
    stack.locals.grow(None, 0.into()).unwrap();
    stack.locals.set(&Index::Num(0), 42.into()).unwrap();
    exec_instr_handler(Instruction::LocalGet(Index::Num(0)), &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 42.into());
}

#[test]
fn test_local_get_error() {
    let mut stack = FuncStack::new();
    assert!(exec_instr_handler(Instruction::LocalGet(Index::Num(0)), &mut stack,).is_err());
}

#[test]
fn test_local_set() {
    let mut stack = FuncStack::new();
    stack.push(15.into()).unwrap();
    stack.locals.grow(None, 0.into()).unwrap();
    stack.locals.grow(None, 0.into()).unwrap();
    exec_instr_handler(Instruction::LocalSet(Index::Num(1)), &mut stack).unwrap();
    assert_eq!(stack.locals.get(&Index::Num(1)).unwrap().clone(), 15.into());
    assert!(stack.pop().is_err());
}

#[test]
fn test_local_set_locals_error() {
    let mut stack = FuncStack::new();
    stack.push(15.into()).unwrap();
    assert!(exec_instr_handler(Instruction::LocalSet(Index::Num(0)), &mut stack,).is_err());
}

#[test]
fn test_local_set_stack_error() {
    let mut stack = FuncStack::new();
    assert!(exec_instr_handler(Instruction::LocalSet(Index::Num(0)), &mut stack,).is_err());
}

#[test]
fn test_local_get_by_id() {
    let mut stack = FuncStack::new();
    stack
        .locals
        .grow(Some(String::from("num")), 0.into())
        .unwrap();
    stack.locals.set(&Index::Num(0), 42.into()).unwrap();

    let id = test_index("num");

    exec_instr_handler(Instruction::LocalGet(id), &mut stack).unwrap();
    assert_eq!(stack.pop().unwrap(), 42.into());
}

#[test]
fn test_local_get_by_id_error() {
    let mut stack = FuncStack::new();
    stack
        .locals
        .grow(Some(String::from("num")), 0.into())
        .unwrap();
    stack.locals.set(&Index::Num(0), 42.into()).unwrap();

    let id = test_index("num_other");
    assert!(exec_instr_handler(Instruction::LocalGet(id), &mut stack).is_err());
}

#[test]
fn test_local_set_by_id() {
    let mut stack = FuncStack::new();
    stack.push(15.into()).unwrap();
    stack
        .locals
        .grow(Some(String::from("num")), 0.into())
        .unwrap();
    stack
        .locals
        .grow(Some(String::from("num_other")), 0.into())
        .unwrap();

    let id = test_index("num_other");

    exec_instr_handler(Instruction::LocalSet(id), &mut stack).unwrap();
    assert_eq!(stack.locals.get(&Index::Num(1)).unwrap().clone(), 15.into());
    assert!(stack.pop().is_err());
}

#[test]
fn test_local_set_by_id_error() {
    let mut stack = FuncStack::new();
    stack.push(15.into()).unwrap();
    stack
        .locals
        .grow(Some(String::from("num")), 0.into())
        .unwrap();

    let id = test_index("num_other");

    assert!(exec_instr_handler(Instruction::LocalSet(id), &mut stack).is_err());
}

#[test]
fn test_local_tee() {
    let mut stack = FuncStack::new();
    stack.push(15.into()).unwrap();
    stack.locals.grow(None, 0.into()).unwrap();
    stack.locals.grow(None, 0.into()).unwrap();
    exec_instr_handler(Instruction::LocalTee(Index::Num(1)), &mut stack).unwrap();
    assert_eq!(stack.locals.get(&Index::Num(1)).unwrap().clone(), 15.into());
    assert_eq!(stack.pop().unwrap(), 15.into());
}

#[test]
fn test_local_tee_error() {
    let mut stack = FuncStack::new();
    assert!(exec_instr_handler(Instruction::LocalTee(Index::Num(0)), &mut stack,).is_err());
}

#[test]
fn test_local_tee_by_id() {
    let mut stack = FuncStack::new();
    stack.push(15.into()).unwrap();
    stack
        .locals
        .grow(Some(String::from("num")), 0.into())
        .unwrap();
    stack
        .locals
        .grow(Some(String::from("num_other")), 0.into())
        .unwrap();

    let id = test_index("num_other");

    exec_instr_handler(Instruction::LocalTee(id), &mut stack).unwrap();
    assert_eq!(stack.locals.get(&Index::Num(1)).unwrap().clone(), 15.into());
    assert_eq!((stack.pop().unwrap()), 15.into());
}

#[test]
fn test_local_tee_by_id_error() {
    let mut stack = FuncStack::new();
    stack.push(15.into()).unwrap();
    stack
        .locals
        .grow(Some(String::from("num")), 0.into())
        .unwrap();

    let id = test_index("num_other");
    assert!(exec_instr_handler(Instruction::LocalTee(id), &mut stack).is_err());
}

#[test]
fn test_return_instr() {
    let response = exec_instr_handler(Instruction::Return, &mut FuncStack::new()).unwrap();
    assert_eq!(response.control, Control::Return);
}

#[test]
fn test_nop() {
    let response = exec_instr_handler(Instruction::Nop, &mut FuncStack::new()).unwrap();
    assert_eq!(response.control, Control::None);
}

#[test]
fn test_call_func() {
    let response = exec_instr_handler(
        Instruction::Call(Index::Id(String::from("fn"))),
        &mut FuncStack::new(),
    )
    .unwrap();

    match response.control {
        Control::ExecFunc(id) => assert_eq!(id, test_index("fn")),
        _ => panic!("Expected Exec::Call"),
    }
}

#[test]
fn test_if_instr() {
    let mut stack = FuncStack::new();
    stack.push(1.into()).unwrap();
    let block_type = test_block_type!((test_local!(ValType::I64)), (ValType::I32));
    let response = exec_instr_handler(
        test_if!(
            block_type,
            (Instruction::I32Const(2)),
            (Instruction::I32Const(3))
        ),
        &mut stack,
    )
    .unwrap();

    if let Control::ExecBlock(block_type, block) = response.control {
        assert_eq!(block_type.ty.params.len(), 1);
        assert_eq!(block_type.ty.params[0].val_type, ValType::I64);
        assert_eq!(block_type.ty.results.len(), 1);
        assert_eq!(block_type.ty.results[0], ValType::I32);

        assert_eq!(block.instrs.len(), 1);
        assert_eq!(block.instrs[0], Instruction::I32Const(2));
    } else {
        panic!("Expected Exec::Block");
    }
}

#[test]
fn test_if_else() {
    let mut stack = FuncStack::new();
    stack.push(0.into()).unwrap();
    let block_type = test_block_type!((test_local!(ValType::I64)), (ValType::I32));
    let response = exec_instr_handler(
        test_if!(
            block_type,
            (Instruction::I32Const(2)),
            (Instruction::I32Const(3))
        ),
        &mut stack,
    )
    .unwrap();

    if let Control::ExecBlock(_, block) = response.control {
        assert_eq!(block.instrs[0], Instruction::I32Const(3));
    } else {
        panic!("Expected Exec::Block");
    }
}

#[test]
fn test_if_error() {
    assert!(exec_instr_handler(test_if!(test_block_type!()), &mut FuncStack::new()).is_err());
}

#[test]
#[should_panic]
fn test_else() {
    exec_instr_handler(Instruction::Else, &mut FuncStack::new()).unwrap();
}

#[test]
#[should_panic]
fn test_end() {
    exec_instr_handler(Instruction::End, &mut FuncStack::new()).unwrap();
}

#[test]
fn test_block() {
    let block_type = test_block_type!((test_local!(ValType::I64)), (ValType::I32));
    let response = exec_instr_handler(
        test_block!(block_type, (Instruction::I32Const(2))),
        &mut FuncStack::new(),
    )
    .unwrap();

    if let Control::ExecBlock(block_type, block) = response.control {
        assert_eq!(block_type.ty.params.len(), 1);
        assert_eq!(block_type.ty.params[0].val_type, ValType::I64);
        assert_eq!(block_type.ty.results.len(), 1);
        assert_eq!(block_type.ty.results[0], ValType::I32);

        assert_eq!(block.instrs.len(), 1);
        assert_eq!(block.instrs[0], Instruction::I32Const(2));
    } else {
        panic!("Expected Exec::Block");
    }
}

#[test]
fn test_branch() {
    let response =
        exec_instr_handler(Instruction::Br(Index::Num(0)), &mut FuncStack::new()).unwrap();
    assert_eq!(response.control, Control::Branch(Index::Num(0)));
}

#[test]
fn test_loop() {
    let block_type = test_block_type!((test_local!(ValType::I64)), (ValType::I32));
    let response = exec_instr_handler(
        test_loop!(block_type, (Instruction::I32Const(2))),
        &mut FuncStack::new(),
    )
    .unwrap();

    if let Control::ExecLoop(block_type, block) = response.control {
        assert_eq!(block_type.ty.params[0].val_type, ValType::I64);
        assert_eq!(block_type.ty.results[0], ValType::I32);

        assert_eq!(block.instrs[0], Instruction::I32Const(2));
    } else {
        panic!("Expected Exec::Loop");
    }
}

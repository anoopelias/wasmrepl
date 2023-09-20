use crate::response::{Control, Response};
use crate::{executor::State, value::Value};
use anyhow::Result;

use crate::model::{Index, Instruction};
use crate::test_utils::test_index;

use super::Handler;

fn exec_instr_handler(instr: &Instruction, state: &mut State) -> Result<Response> {
    let mut handler = Handler::new(state);
    handler.handle(instr)
}

#[test]
fn test_i32_const() {
    let mut state = State::new();
    exec_instr_handler(&Instruction::I32Const(42), &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 42.into());
}

#[test]
fn test_i64_const() {
    let mut state = State::new();
    exec_instr_handler(&Instruction::I64Const(52), &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 52i64.into());
}

#[test]
fn test_drop() {
    let mut state = State::new();
    state.stack.push(42.into());
    exec_instr_handler(&Instruction::Drop, &mut state).unwrap();
    assert!(state.stack.pop().is_err());
}

#[test]
fn test_drop_error() {
    let mut state = State::new();
    assert!(exec_instr_handler(&Instruction::Drop, &mut state).is_err());
}

#[test]
fn test_i32_clz() {
    let mut state = State::new();
    state.stack.push(1023.into());
    exec_instr_handler(&Instruction::I32Clz, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 22.into());
}

#[test]
fn test_i32_clz_error() {
    let mut state = State::new();
    assert!(exec_instr_handler(&Instruction::I32Clz, &mut state).is_err());
}

#[test]
fn test_i32_clz_type_error() {
    let mut state = State::new();
    state.stack.push(1023i64.into());
    assert!(exec_instr_handler(&Instruction::I32Clz, &mut state).is_err());
}

#[test]
fn test_i32_ctz() {
    let mut state = State::new();
    state.stack.push(1024.into());
    exec_instr_handler(&Instruction::I32Ctz, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 10.into());
}

#[test]
fn test_i32_ctz_error() {
    let mut state = State::new();
    assert!(exec_instr_handler(&Instruction::I32Ctz, &mut state).is_err());
}

#[test]
fn test_i32_popcnt() {
    let mut state = State::new();
    state.stack.push(1023.into());
    exec_instr_handler(&Instruction::I32Popcnt, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 10.into());
}

#[test]
fn test_i32_add() {
    let mut state = State::new();
    state.stack.push(1.into());
    state.stack.push(2.into());
    exec_instr_handler(&Instruction::I32Add, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 3.into());
}

#[test]
fn test_i32_add_error() {
    let mut state = State::new();
    state.stack.push(1.into());
    assert!(exec_instr_handler(&Instruction::I32Add, &mut state).is_err());
}

#[test]
fn test_i32_add_type_error() {
    let mut state = State::new();
    state.stack.push(1i64.into());
    state.stack.push(2.into());
    assert!(exec_instr_handler(&Instruction::I32Add, &mut state).is_err());
}

#[test]
fn test_i32_sub() {
    let mut state = State::new();
    state.stack.push(2.into());
    state.stack.push(1.into());
    exec_instr_handler(&Instruction::I32Sub, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), Value::from(1));
}

#[test]
fn test_i32_sub_overflow() {
    let mut state = State::new();
    state.stack.push(i32::MAX.into());
    state.stack.push(Value::from(-1));
    exec_instr_handler(&Instruction::I32Sub, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), Value::from(-2147483648));
}

#[test]
fn test_i32_sub_error() {
    let mut state = State::new();
    state.stack.push(1.into());
    assert!(exec_instr_handler(&Instruction::I32Sub, &mut state).is_err());
}

#[test]
fn test_i32_mul() {
    let mut state = State::new();
    state.stack.push(2.into());
    state.stack.push(3.into());
    exec_instr_handler(&Instruction::I32Mul, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 6.into());
}

#[test]
fn test_i32_mul_overflow() {
    let mut state = State::new();
    state.stack.push(i32::MAX.into());
    state.stack.push(3.into());
    exec_instr_handler(&Instruction::I32Mul, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 2147483645.into());
}

#[test]
fn test_i32_mul_error() {
    let mut state = State::new();
    state.stack.push(1.into());
    assert!(exec_instr_handler(&Instruction::I32Mul, &mut state).is_err());
}

#[test]
fn test_i32_div_s() {
    let mut state = State::new();
    state.stack.push(i32::MIN.into());
    state.stack.push(2.into());
    exec_instr_handler(&Instruction::I32DivU, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0x40000000.into());
}

#[test]
fn test_i32_div_s_error() {
    let mut state = State::new();
    state.stack.push(1.into());
    assert!(exec_instr_handler(&Instruction::I32DivS, &mut state).is_err());
}

#[test]
fn test_i32_div_s_type_error() {
    let mut state = State::new();
    state.stack.push(1i64.into());
    state.stack.push(2.into());
    assert!(exec_instr_handler(&Instruction::I32DivS, &mut state).is_err());
}

#[test]
fn test_i32_div_u() {
    let mut state = State::new();
    state.stack.push(7.into());
    state.stack.push(3.into());
    exec_instr_handler(&Instruction::I32DivU, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 2.into());
}

#[test]
fn test_i32_rem_s() {
    let mut state = State::new();
    state.stack.push(7.into());
    state.stack.push(3.into());
    exec_instr_handler(&Instruction::I32RemS, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 1.into());
}

#[test]
fn test_i32_rem_u() {
    let mut state = State::new();
    state.stack.push(i32::MIN.into());
    state.stack.push((-1).into());
    exec_instr_handler(&Instruction::I32RemU, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), i32::MIN.into());
}

#[test]
fn test_i32_and() {
    let mut state = State::new();
    state.stack.push(0b1010.into());
    state.stack.push(0b1100.into());
    exec_instr_handler(&Instruction::I32And, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0b1000.into());
}

#[test]
fn test_i32_or() {
    let mut state = State::new();
    state.stack.push(0b1010.into());
    state.stack.push(0b1100.into());
    exec_instr_handler(&Instruction::I32Or, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0b1110.into());
}

#[test]
fn test_i32_xor() {
    let mut state = State::new();
    state.stack.push(0b1010.into());
    state.stack.push(0b1100.into());
    exec_instr_handler(&Instruction::I32Xor, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0b0110.into());
}

#[test]
fn test_i32_shl() {
    let mut state = State::new();
    state.stack.push(0b1010.into());
    state.stack.push(2.into());
    exec_instr_handler(&Instruction::I32Shl, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0b101000.into());
}

#[test]
fn test_i32_shr_s() {
    let mut state = State::new();
    state.stack.push(i32::MIN.into());
    state.stack.push(1.into());
    exec_instr_handler(&Instruction::I32ShrS, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (-1073741824).into());
}

#[test]
fn test_i32_shr_u() {
    let mut state = State::new();
    state.stack.push(i32::MIN.into());
    state.stack.push(1.into());
    exec_instr_handler(&Instruction::I32ShrU, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0x40000000.into());
}

#[test]
fn test_i32_rotl() {
    let mut state = State::new();
    state.stack.push(0x10008002.into());
    state.stack.push(2.into());
    exec_instr_handler(&Instruction::I32Rotl, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0x40020008.into());
}

#[test]
fn test_i32_rotr() {
    let mut state = State::new();
    state.stack.push(0x40020008.into());
    state.stack.push(2.into());
    exec_instr_handler(&Instruction::I32Rotr, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0x10008002.into());
}

#[test]
fn test_i64_clz() {
    let mut state = State::new();
    state.stack.push(1023i64.into());
    exec_instr_handler(&Instruction::I64Clz, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 54i64.into());
}

#[test]
fn test_i64_ctz() {
    let mut state = State::new();
    state.stack.push(1024i64.into());
    exec_instr_handler(&Instruction::I64Ctz, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 10i64.into());
}

#[test]
fn test_i64_popcnt() {
    let mut state = State::new();
    state.stack.push(1023i64.into());
    exec_instr_handler(&Instruction::I64Popcnt, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 10i64.into());
}

#[test]
fn test_i64_add() {
    let mut state = State::new();
    state.stack.push(1i64.into());
    state.stack.push(2i64.into());
    exec_instr_handler(&Instruction::I64Add, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 3i64.into());
}

#[test]
fn test_i64_sub() {
    let mut state = State::new();
    state.stack.push(2i64.into());
    state.stack.push(1i64.into());
    exec_instr_handler(&Instruction::I64Sub, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 1i64.into());
}

#[test]
fn test_i64_mul() {
    let mut state = State::new();
    state.stack.push(2i64.into());
    state.stack.push(3i64.into());
    exec_instr_handler(&Instruction::I64Mul, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 6i64.into());
}

#[test]
fn test_i64_div_s() {
    let mut state = State::new();
    state.stack.push(7i64.into());
    state.stack.push(3i64.into());
    exec_instr_handler(&Instruction::I64DivS, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 2i64.into());
}

#[test]
fn test_i64_div_u() {
    let mut state = State::new();
    state.stack.push(i64::MIN.into());
    state.stack.push(2i64.into());
    exec_instr_handler(&Instruction::I64DivU, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0x4000000000000000i64.into());
}

#[test]
fn test_i64_rem_s() {
    let mut state = State::new();
    state.stack.push(7i64.into());
    state.stack.push(3i64.into());
    exec_instr_handler(&Instruction::I64RemS, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 1i64.into());
}

#[test]
fn test_i64_rem_u() {
    let mut state = State::new();
    state.stack.push(i64::MIN.into());
    state.stack.push((-1i64).into());
    exec_instr_handler(&Instruction::I64RemU, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), i64::MIN.into());
}

#[test]
fn test_i64_and() {
    let mut state = State::new();
    state.stack.push(0b1010i64.into());
    state.stack.push(0b1100i64.into());
    exec_instr_handler(&Instruction::I64And, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0b1000i64.into());
}

#[test]
fn test_i64_or() {
    let mut state = State::new();
    state.stack.push(0b1010i64.into());
    state.stack.push(0b1100i64.into());
    exec_instr_handler(&Instruction::I64Or, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0b1110i64.into());
}

#[test]
fn test_i64_xor() {
    let mut state = State::new();
    state.stack.push(0b1010i64.into());
    state.stack.push(0b1100i64.into());
    exec_instr_handler(&Instruction::I64Xor, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0b0110i64.into());
}

#[test]
fn test_i64_shl() {
    let mut state = State::new();
    state.stack.push(0b1010i64.into());
    state.stack.push(2i64.into());
    exec_instr_handler(&Instruction::I64Shl, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0b101000i64.into());
}

#[test]
fn test_i64_shr_u() {
    let mut state = State::new();
    state.stack.push(i64::MIN.into());
    state.stack.push(1i64.into());
    exec_instr_handler(&Instruction::I64ShrU, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0x4000000000000000i64.into());
}

#[test]
fn test_i64_shr_s() {
    let mut state = State::new();
    state.stack.push(i64::MIN.into());
    state.stack.push(1i64.into());
    exec_instr_handler(&Instruction::I64ShrS, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (-4611686018427387904i64).into());
}

#[test]
fn test_i64_rotl() {
    let mut state = State::new();
    state.stack.push(0x10008002i64.into());
    state.stack.push(2i64.into());
    exec_instr_handler(&Instruction::I64Rotl, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0x40020008i64.into());
}

#[test]
fn test_i64_rotr() {
    let mut state = State::new();
    state.stack.push(0x40020008i64.into());
    state.stack.push(2i64.into());
    exec_instr_handler(&Instruction::I64Rotr, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0x10008002i64.into());
}

#[test]
fn test_f32_const() {
    let mut state = State::new();
    exec_instr_handler(&Instruction::F32Const(3.14), &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 3.14f32.into());
}

#[test]
fn test_f32_abs() {
    let mut state = State::new();
    state.stack.push((-2.5f32).into());
    exec_instr_handler(&Instruction::F32Abs, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 2.5f32.into());
}

#[test]
fn test_f32_neg() {
    let mut state = State::new();
    state.stack.push(2.5f32.into());
    exec_instr_handler(&Instruction::F32Neg, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (-2.5f32).into());
}

#[test]
fn test_f32_ceil() {
    let mut state = State::new();
    state.stack.push((-2.5f32).into());
    exec_instr_handler(&Instruction::F32Ceil, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (-2.0f32).into());
}

#[test]
fn test_f32_floor() {
    let mut state = State::new();
    state.stack.push(2.5f32.into());
    exec_instr_handler(&Instruction::F32Floor, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (2.0f32).into());
}

#[test]
fn test_f32_trunc() {
    let mut state = State::new();
    state.stack.push(2.5f32.into());
    exec_instr_handler(&Instruction::F32Trunc, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (2.0f32).into());
}

#[test]
fn test_f32_nearest() {
    let mut state = State::new();
    state.stack.push(2.5f32.into());
    exec_instr_handler(&Instruction::F32Nearest, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (3.0f32).into());
}

#[test]
fn test_f32_sqrt() {
    let mut state = State::new();
    state.stack.push(4.0f32.into());
    exec_instr_handler(&Instruction::F32Sqrt, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (2.0f32).into());
}

#[test]
fn test_f32_add() {
    let mut state = State::new();
    state.stack.push(1.0f32.into());
    state.stack.push(2.0f32.into());
    exec_instr_handler(&Instruction::F32Add, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 3.0f32.into());
}

#[test]
fn test_f32_sub() {
    let mut state = State::new();
    state.stack.push(2.0f32.into());
    state.stack.push(1.5f32.into());
    exec_instr_handler(&Instruction::F32Sub, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0.5f32.into());
}

#[test]
fn test_f32_mul() {
    let mut state = State::new();
    state.stack.push(2.5f32.into());
    state.stack.push(2.0f32.into());
    exec_instr_handler(&Instruction::F32Mul, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 5.0f32.into());
}

#[test]
fn test_f32_div() {
    let mut state = State::new();
    state.stack.push(7.0f32.into());
    state.stack.push(2.0f32.into());
    exec_instr_handler(&Instruction::F32Div, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 3.5f32.into());
}

#[test]
fn test_f32_min() {
    let mut state = State::new();
    state.stack.push(2.0f32.into());
    state.stack.push(3.0f32.into());
    exec_instr_handler(&Instruction::F32Min, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 2.0f32.into());
}

#[test]
fn test_f32_max() {
    let mut state = State::new();
    state.stack.push(2.0f32.into());
    state.stack.push(3.0f32.into());
    exec_instr_handler(&Instruction::F32Max, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 3.0f32.into());
}

#[test]
fn test_f32_copysign() {
    let mut state = State::new();
    state.stack.push(2.0f32.into());
    state.stack.push((-3.0f32).into());
    exec_instr_handler(&Instruction::F32Copysign, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (-2.0f32).into());
}

#[test]
fn test_f64_const() {
    let mut state = State::new();
    exec_instr_handler(&Instruction::F64Const(3.14), &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 3.14f64.into());
}

#[test]
fn test_f64_abs() {
    let mut state = State::new();
    state.stack.push((-2.5f64).into());
    exec_instr_handler(&Instruction::F64Abs, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 2.5f64.into());
}

#[test]
fn test_f64_neg() {
    let mut state = State::new();
    state.stack.push(2.5f64.into());
    exec_instr_handler(&Instruction::F64Neg, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (-2.5f64).into());
}

#[test]
fn test_f64_ceil() {
    let mut state = State::new();
    state.stack.push(2.5f64.into());
    exec_instr_handler(&Instruction::F64Ceil, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (3.0f64).into());
}

#[test]
fn test_f64_floor() {
    let mut state = State::new();
    state.stack.push(2.5f64.into());
    exec_instr_handler(&Instruction::F64Floor, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (2.0f64).into());
}

#[test]
fn test_f64_trunc() {
    let mut state = State::new();
    state.stack.push(2.5f64.into());
    exec_instr_handler(&Instruction::F64Trunc, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (2.0f64).into());
}

#[test]
fn test_f64_nearest() {
    let mut state = State::new();
    state.stack.push(2.5f64.into());
    exec_instr_handler(&Instruction::F64Nearest, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (3.0f64).into());
}

#[test]
fn test_f64_sqrt() {
    let mut state = State::new();
    state.stack.push(4.0f64.into());
    exec_instr_handler(&Instruction::F64Sqrt, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (2.0f64).into());
}

#[test]
fn test_f64_add() {
    let mut state = State::new();
    state.stack.push(1.0f64.into());
    state.stack.push(2.0f64.into());
    exec_instr_handler(&Instruction::F64Add, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 3.0f64.into());
}

#[test]
fn test_f64_sub() {
    let mut state = State::new();
    state.stack.push(2.0f64.into());
    state.stack.push(1.5f64.into());
    exec_instr_handler(&Instruction::F64Sub, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 0.5f64.into());
}

#[test]
fn test_f64_mul() {
    let mut state = State::new();
    state.stack.push(2.5f64.into());
    state.stack.push(2.0f64.into());
    exec_instr_handler(&Instruction::F64Mul, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 5.0f64.into());
}

#[test]
fn test_f64_div() {
    let mut state = State::new();
    state.stack.push(12.0f64.into());
    state.stack.push(3.0f64.into());
    exec_instr_handler(&Instruction::F64Div, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 4.0f64.into());
}

#[test]
fn test_f64_min() {
    let mut state = State::new();
    state.stack.push(2.0f64.into());
    state.stack.push(3.0f64.into());
    exec_instr_handler(&Instruction::F64Min, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 2.0f64.into());
}

#[test]
fn test_f64_max() {
    let mut state = State::new();
    state.stack.push(2.0f64.into());
    state.stack.push(3.0f64.into());
    exec_instr_handler(&Instruction::F64Max, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 3.0f64.into());
}

#[test]
fn test_f64_copysign() {
    let mut state = State::new();
    state.stack.push(2.0f64.into());
    state.stack.push((-3.0f64).into());
    exec_instr_handler(&Instruction::F64Copysign, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), (-2.0f64).into());
}

#[test]
fn test_local_get() {
    let mut state = State::new();
    state.locals.grow(None, 0.into()).unwrap();
    state.locals.set(&Index::Num(0), 42.into()).unwrap();
    exec_instr_handler(&Instruction::LocalGet(Index::Num(0)), &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 42.into());
}

#[test]
fn test_local_get_error() {
    let mut state = State::new();
    assert!(exec_instr_handler(&Instruction::LocalGet(Index::Num(0)), &mut state,).is_err());
}

#[test]
fn test_local_set() {
    let mut state = State::new();
    state.stack.push(15.into());
    state.locals.grow(None, 0.into()).unwrap();
    state.locals.grow(None, 0.into()).unwrap();
    exec_instr_handler(&Instruction::LocalSet(Index::Num(1)), &mut state).unwrap();
    assert_eq!(state.locals.get(&Index::Num(1)).unwrap().clone(), 15.into());
    assert!(state.stack.pop().is_err());
}

#[test]
fn test_local_set_locals_error() {
    let mut state = State::new();
    state.stack.push(15.into());
    assert!(exec_instr_handler(&Instruction::LocalSet(Index::Num(0)), &mut state,).is_err());
}

#[test]
fn test_local_set_stack_error() {
    let mut state = State::new();
    assert!(exec_instr_handler(&Instruction::LocalSet(Index::Num(0)), &mut state,).is_err());
}

#[test]
fn test_local_get_by_id() {
    let mut state = State::new();
    state
        .locals
        .grow(Some(String::from("num")), 0.into())
        .unwrap();
    state.locals.set(&Index::Num(0), 42.into()).unwrap();

    let id = test_index("num");

    exec_instr_handler(&Instruction::LocalGet(id), &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 42.into());
}

#[test]
fn test_local_get_by_id_error() {
    let mut state = State::new();
    state
        .locals
        .grow(Some(String::from("num")), 0.into())
        .unwrap();
    state.locals.set(&Index::Num(0), 42.into()).unwrap();

    let id = test_index("num_other");
    assert!(exec_instr_handler(&Instruction::LocalGet(id), &mut state).is_err());
}

#[test]
fn test_local_set_by_id() {
    let mut state = State::new();
    state.stack.push(15.into());
    state
        .locals
        .grow(Some(String::from("num")), 0.into())
        .unwrap();
    state
        .locals
        .grow(Some(String::from("num_other")), 0.into())
        .unwrap();

    let id = test_index("num_other");

    exec_instr_handler(&Instruction::LocalSet(id), &mut state).unwrap();
    assert_eq!(state.locals.get(&Index::Num(1)).unwrap().clone(), 15.into());
    assert!(state.stack.pop().is_err());
}

#[test]
fn test_local_set_by_id_error() {
    let mut state = State::new();
    state.stack.push(15.into());
    state
        .locals
        .grow(Some(String::from("num")), 0.into())
        .unwrap();

    let id = test_index("num_other");

    assert!(exec_instr_handler(&Instruction::LocalSet(id), &mut state).is_err());
}

#[test]
fn test_local_tee() {
    let mut state = State::new();
    state.stack.push(15.into());
    state.locals.grow(None, 0.into()).unwrap();
    state.locals.grow(None, 0.into()).unwrap();
    exec_instr_handler(&Instruction::LocalTee(Index::Num(1)), &mut state).unwrap();
    assert_eq!(state.locals.get(&Index::Num(1)).unwrap().clone(), 15.into());
    assert_eq!(state.stack.pop().unwrap(), 15.into());
}

#[test]
fn test_local_tee_error() {
    let mut state = State::new();
    assert!(exec_instr_handler(&Instruction::LocalTee(Index::Num(0)), &mut state,).is_err());
}

#[test]
fn test_local_tee_by_id() {
    let mut state = State::new();
    state.stack.push(15.into());
    state
        .locals
        .grow(Some(String::from("num")), 0.into())
        .unwrap();
    state
        .locals
        .grow(Some(String::from("num_other")), 0.into())
        .unwrap();

    let id = test_index("num_other");

    exec_instr_handler(&Instruction::LocalTee(id), &mut state).unwrap();
    assert_eq!(state.locals.get(&Index::Num(1)).unwrap().clone(), 15.into());
    assert_eq!((state.stack.pop().unwrap()), 15.into());
}

#[test]
fn test_local_tee_by_id_error() {
    let mut state = State::new();
    state.stack.push(15.into());
    state
        .locals
        .grow(Some(String::from("num")), 0.into())
        .unwrap();

    let id = test_index("num_other");
    assert!(exec_instr_handler(&Instruction::LocalTee(id), &mut state).is_err());
}

#[test]
fn test_return_instr() {
    let response = exec_instr_handler(&Instruction::Return, &mut State::new()).unwrap();
    assert!(response.is_return)
}

#[test]
fn test_nop() {
    let response = exec_instr_handler(&Instruction::Nop, &mut State::new()).unwrap();
    assert!(!response.is_return)
}

#[test]
fn test_call_func() {
    let response = exec_instr_handler(
        &Instruction::Call(Index::Id(String::from("fn"))),
        &mut State::new(),
    )
    .unwrap();

    match response.contd {
        Control::ExecFunc(id) => assert_eq!(id, test_index("fn")),
        _ => panic!("Expected Exec::Call"),
    }
}

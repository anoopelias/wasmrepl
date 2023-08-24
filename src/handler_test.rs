use crate::{
    executor::State,
    value::{test_utils::test_val_i32, Value},
};
use anyhow::Result;

use wast::{
    core::Instruction,
    parser::{self as wastparser, ParseBuffer},
    token::{Float32, Float64, Id, Index, Span},
};

use super::Handler;

fn test_new_index_id<'a>(buf: &'a ParseBuffer) -> Index<'a> {
    let id = wastparser::parse::<Id>(buf).unwrap();
    Index::Id(id)
}

fn exec_instr_handler(instr: &Instruction, state: &mut State) -> Result<()> {
    let mut handler = Handler::new(state);
    handler.handle(instr)
}

fn float32_for(buf: &ParseBuffer) -> Float32 {
    wastparser::parse::<Float32>(&buf).unwrap()
}

fn float64_for(buf: &ParseBuffer) -> Float64 {
    wastparser::parse::<Float64>(&buf).unwrap()
}

#[test]
fn test_unknown_instr() {
    let mut state = State::new();
    let mut handler = Handler::new(&mut state);
    assert!(handler.handle(&Instruction::Nop).is_err());
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
    state.stack.push(7.into());
    state.stack.push(3.into());
    exec_instr_handler(&Instruction::I32DivS, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 2.into());
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
fn test_f32_const() {
    let mut state = State::new();
    let wat = "3.14";
    let buf = ParseBuffer::new(wat).unwrap();
    exec_instr_handler(&Instruction::F32Const(float32_for(&buf)), &mut state).unwrap();
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
fn test_f64_const() {
    let mut state = State::new();
    let wat = "3.14";
    let buf = ParseBuffer::new(wat).unwrap();
    exec_instr_handler(&Instruction::F64Const(float64_for(&buf)), &mut state).unwrap();
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
fn test_local_get() {
    let mut state = State::new();
    state.locals.grow(test_val_i32(0));
    state.locals.set(0, 42.into()).unwrap();
    exec_instr_handler(
        &Instruction::LocalGet(Index::Num(0, Span::from_offset(0))),
        &mut state,
    )
    .unwrap();
    assert_eq!(state.stack.pop().unwrap(), 42.into());
}

#[test]
fn test_local_get_error() {
    let mut state = State::new();
    assert!(exec_instr_handler(
        &Instruction::LocalGet(Index::Num(0, Span::from_offset(0))),
        &mut state,
    )
    .is_err());
}

#[test]
fn test_local_set() {
    let mut state = State::new();
    state.stack.push(15.into());
    state.locals.grow(test_val_i32(0));
    state.locals.grow(test_val_i32(0));
    exec_instr_handler(
        &Instruction::LocalSet(Index::Num(1, Span::from_offset(0))),
        &mut state,
    )
    .unwrap();
    assert_eq!(state.locals.get(1).unwrap().clone(), 15.into());
}

#[test]
fn test_local_set_locals_error() {
    let mut state = State::new();
    state.stack.push(15.into());
    assert!(exec_instr_handler(
        &Instruction::LocalSet(Index::Num(0, Span::from_offset(0))),
        &mut state,
    )
    .is_err());
}

#[test]
fn test_local_set_stack_error() {
    let mut state = State::new();
    assert!(exec_instr_handler(
        &Instruction::LocalSet(Index::Num(0, Span::from_offset(0))),
        &mut state,
    )
    .is_err());
}

#[test]
fn test_local_get_by_id() {
    let mut state = State::new();
    state.locals.grow_by_id("num", test_val_i32(0)).unwrap();
    state.locals.set(0, 42.into()).unwrap();

    let str_id = String::from("$num");
    let buf_id = ParseBuffer::new(&str_id).unwrap();
    let id = test_new_index_id(&buf_id);

    exec_instr_handler(&Instruction::LocalGet(id), &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 42.into());
}

#[test]
fn test_local_get_by_id_error() {
    let mut state = State::new();
    state.locals.grow_by_id("num", test_val_i32(0)).unwrap();
    state.locals.set(0, 42.into()).unwrap();

    let str_id = String::from("$num_other");
    let buf_id = ParseBuffer::new(&str_id).unwrap();
    let id = test_new_index_id(&buf_id);

    assert!(exec_instr_handler(&Instruction::LocalGet(id), &mut state).is_err());
}

#[test]
fn test_local_set_by_id() {
    let mut state = State::new();
    state.stack.push(15.into());
    state.locals.grow_by_id("num", test_val_i32(0)).unwrap();
    state
        .locals
        .grow_by_id("num_other", test_val_i32(0))
        .unwrap();

    let str_id = String::from("$num_other");
    let buf_id = ParseBuffer::new(&str_id).unwrap();
    let id = test_new_index_id(&buf_id);

    exec_instr_handler(&Instruction::LocalSet(id), &mut state).unwrap();
    assert_eq!(state.locals.get(1).unwrap().clone(), 15.into());
}

#[test]
fn test_local_set_by_id_error() {
    let mut state = State::new();
    state.stack.push(15.into());
    state.locals.grow_by_id("num", test_val_i32(0)).unwrap();

    let str_id = String::from("$num_other");
    let buf_id = ParseBuffer::new(&str_id).unwrap();
    let id = test_new_index_id(&buf_id);

    assert!(exec_instr_handler(&Instruction::LocalSet(id), &mut state).is_err());
}

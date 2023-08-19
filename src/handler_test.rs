use crate::{executor::State, value::Value};
use anyhow::Result;

use wast::{
    core::Instruction,
    parser::{self as wastparser, ParseBuffer},
    token::{Id, Index, Span},
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
fn test_i64_clz() {
    let mut state = State::new();
    state.stack.push(1023i64.into());
    exec_instr_handler(&Instruction::I64Clz, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 54i64.into());
}

#[test]
fn test_i64_clz_type_error() {
    let mut state = State::new();
    state.stack.push(1023.into());
    assert!(exec_instr_handler(&Instruction::I64Clz, &mut state).is_err());
}

#[test]
fn test_i32_ctz() {
    let mut state = State::new();
    state.stack.push(1024.into());
    exec_instr_handler(&Instruction::I32Ctz, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 10.into());
}

#[test]
fn test_i32_ctz_max() {
    let mut state = State::new();
    state.stack.push(0.into());
    exec_instr_handler(&Instruction::I32Ctz, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 32.into());
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
fn test_i32_add_overflow() {
    let mut state = State::new();
    state.stack.push(i32::MAX.into());
    state.stack.push(1.into());
    exec_instr_handler(&Instruction::I32Add, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), Value::from(-2147483648));
}

#[test]
fn test_i32_add_error() {
    let mut state = State::new();
    state.stack.push(1.into());
    assert!(exec_instr_handler(&Instruction::I32Add, &mut state).is_err());
}

#[test]
fn test_i32_sub() {
    let mut state = State::new();
    state.stack.push(2.into());
    state.stack.push(1.into());
    exec_instr_handler(&Instruction::I32Sub, &mut state).unwrap();
    assert_eq!(state.stack.pop().unwrap(), 1.into());
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
fn test_i32_div_s_div_by_zero() {
    let mut state = State::new();
    state.stack.push(1.into());
    state.stack.push(0.into());
    assert!(exec_instr_handler(&Instruction::I32DivS, &mut state).is_err());
}

#[test]
fn test_local_get() {
    let mut state = State::new();
    state.locals.grow();
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
    state.locals.grow();
    state.locals.grow();
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
    state.locals.grow_by_id("num").unwrap();
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
    state.locals.grow_by_id("num").unwrap();
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
    state.locals.grow_by_id("num").unwrap();
    state.locals.grow_by_id("num_other").unwrap();

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
    state.locals.grow_by_id("num").unwrap();

    let str_id = String::from("$num_other");
    let buf_id = ParseBuffer::new(&str_id).unwrap();
    let id = test_new_index_id(&buf_id);

    assert!(exec_instr_handler(&Instruction::LocalSet(id), &mut state).is_err());
}

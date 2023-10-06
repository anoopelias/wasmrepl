use crate::{
    model::{FuncType, Local, ValType},
    test_utils::{test_func_type, test_local, test_local_id},
    value::Value,
};

use super::CallStack;

#[test]
fn test_func_add_remove() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((test_local!(ValType::I64)), (ValType::I32));

    call_stack.push(Value::I64(1)).unwrap();
    call_stack.add_func(&func_type).unwrap();

    call_stack.push(Value::I32(2)).unwrap();
    call_stack.remove_func(&func_type, true).unwrap();

    let value = call_stack.pop().unwrap();
    assert_eq!(value, Value::I32(2));
}

#[test]
fn test_func_add_not_enough_inputs_error() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((test_local!(ValType::I64)), ());

    assert!(call_stack.add_func(&func_type).is_err());
}

#[test]
fn test_func_add_invalid_input_error() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((test_local!(ValType::I64)), ());

    call_stack.push(Value::I32(1));
    assert!(call_stack.add_func(&func_type).is_err());
}

#[test]
fn test_func_add_local_id_already_exists() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!(
        (
            test_local_id!("num", ValType::I32),
            test_local_id!("num", ValType::I32)
        ),
        ()
    );

    call_stack.push(Value::I32(1));
    assert!(call_stack.add_func(&func_type).is_err());
}

#[test]
fn test_func_remove_not_enough_outputs_error() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((), (ValType::I32));

    call_stack.add_func(&func_type).unwrap();
    assert!(call_stack.remove_func(&func_type, true).is_err());
}

#[test]
fn test_func_remove_incorrect_type() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((), (ValType::I32));

    call_stack.add_func(&func_type).unwrap();

    call_stack.push(Value::I64(2)).unwrap();
    assert!(call_stack.remove_func(&func_type, true).is_err());
}

#[test]
fn test_func_remove_too_many_outputs() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((), (ValType::I32));

    call_stack.add_func(&func_type).unwrap();

    call_stack.push(Value::I64(2)).unwrap();
    call_stack.push(Value::I32(3)).unwrap();
    call_stack.remove_func(&func_type, false).unwrap();

    let value = call_stack.pop().unwrap();
    assert_eq!(value, Value::I32(3));
}

#[test]
fn test_func_remove_too_many_outputs_error() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((), (ValType::I32));

    call_stack.add_func(&func_type).unwrap();

    call_stack.push(Value::I64(2)).unwrap();
    call_stack.push(Value::I32(2)).unwrap();
    assert!(call_stack.remove_func(&func_type, true).is_err());
}

#[test]
fn test_commit_block_rollback() {
    let mut call_stack = CallStack::new();
    call_stack.push(Value::I32(1));
    call_stack.commit();

    call_stack.push(Value::I32(2));
    call_stack.rollback();

    call_stack.push(Value::I32(3));

    assert_eq!(call_stack.pop().unwrap(), Value::I32(3));
    assert_eq!(call_stack.pop().unwrap(), Value::I32(1));
    assert!(call_stack.pop().is_err());
}

use crate::{
    call_stack::{CallStack, FuncStack},
    model::{FuncType, Index, Local, ValType},
    test_utils::{test_func_type, test_local, test_local_id},
    value::Value,
};

#[test]
fn test_func_add_remove() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((test_local!(ValType::I64)), (ValType::I32));

    call_stack
        .get_func_stack()
        .unwrap()
        .push(Value::I64(1))
        .unwrap();
    call_stack.add_func_stack(&func_type).unwrap();

    call_stack
        .get_func_stack()
        .unwrap()
        .push(Value::I32(2))
        .unwrap();
    call_stack.remove_func_stack(&func_type, true).unwrap();

    let value = call_stack.get_func_stack().unwrap().pop().unwrap();
    assert_eq!(value, Value::I32(2));
}

#[test]
fn test_func_add_not_enough_inputs_error() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((test_local!(ValType::I64)), ());

    assert!(call_stack.add_func_stack(&func_type).is_err());
}

#[test]
fn test_func_add_invalid_input_error() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((test_local!(ValType::I64)), ());

    call_stack
        .get_func_stack()
        .unwrap()
        .push(Value::I32(1))
        .unwrap();
    assert!(call_stack.add_func_stack(&func_type).is_err());
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

    call_stack
        .get_func_stack()
        .unwrap()
        .push(Value::I32(1))
        .unwrap();
    assert!(call_stack.add_func_stack(&func_type).is_err());
}

#[test]
fn test_func_remove_not_enough_outputs_error() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((), (ValType::I32));

    call_stack.add_func_stack(&func_type).unwrap();
    assert!(call_stack.remove_func_stack(&func_type, true).is_err());
}

#[test]
fn test_func_remove_incorrect_type() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((), (ValType::I32));

    call_stack.add_func_stack(&func_type).unwrap();

    call_stack
        .get_func_stack()
        .unwrap()
        .push(Value::I64(2))
        .unwrap();
    assert!(call_stack.remove_func_stack(&func_type, true).is_err());
}

#[test]
fn test_func_remove_too_many_outputs() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((), (ValType::I32));

    call_stack.add_func_stack(&func_type).unwrap();

    let func_stack = call_stack.get_func_stack().unwrap();
    func_stack.push(Value::I64(2)).unwrap();
    func_stack.push(Value::I32(3)).unwrap();
    call_stack.remove_func_stack(&func_type, false).unwrap();

    let value = call_stack.get_func_stack().unwrap().pop().unwrap();
    assert_eq!(value, Value::I32(3));
}

#[test]
fn test_func_remove_too_many_outputs_error() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((), (ValType::I32));

    call_stack.add_func_stack(&func_type).unwrap();

    let func_stack = call_stack.get_func_stack().unwrap();
    func_stack.push(Value::I64(2)).unwrap();
    func_stack.push(Value::I32(2)).unwrap();
    assert!(call_stack.remove_func_stack(&func_type, true).is_err());
}

#[test]
fn test_block_commit_rollback() {
    let mut call_stack = CallStack::new();

    call_stack
        .get_func_stack()
        .unwrap()
        .push(Value::I32(1))
        .unwrap();
    call_stack.commit();

    call_stack
        .get_func_stack()
        .unwrap()
        .push(Value::I32(2))
        .unwrap();
    call_stack.rollback();

    call_stack
        .get_func_stack()
        .unwrap()
        .push(Value::I32(3))
        .unwrap();

    let func_stack = call_stack.get_func_stack().unwrap();
    assert_eq!(func_stack.pop().unwrap(), Value::I32(3));
    assert_eq!(func_stack.pop().unwrap(), Value::I32(1));
    assert!(func_stack.pop().is_err());
}

#[test]
fn test_locals_commit_rollback() {
    let mut call_stack = CallStack::new();

    let locals = &mut call_stack.get_func_stack().unwrap().locals;
    locals.grow(Some("id".to_string()), Value::I32(1)).unwrap();
    locals
        .set(&Index::Id("id".to_string()), Value::I32(2))
        .unwrap();
    call_stack.commit();

    let locals = &mut call_stack.get_func_stack().unwrap().locals;
    locals
        .set(&Index::Id("id".to_string()), Value::I32(3))
        .unwrap();
    call_stack.rollback();

    let locals = &mut call_stack.get_func_stack().unwrap().locals;
    assert_eq!(
        locals.get(&Index::Id("id".to_string())).unwrap(),
        &Value::I32(2)
    );
}

#[test]
fn test_block_add_remove() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((test_local!(ValType::I64)), (ValType::I32));

    call_stack
        .get_func_stack()
        .unwrap()
        .push(Value::I64(1))
        .unwrap();
    call_stack.add_block_stack(&func_type).unwrap();

    let func_stack = call_stack.get_func_stack().unwrap();
    assert_eq!(func_stack.pop().unwrap(), Value::I64(1));
    func_stack.push(Value::I32(2)).unwrap();
}

#[test]
fn test_block_add_not_enough_inputs_error() {
    let mut func_stack = FuncStack::new();
    let func_type = test_func_type!((test_local!(ValType::I64)), ());
    assert!(func_stack.add_block_stack(&func_type).is_err());
}

#[test]
fn test_block_add_invalid_inputs_error() {
    let mut func_stack = FuncStack::new();
    let func_type = test_func_type!((test_local!(ValType::I64)), ());

    func_stack.push(Value::I32(1)).unwrap();
    assert!(func_stack.add_block_stack(&func_type).is_err());
}

#[test]
fn test_block_remove_not_enough_outputs_error() {
    let mut func_stack = FuncStack::new();
    let func_type = test_func_type!((), (ValType::I32));

    func_stack.add_block_stack(&func_type).unwrap();
    assert!(func_stack.remove_block_stack(&func_type, true).is_err());
}

#[test]
fn test_block_remove_incorrect_type() {
    let mut func_stack = FuncStack::new();
    let func_type = test_func_type!((), (ValType::I32));

    func_stack.add_block_stack(&func_type).unwrap();

    func_stack.push(Value::I64(2)).unwrap();
    assert!(func_stack.remove_block_stack(&func_type, true).is_err());
}

#[test]
fn test_block_remove_too_many_outputs() {
    let mut func_stack = FuncStack::new();
    let func_type = test_func_type!((), (ValType::I32));

    func_stack.add_block_stack(&func_type).unwrap();

    func_stack.push(Value::I64(2)).unwrap();
    func_stack.push(Value::I32(3)).unwrap();
    func_stack.remove_block_stack(&func_type, false).unwrap();

    assert_eq!(func_stack.pop().unwrap(), Value::I32(3));
}

#[test]
fn test_block_remove_too_many_outputs_error() {
    let mut func_stack = FuncStack::new();
    let func_type = test_func_type!((), (ValType::I32));

    func_stack.add_block_stack(&func_type).unwrap();

    func_stack.push(Value::I64(2)).unwrap();
    func_stack.push(Value::I32(2)).unwrap();
    assert!(func_stack.remove_block_stack(&func_type, true).is_err());
}

#[test]
fn test_to_string() {
    let mut call_stack = CallStack::new();
    let func_stack = call_stack.get_func_stack().unwrap();
    func_stack.push(Value::I64(1)).unwrap();
    func_stack.push(Value::I32(2)).unwrap();
    call_stack.commit();
    assert_eq!(call_stack.to_string(), "[1, 2]");
}

#[test]
fn test_len() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((), ());

    call_stack.add_func_stack(&func_type).unwrap();
    assert_eq!(call_stack.len(), 2);
}

#[test]
fn test_peek() {
    let mut func_stack = FuncStack::new();
    func_stack.push(Value::I64(1)).unwrap();
    func_stack.push(Value::I32(2)).unwrap();
    assert_eq!(func_stack.peek().unwrap(), Value::I32(2));
    func_stack.pop().unwrap();
    assert_eq!(func_stack.peek().unwrap(), Value::I64(1));
}

#[test]
fn test_soft_string() {
    let mut call_stack = CallStack::new();
    let func_stack = call_stack.get_func_stack().unwrap();
    func_stack.push(Value::I64(1)).unwrap();
    func_stack.push(Value::I32(2)).unwrap();
    assert_eq!(func_stack.to_soft_string().unwrap(), "[1, 2]");
}

#[test]
fn test_func_output_stack_order() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((), (ValType::I32, ValType::I64));
    call_stack.add_func_stack(&func_type).unwrap();

    let func_stack = call_stack.get_func_stack().unwrap();
    func_stack.push(Value::I32(1)).unwrap();
    func_stack.push(Value::I64(2)).unwrap();

    call_stack.remove_func_stack(&func_type, false).unwrap();

    let func_stack = call_stack.get_func_stack().unwrap();
    assert_eq!(func_stack.pop().unwrap(), Value::I64(2));
    assert_eq!(func_stack.pop().unwrap(), Value::I32(1));
}

#[test]
fn test_block_output_stack_order() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((), (ValType::I32, ValType::I64));
    call_stack.add_block_stack(&func_type).unwrap();

    let func_stack = call_stack.get_func_stack().unwrap();
    func_stack.push(Value::I32(1)).unwrap();
    func_stack.push(Value::I64(2)).unwrap();

    call_stack.remove_block_stack(&func_type, true).unwrap();

    let func_stack = call_stack.get_func_stack().unwrap();
    assert_eq!(func_stack.pop().unwrap(), Value::I64(2));
    assert_eq!(func_stack.pop().unwrap(), Value::I32(1));
}

#[test]
fn test_block_input_stack_order() {
    let mut call_stack = CallStack::new();
    let func_type = test_func_type!((test_local!(ValType::I64), test_local!(ValType::I32)), ());
    let func_stack = call_stack.get_func_stack().unwrap();
    func_stack.push(Value::I64(1)).unwrap();
    func_stack.push(Value::I32(2)).unwrap();

    call_stack.add_block_stack(&func_type).unwrap();

    let func_stack = call_stack.get_func_stack().unwrap();
    assert_eq!(func_stack.pop().unwrap(), Value::I32(2));
    assert_eq!(func_stack.pop().unwrap(), Value::I64(1));
}

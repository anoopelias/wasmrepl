use crate::model::{
    Expression, Func, FuncType, Index, Instruction, Line, LineExpression, Local, ValType,
};

use crate::executor::Executor;
use crate::test_utils::{
    test_block, test_block_type, test_if, test_index, test_local, test_local_id,
};

macro_rules! test_line {
    (($( $y:expr ),*)($( $x:expr ),*)) => {
        Line::Expression(LineExpression {
            locals:  vec![$( $y ),*],
            expr:  Expression { instrs: (vec![$( $x ),*]) }
        })
    };
}

macro_rules! test_func {
    ($fname:expr, ($( $param:expr ),*)($( $res:expr ),*)($( $instr:expr ),*)) => {
        Line::Func(Func {
            id: Some(String::from($fname)),
            ty: FuncType {
                params: vec![
                    $( $param ),*
                ],
                results: vec![$( $res ),*]

            },
            line_expression: LineExpression {
                locals: vec![],
                expr:  Expression { instrs: vec![$( $instr ),*] }
            },
        })
    };
}

#[test]
fn test_add() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(42),
        Instruction::I32Const(58),
        Instruction::I32Add
    )];
    let response = executor.execute_line(line).unwrap();
    assert_eq!(response.message(), "[100]");
}

#[test]
fn test_error_rollback() {
    let mut executor = Executor::new();
    let line = test_line![()(Instruction::I32Const(55))];
    executor.execute_line(line).unwrap();

    let line = test_line![()(Instruction::I32Const(55), Instruction::F32Neg)];
    assert!(executor.execute_line(line).is_err());
    // Ensure rollback
    assert_eq!(
        executor.call_stack[0].stack.to_soft_string().unwrap(),
        "[55]"
    );
}

#[test]
fn test_local_set_get() {
    let mut executor = Executor::new();
    let line = test_line![(test_local!(ValType::I32))(
        Instruction::I32Const(42),
        Instruction::LocalSet(Index::Num(0)),
        Instruction::LocalGet(Index::Num(0))
    )];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "local ;0;\n[42]"
    );
}

#[test]
fn test_local_set_commit() {
    let mut executor = Executor::new();
    let line = test_line![(test_local!(ValType::I32))(
        Instruction::I32Const(42),
        Instruction::LocalSet(Index::Num(0)),
        Instruction::LocalGet(Index::Num(0))
    )];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "local ;0;\n[42]"
    );

    let line = test_line![()(
        Instruction::Drop,
        Instruction::I32Const(55),
        Instruction::LocalSet(Index::Num(0)),
        Instruction::LocalGet(Index::Num(0))
    )];
    assert_eq!(executor.execute_line(line).unwrap().message(), "[55]");
}

#[test]
fn test_local_set_local_value_rollback() {
    let mut executor = Executor::new();
    let line = test_line![(test_local!(ValType::I32))(
        Instruction::I32Const(42),
        Instruction::LocalSet(Index::Num(0))
    )];
    executor.execute_line(line).unwrap();

    let line = test_line![()(
        Instruction::I32Const(43),
        Instruction::I32Const(55),
        Instruction::LocalSet(Index::Num(0)),
        // Failing instruction
        Instruction::F32Neg
    )];
    assert!(executor.execute_line(line).is_err());

    let line = test_line![(test_local!(ValType::I32))(Instruction::LocalGet(
        Index::Num(0)
    ))];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "local ;1;\n[42]"
    );
}

#[test]
fn test_local_rollback() {
    let mut executor = Executor::new();
    let line = test_line![(test_local_id!("num", ValType::I32))()];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "local ;0; num\n[]"
    );
    let line = test_line![(test_local_id!("num", ValType::I32))()];
    assert!(executor.execute_line(line).is_err());

    let line = test_line![(test_local_id!("num2", ValType::I32))()];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "local ;1; num2\n[]"
    );
}

#[test]
fn test_local_by_id() {
    let mut executor = Executor::new();

    let local = test_local_id!("num", ValType::I32);

    let set_index = test_index("num");
    let get_index = test_index("num");

    let line = test_line![(local)(
        Instruction::I32Const(42),
        Instruction::LocalSet(set_index),
        Instruction::LocalGet(get_index)
    )];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "local ;0; num\n[42]"
    );
}

#[test]
fn test_local_by_id_mix() {
    let mut executor = Executor::new();
    let local = test_local_id!("num", ValType::I32);
    let index = test_index("num");

    let line = test_line![(test_local!(ValType::I32), local)(
        Instruction::I32Const(42),
        Instruction::LocalSet(index),
        Instruction::LocalGet(Index::Num(1))
    )];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "local ;0;\nlocal ;1; num\n[42]"
    );
}

#[test]
fn test_local_set_get_i64() {
    let mut executor = Executor::new();
    let line = test_line![(test_local!(ValType::I64))(
        Instruction::I64Const(42),
        Instruction::LocalSet(Index::Num(0)),
        Instruction::LocalGet(Index::Num(0))
    )];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "local ;0;\n[42]"
    );
}

#[test]
fn test_local_set_get_f32() {
    let mut executor = Executor::new();
    let local = test_local!(ValType::F32);
    let line = test_line![(local)(
        Instruction::F32Const(3.14),
        Instruction::LocalSet(Index::Num(0)),
        Instruction::LocalGet(Index::Num(0))
    )];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "local ;0;\n[3.14]"
    );
}

#[test]
fn test_local_set_get_f64() {
    let mut executor = Executor::new();
    let local = Local {
        id: None,
        val_type: ValType::F64,
    };
    let line = test_line![(local)(
        Instruction::F64Const(3.14f64),
        Instruction::LocalSet(Index::Num(0)),
        Instruction::LocalGet(Index::Num(0))
    )];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "local ;0;\n[3.14]"
    );
}

#[test]
fn test_local_set_get_type_error() {
    let mut executor = Executor::new();
    let line = test_line![(test_local!(ValType::I32))(
        Instruction::I64Const(55),
        Instruction::LocalSet(Index::Num(0))
    )];
    assert!(executor.execute_line(line).is_err());
}

#[test]
fn test_func() {
    let mut executor = Executor::new();
    let func = test_func!(
        "subtract",
        (
            test_local_id!("first", ValType::I32),
            test_local_id!("second", ValType::I32)
        )(ValType::I32, ValType::I32)(
            Instruction::LocalGet(test_index("first")),
            Instruction::LocalGet(test_index("first")),
            Instruction::LocalGet(test_index("second")),
            Instruction::I32Sub
        )
    );
    let response = executor.execute_line(func).unwrap();
    assert_eq!(response.message(), "func ;0; subtract");

    let call_sub = test_line![()(
        Instruction::I32Const(7),
        Instruction::I32Const(2),
        Instruction::Call(test_index("subtract"))
    )];
    assert_eq!(executor.execute_line(call_sub).unwrap().message(), "[7, 5]");
}

#[test]
fn test_func_error_less_number_of_inputs() {
    let mut executor = Executor::new();
    let func = test_func!("fun", (test_local!(ValType::I32))()());
    executor.execute_line(func).unwrap();

    let call_fun = test_line![()(Instruction::Call(test_index("fun")))];
    assert!(executor.execute_line(call_fun).is_err());
}

#[test]
fn test_func_error_less_number_of_outputs() {
    let mut executor = Executor::new();
    let func = test_func!("fun", ()(ValType::I32)());
    executor.execute_line(func).unwrap();

    let call = test_line![()(Instruction::Call(test_index("fun")))];
    // We expect one output but will get none hence an error
    assert!(executor.execute_line(call).is_err());
}

#[test]
fn test_func_error_more_number_of_outputs() {
    let mut executor = Executor::new();
    let func = test_func!("fun", ()()(Instruction::I32Const(5)));
    executor.execute_line(func).unwrap();

    let call = test_line![()(Instruction::Call(test_index("fun")))];
    // We expect no output but will get one hence an error
    assert!(executor.execute_line(call).is_err());
}

#[test]
fn test_func_input_type() {
    let mut executor = Executor::new();
    let func = test_func!(
        "fun",
        (test_local!(ValType::I32), test_local!(ValType::I64))()()
    );
    executor.execute_line(func).unwrap();

    let call_fun = test_line![()(
        Instruction::I32Const(5),
        Instruction::I64Const(10),
        Instruction::Call(test_index("fun"))
    )];
    executor.execute_line(call_fun).unwrap();
}

#[test]
fn test_func_error_input_type() {
    let mut executor = Executor::new();
    let func = test_func!(
        "fun",
        (test_local!(ValType::I32), test_local!(ValType::I64))()()
    );
    executor.execute_line(func).unwrap();

    let call_fun = test_line![()(
        Instruction::I64Const(5),
        Instruction::I32Const(10),
        Instruction::Call(test_index("fun"))
    )];
    assert!(executor.execute_line(call_fun).is_err());
}

#[test]
fn test_func_output_type() {
    let mut executor = Executor::new();
    let func = test_func!(
        "fun",
        ()(ValType::I32, ValType::I64)(Instruction::I32Const(5), Instruction::I64Const(10))
    );
    executor.execute_line(func).unwrap();

    let call_fun = test_line![()(Instruction::Call(test_index("fun")))];
    executor.execute_line(call_fun).unwrap();
}

#[test]
fn test_func_output_type_error() {
    let mut executor = Executor::new();
    let func = test_func!(
        "fun",
        ()(ValType::I32, ValType::I64)(Instruction::I64Const(10), Instruction::I32Const(5))
    );
    executor.execute_line(func).unwrap();

    let call_fun = test_line![()(Instruction::Call(test_index("fun")))];
    assert!(executor.execute_line(call_fun).is_err());
}

#[test]
fn test_func_no_id() {
    let mut executor = Executor::new();
    let func = Line::Func(Func {
        id: None,
        ty: FuncType {
            params: vec![test_local!(ValType::I32)],
            results: vec![ValType::I32],
        },
        line_expression: LineExpression {
            locals: vec![],
            expr: Expression {
                instrs: vec![Instruction::LocalGet(Index::Num(0))],
            },
        },
    });
    let response = executor.execute_line(func).unwrap();
    assert_eq!(response.message(), "func ;0;");

    let call_fun = test_line![()(
        Instruction::I32Const(2),
        Instruction::Call(Index::Num(0))
    )];
    let response = executor.execute_line(call_fun).unwrap();
    assert_eq!(response.message(), "[2]");
}

#[test]
fn test_func_return() {
    let mut executor = Executor::new();
    let func = test_func!(
        "fun",
        ()(ValType::I32)(
            Instruction::I32Const(5),
            Instruction::Return,
            Instruction::Drop,
            Instruction::I32Const(6)
        )
    );
    executor.execute_line(func).unwrap();

    let call_fun = test_line![()(Instruction::Call(test_index("fun")))];
    let response = executor.execute_line(call_fun).unwrap();
    assert_eq!(response.message(), "[5]");
}

#[test]
fn test_func_return_too_many() {
    let mut executor = Executor::new();
    let func = test_func!(
        "fun",
        ()(ValType::I32, ValType::I64)(
            Instruction::I32Const(10),
            Instruction::I32Const(20),
            Instruction::I64Const(30),
            Instruction::Return
        )
    );
    executor.execute_line(func).unwrap();

    let call_fun = test_line![()(Instruction::Call(test_index("fun")))];
    let response = executor.execute_line(call_fun).unwrap();
    assert_eq!(response.message(), "[20, 30]");
}

#[test]
fn test_func_stack_overflow_error() {
    let mut executor = Executor::new();
    let func = test_func!(
        "fun",
        ()()(Instruction::Call(Index::Id(String::from("fun"))))
    );
    executor.execute_line(func).unwrap();

    let call_fun = test_line![()(Instruction::Call(test_index("fun")))];
    assert!(executor.execute_line(call_fun).is_err());
}

#[test]
fn test_return_line() {
    let mut executor = Executor::new();
    let line = test_line![()(Instruction::I32Const(5), Instruction::Return)];
    assert!(executor.execute_line(line).is_err());

    // Ensure rollback
    assert_eq!(executor.call_stack[0].stack.to_soft_string().unwrap(), "[]");
}

#[test]
fn test_if() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(12),
        Instruction::I32Const(3),
        Instruction::I32Const(1),
        test_if!((test_local!(ValType::I32), test_local!(ValType::I32))(
            ValType::I32
        )(Instruction::I32Add)(Instruction::I32Sub)),
        Instruction::I32Const(4)
    )];
    assert_eq!(executor.execute_line(line).unwrap().message(), "[15, 4]");
}

#[test]
fn test_if_execution_error() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        test_if!((test_local!(ValType::I32), test_local!(ValType::I32))(
            ValType::I32
        )(Instruction::I32Add)(Instruction::I32Sub)),
        Instruction::I32Const(4)
    )];
    assert!(executor.execute_line(line).is_err());
}

#[test]
fn test_if_param_error() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        test_if!((test_local!(ValType::I32))(ValType::I32)),
        Instruction::I32Const(4)
    )];
    assert!(executor.execute_line(line).is_err());
    assert_eq!(executor.call_stack[0].stack.to_soft_string().unwrap(), "[]");
}

#[test]
fn test_if_param_type_error() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::F32Const(1.0),
        Instruction::I32Const(1),
        test_if!((test_local!(ValType::I32))(ValType::I32)),
        Instruction::I32Const(4)
    )];
    assert!(executor.execute_line(line).is_err());
    assert_eq!(executor.call_stack[0].stack.to_soft_string().unwrap(), "[]");
}

#[test]
fn test_if_result_error() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        test_if!(()(ValType::I32)),
        Instruction::I32Const(4)
    )];
    assert!(executor.execute_line(line).is_err());
    assert_eq!(executor.call_stack[0].stack.to_soft_string().unwrap(), "[]");
}

#[test]
fn test_if_result_type_error() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        test_if!(()(ValType::I32)(Instruction::F64Const(1.0))(
            Instruction::F64Const(2.0)
        )),
        Instruction::I32Const(4)
    )];
    assert!(executor.execute_line(line).is_err());
    assert_eq!(executor.call_stack[0].stack.to_soft_string().unwrap(), "[]");
}

#[test]
fn test_if_result_too_many() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        test_if!(()(ValType::I32)(
            Instruction::I32Const(1),
            Instruction::I32Const(3)
        )(Instruction::I32Const(2), Instruction::I32Const(4))),
        Instruction::I32Const(4)
    )];
    assert!(executor.execute_line(line).is_err());
    assert_eq!(executor.call_stack[0].stack.to_soft_string().unwrap(), "[]");
}

#[test]
fn test_else() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(12),
        Instruction::I32Const(3),
        Instruction::I32Const(0),
        test_if!((test_local!(ValType::I32), test_local!(ValType::I32))(
            ValType::I32
        )(Instruction::I32Add)(Instruction::I32Sub)),
        Instruction::I32Const(4)
    )];
    assert_eq!(executor.execute_line(line).unwrap().message(), "[9, 4]");
}

#[test]
fn test_nested_if() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        test_if!(()(ValType::I32)(
            Instruction::I32Const(2),
            test_if!(()(ValType::I32)(Instruction::I32Const(3))(
                Instruction::I32Const(5)
            ))
        )(Instruction::I32Const(4))),
        Instruction::I32Const(6)
    )];
    assert_eq!(executor.execute_line(line).unwrap().message(), "[3, 6]");
}

#[test]
fn test_skip_nested_if() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(-1),
        test_if!(()(ValType::I32)(
            Instruction::I32Const(2),
            test_if!(()(ValType::I32)(Instruction::I32Const(3))(
                Instruction::I32Const(5)
            ))
        )(Instruction::I32Const(4))),
        Instruction::I32Const(6)
    )];
    assert_eq!(executor.execute_line(line).unwrap().message(), "[4, 6]");
}

#[test]
fn test_no_if() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(3),
        test_if!(()()()(Instruction::I32Const(5), Instruction::Drop)),
        Instruction::I32Const(2)
    )];
    assert_eq!(executor.execute_line(line).unwrap().message(), "[2]");
}

#[test]
fn test_no_else() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(-2),
        test_if!(()()(Instruction::I32Const(5), Instruction::Drop)()),
        Instruction::I32Const(2)
    )];
    assert_eq!(executor.execute_line(line).unwrap().message(), "[2]");
}

#[test]
fn test_func_nested_return() {
    let mut executor = Executor::new();
    let func = test_func!(
        "fn",
        (test_local!(ValType::I32))(ValType::I32)(
            Instruction::LocalGet(Index::Num(0)),
            test_if!(()(ValType::I32)(
                Instruction::I32Const(2),
                Instruction::Return,
                Instruction::I32Const(3)
            )(Instruction::I32Const(4))),
            Instruction::I32Const(5),
            Instruction::Return
        )
    );
    executor.execute_line(func).unwrap();

    let call_sub = test_line![()(
        Instruction::I32Const(1),
        Instruction::Call(test_index("fn"))
    )];
    assert_eq!(executor.execute_line(call_sub).unwrap().message(), "[2]");

    let call_sub = test_line![()(
        Instruction::Drop,
        Instruction::I32Const(-1),
        Instruction::Call(test_index("fn"))
    )];
    assert_eq!(executor.execute_line(call_sub).unwrap().message(), "[5]");
}

#[test]
fn test_func_nested_too_many() {
    let mut executor = Executor::new();
    let func = test_func!(
        "fn",
        ()(ValType::I32)(
            Instruction::I32Const(1),
            test_block!(()(ValType::I32)(
                Instruction::I32Const(2),
                Instruction::Return,
                Instruction::I32Const(3)
            ))
        )
    );
    executor.execute_line(func).unwrap();

    let call_func = test_line![()(Instruction::Call(test_index("fn")))];
    assert!(executor.execute_line(call_func).is_err());
}

#[test]
fn test_block() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        Instruction::I32Const(2),
        test_block!((test_local!(ValType::I32))(ValType::I32)(
            Instruction::I32Const(3),
            Instruction::I32Add
        )),
        Instruction::I32Const(4)
    )];
    assert_eq!(executor.execute_line(line).unwrap().message(), "[1, 5, 4]");
}

#[test]
fn test_nested_block() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        test_block!(()(ValType::I32, ValType::I32)(
            Instruction::I32Const(2),
            test_block!(()(ValType::I32)(Instruction::I32Const(3)))
        )),
        Instruction::I32Const(4)
    )];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "[1, 2, 3, 4]"
    );
}

#[test]
fn test_block_branch() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        test_block!(()(ValType::I32)(
            Instruction::I32Const(2),
            Instruction::Br(Index::Num(0)),
            Instruction::I32Const(3)
        )),
        Instruction::I32Const(4)
    )];
    assert_eq!(executor.execute_line(line).unwrap().message(), "[1, 2, 4]");
}

#[test]
fn test_nested_branch_inner_block() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        test_block!(()(ValType::I32, ValType::I32, ValType::I32)(
            Instruction::I32Const(2),
            test_block!(()(ValType::I32)(
                Instruction::I32Const(3),
                Instruction::Br(Index::Num(0)),
                Instruction::I32Const(4)
            )),
            Instruction::I32Const(5)
        )),
        Instruction::I32Const(6)
    )];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "[1, 2, 3, 5, 6]"
    );
}

#[test]
fn test_nested_branch_outer_block() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        test_block!(()(ValType::I32, ValType::I32)(
            Instruction::I32Const(2),
            test_block!(()(ValType::I32)(
                Instruction::I32Const(4),
                Instruction::Br(Index::Num(1)),
                Instruction::I32Const(5)
            )),
            Instruction::I32Const(6)
        )),
        Instruction::I32Const(7)
    )];
    assert_eq!(
        executor.execute_line(line).unwrap().message(),
        "[1, 2, 4, 7]"
    );
}

#[test]
fn test_branch_too_many() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        test_block!(()(ValType::I32)(
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::Br(Index::Num(0)),
            Instruction::I32Const(4)
        )),
        Instruction::I32Const(7)
    )];
    assert_eq!(executor.execute_line(line).unwrap().message(), "[1, 3, 7]");
}

#[test]
fn test_branch_nested_too_many() {
    let mut executor = Executor::new();
    let line = test_line![()(
        Instruction::I32Const(1),
        test_block!(()(ValType::I32)(
            Instruction::I32Const(2),
            test_block!(()(ValType::I32)(
                Instruction::I32Const(3),
                Instruction::Br(Index::Num(1)),
                Instruction::I32Const(4)
            ))
        ))
    )];
    assert!(executor.execute_line(line).is_err());
}

// TODO: tests:
// - Branch too outer error
// - Branch too outer to function
// - Branch with id
// - Branch with function (return)
// - Branch with function id error
// - Branch from if

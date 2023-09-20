use anyhow::{anyhow, Error, Result};

use crate::elements::Elements;
use crate::handler::Handler;
use crate::locals::Locals;
use crate::model::{Func, Index, Instruction, Local, ValType};
use crate::response::{Control, Response};
use crate::value::Value;
use crate::{
    model::{Line, LineExpression},
    stack::Stack,
};

pub struct State {
    pub stack: Stack,
    pub locals: Locals,
}

impl State {
    pub fn new() -> State {
        State {
            stack: Stack::new(),
            locals: Locals::new(),
        }
    }

    fn commit(&mut self) {
        self.stack.commit();
        self.locals.commit();
    }

    fn rollback(&mut self) {
        self.stack.rollback();
        self.locals.rollback();
    }
}

pub struct Executor {
    call_stack: Vec<State>,
    funcs: Elements<Func>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            // Initialize with REPL's root state
            call_stack: vec![State::new()],
            funcs: Elements::new(),
        }
    }

    pub fn execute_line(&mut self, line: Line) -> Result<Response> {
        match line {
            Line::Expression(line) => self.execute_repl_line(line),
            Line::Func(func) => self.execute_add_func(func),
        }
    }

    fn to_state(&self) -> String {
        self.call_stack[0].stack.to_string()
    }

    fn execute_add_func(&mut self, func: Func) -> Result<Response> {
        let id = func.id.clone();
        self.funcs
            .grow(func.id.clone(), func)
            .map(|i| Response::new_index("func", i, id))
    }

    fn execute_repl_line(&mut self, line: LineExpression) -> Result<Response> {
        let result = self.execute_line_expression(line);
        let stack = self.call_stack.last_mut().unwrap();

        match result {
            Ok(response) => {
                if response.contd == Control::Return {
                    stack.rollback();
                    Err(anyhow!("return is allowed only in func"))
                } else {
                    stack.commit();
                    Ok(response)
                }
            }
            Err(err) => {
                stack.rollback();
                Err(err)
            }
        }
        .map(|mut resp| {
            resp.add_message(format!("{}", self.to_state()));
            resp
        })
    }

    fn execute_func(&mut self, index: &Index) -> Result<Response> {
        let (func_state, mut func) = self.prepare_func_call(index)?;
        self.call_stack.push(func_state);

        // Ignoring the response messages from function execution
        // to reduce noise in REPL
        let response = self.execute_line_expression(func.line_expression)?;

        let mut func_state = self.call_stack.pop().unwrap();
        let mut values = vec![];
        while func.results.len() > 0 {
            let value = func_state.stack.pop()?;
            let result = func.results.pop().unwrap();
            value.is_same_type(&result)?;
            values.push(value);
        }

        let prev_stack = &mut self.call_stack.last_mut().unwrap().stack;
        while values.len() > 0 {
            prev_stack.push(values.pop().unwrap());
        }

        if response.contd != Control::Return && !func_state.stack.is_empty() {
            return Err(anyhow!("Too many returns"));
        }

        Ok(Response::new())
    }

    fn prepare_func_call(&mut self, index: &Index) -> Result<(State, Func)> {
        let mut func_state = State::new();
        let mut func = self.funcs.get(index)?.clone();
        while let Some(param) = func.params.pop() {
            let val = self.call_stack.last_mut().unwrap().stack.pop()?;
            val.is_same_type(&param.val_type)?;
            func_state.locals.grow(param.id, val)?;
        }
        if !self.call_stack.last_mut().unwrap().stack.is_empty() {
            return Err(Error::msg("Too many inputs to func"));
        }
        Ok((func_state, func))
    }

    fn execute_line_expression(&mut self, line_expr: LineExpression) -> Result<Response> {
        let mut response = Response::new();
        for lc in line_expr.locals.into_iter() {
            match self.execute_local(lc).map(|resp| response.extend(resp)) {
                Ok(response) => response,
                Err(err) => {
                    return Err(err);
                }
            }
        }

        self.execute_block(&line_expr.expr.instrs)
            .map(|resp| response.extend(resp))?;

        Ok(response)
    }

    fn execute_block(&mut self, instrs: &Vec<Instruction>) -> Result<Response> {
        let mut response = Response::new();
        for instr in instrs.iter() {
            let resp = self.execute_instruction(instr)?;
            response.extend(resp);

            match &response.contd {
                Control::ExecFunc(index) => response.extend(self.execute_func(&index)?),
                Control::None => (),
                Control::Return => break,
            };
        }
        Ok(response)
    }

    fn execute_local(&mut self, lc: Local) -> Result<Response> {
        let id = lc.id.clone();
        let state = self.call_stack.last_mut().unwrap();
        state
            .locals
            .grow(lc.id.clone(), default_value(lc)?)
            .map(|i| Response::new_index("local", i, id))
    }

    fn execute_instruction(&mut self, instr: &Instruction) -> Result<Response> {
        let mut handler = Handler::new(self.call_stack.last_mut().unwrap());
        handler.handle(instr)
    }
}

fn default_value(local: Local) -> Result<Value> {
    match local.val_type {
        ValType::I32 => Ok(Value::default_i32()),
        ValType::I64 => Ok(Value::default_i64()),
        ValType::F32 => Ok(Value::default_f32()),
        ValType::F64 => Ok(Value::default_f64()),
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Expression, Func, Index, Instruction, Local, ValType};

    use crate::executor::Executor;
    use crate::model::{Line, LineExpression};
    use crate::test_utils::test_index;

    macro_rules! test_line {
        (($( $y:expr ),*)($( $x:expr ),*)) => {
            Line::Expression(LineExpression {
                locals:  vec![$( $y ),*],
                expr: Expression{
                    instrs: vec!(
                        $( $x ),*
                    )
                }
            })
        };
    }

    macro_rules! test_func {
        ($fname:expr, ($( $param:expr ),*)($( $res:expr ),*)($( $instr:expr ),*)) => {
            Line::Func(Func {
                id: Some(String::from($fname)),
                params: vec![
                    $( $param ),*
                ],
                results: vec![$( $res ),*],
                line_expression: LineExpression {
                    locals: vec![],
                    expr: Expression {
                        instrs: vec![
                            $( $instr ),*
                        ],
                    },
                },
            })
        };
    }

    macro_rules! test_local_id {
        ($id:expr, $type:expr) => {
            Local {
                id: Some(String::from($id)),
                val_type: $type,
            }
        };
    }

    macro_rules! test_local {
        ($type:expr) => {
            Local {
                id: None,
                val_type: $type,
            }
        };
    }

    #[test]
    fn test_execute_add() {
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
    fn test_execute_error_rollback() {
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
    fn execute_func() {
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
    fn execute_func_error_less_number_of_inputs() {
        let mut executor = Executor::new();
        let func = test_func!("fun", (test_local!(ValType::I32))()());
        executor.execute_line(func).unwrap();

        let call_fun = test_line![()(Instruction::Call(test_index("fun")))];
        assert!(executor.execute_line(call_fun).is_err());
    }

    #[test]
    fn execute_func_more_number_of_inputs() {
        let mut executor = Executor::new();
        let func = test_func!("fun", (test_local!(ValType::I32))()());
        executor.execute_line(func).unwrap();

        let call_fun = test_line![()(
            Instruction::I32Const(5),
            Instruction::I32Const(10),
            Instruction::Call(test_index("fun"))
        )];
        assert!(executor.execute_line(call_fun).is_err());
    }

    #[test]
    fn execute_func_error_less_number_of_outputs() {
        let mut executor = Executor::new();
        let func = test_func!("fun", ()(ValType::I32)());
        executor.execute_line(func).unwrap();

        let call = test_line![()(Instruction::Call(test_index("fun")))];
        // We expect one output but will get none hence an error
        assert!(executor.execute_line(call).is_err());
    }

    #[test]
    fn execute_func_error_more_number_of_outputs() {
        let mut executor = Executor::new();
        let func = test_func!("fun", ()()(Instruction::I32Const(5)));
        executor.execute_line(func).unwrap();

        let call = test_line![()(Instruction::Call(test_index("fun")))];
        // We expect no output but will get one hence an error
        assert!(executor.execute_line(call).is_err());
    }

    #[test]
    fn execute_func_input_type() {
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
    fn execute_func_error_input_type() {
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
    fn execute_func_output_type() {
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
    fn execute_func_output_type_error() {
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
    fn execute_func_no_id() {
        let mut executor = Executor::new();
        let func = Line::Func(Func {
            id: None,
            params: vec![test_local!(ValType::I32)],
            results: vec![ValType::I32],
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
    fn test_return_line() {
        let mut executor = Executor::new();
        let line = test_line![()(Instruction::I32Const(5), Instruction::Return)];
        assert!(executor.execute_line(line).is_err());

        // Ensure rollback
        assert_eq!(executor.call_stack[0].stack.to_soft_string().unwrap(), "[]");
    }
}

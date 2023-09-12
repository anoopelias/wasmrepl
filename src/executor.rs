use anyhow::{Error, Result};

use crate::elements::Elements;
use crate::handler::Handler;
use crate::locals::Locals;
use crate::model::{Func, Index, Instruction, Local, ValType};
use crate::response::Response;
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

    fn commit(&mut self) -> Result<()> {
        self.stack.commit()?;
        self.locals.commit();
        Ok(())
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
        self.execute_line_expression(line).map(|mut resp| {
            resp.add_message(format!("{}", self.to_state()));
            resp
        })
    }

    fn execute_func(&mut self, index: &Index) -> Result<Response> {
        let (func_state, func) = self.prepare_func_call(index)?;
        let func_results = &func.results;
        self.call_stack.push(func_state);

        // Ignoring the response messages from function execution
        // to reduce noise in REPL
        self.execute_line_expression(func.line_expression)?;

        let values = self.fetch_results();
        self.validate_results(func_results, values)
    }

    fn validate_results(
        &mut self,
        func_results: &Vec<ValType>,
        mut values: Vec<Value>,
    ) -> Result<Response> {
        for result in func_results {
            let value = match values.pop() {
                Some(value) => {
                    value.is_same_type(result)?;
                    value
                }
                None => return Err(Error::msg("Not enough returns")),
            };
            self.call_stack.last_mut().unwrap().stack.push(value);
        }

        if values.len() > 0 {
            return Err(Error::msg("Too many returns"));
        }

        Ok(Response::new())
    }

    fn fetch_results(&mut self) -> Vec<Value> {
        let mut func_state = self.call_stack.pop().unwrap();
        let mut values = vec![];

        loop {
            // TODO: `pop` should return an `Option` instead of `Result`
            // so that we can properly iterate.
            let value = match func_state.stack.pop() {
                Ok(value) => value,
                Err(_) => break,
            };
            values.push(value);
        }
        values
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
                    self.call_stack.last_mut().unwrap().rollback();
                    return Err(err);
                }
            }
        }

        for instr in line_expr.expr.instrs.into_iter() {
            match self.execute_instruction(instr) {
                Ok(resp) => {
                    response.extend(resp);
                }
                Err(err) => {
                    self.call_stack.last_mut().unwrap().rollback();
                    return Err(err);
                }
            }
        }

        self.call_stack.last_mut().unwrap().commit()?;
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

    fn execute_instruction(&mut self, instr: Instruction) -> Result<Response> {
        match instr {
            Instruction::Call(index) => self.execute_func(&index),
            _ => {
                let mut handler = Handler::new(self.call_stack.last_mut().unwrap());
                handler.handle(&instr)?;
                Ok(Response::new())
            }
        }
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
}

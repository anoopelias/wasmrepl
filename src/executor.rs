use anyhow::{Error, Result};

use crate::elements::Elements;
use crate::handler::Handler;
use crate::locals::Locals;
use crate::model::{Func, Index, Instruction, Local, Response, ValType};
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
            Line::Func(func) => {
                self.funcs.grow(func.id.clone(), func)?;
                Ok(Response::new(String::from("Added func")))
            }
        }
    }

    pub fn to_state(&self) -> String {
        self.call_stack[0].stack.to_string()
    }

    fn execute_repl_line(&mut self, line: LineExpression) -> Result<Response> {
        match self.execute_line_expression(line) {
            Ok(_) => Ok(Response::new(format!("{}", self.to_state()))),
            Err(err) => Err(err),
        }
    }

    fn execute_func(&mut self, index: &Index) -> Result<()> {
        let mut func_state = State::new();

        // Prepare a state for func call
        // TODO: Can we get away without cloning?
        let mut func = self.funcs.get(index)?.clone();
        while let Some(param) = func.params.pop() {
            let val = self.call_stack.last_mut().unwrap().stack.pop()?;
            val.is_same_type(&param.val_type)?;
            func_state.locals.grow(param.id, val)?;
        }
        if !self.call_stack.last_mut().unwrap().stack.is_empty() {
            return Err(Error::msg("Too many inputs to func"));
        }

        // Make func call
        self.call_stack.push(func_state);
        self.execute_line_expression(func.line_expression)?;

        // Validate results
        let mut func_state = self.call_stack.pop().unwrap();
        let mut values = vec![];

        loop {
            let value = match func_state.stack.pop() {
                Ok(value) => value,
                Err(_) => break,
            };
            values.push(value);
        }

        for result in &func.results {
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

        Ok(())
    }

    fn execute_line_expression(&mut self, line_expr: LineExpression) -> Result<()> {
        for lc in line_expr.locals.into_iter() {
            self.execute_local(lc)?;
        }

        for instr in line_expr.expr.instrs.into_iter() {
            match self.execute_instruction(instr) {
                Ok(_) => {}
                Err(err) => {
                    self.call_stack.last_mut().unwrap().rollback();
                    return Err(err);
                }
            }
        }

        self.call_stack.last_mut().unwrap().commit()
    }

    fn execute_local(&mut self, lc: Local) -> Result<()> {
        let state = self.call_stack.last_mut().unwrap();
        state.locals.grow(lc.id.clone(), default_value(lc)?)
    }

    fn execute_instruction(&mut self, instr: Instruction) -> Result<()> {
        match instr {
            Instruction::Call(index) => self.execute_func(&index),
            _ => {
                let mut handler = Handler::new(self.call_stack.last_mut().unwrap());
                handler.handle(&instr)
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
        executor.execute_line(line).unwrap();
        assert_eq!(executor.to_state(), "[100]");
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
        executor.execute_line(line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_set_commit() {
        let mut executor = Executor::new();
        let line = test_line![(test_local!(ValType::I32))(
            Instruction::I32Const(42),
            Instruction::LocalSet(Index::Num(0)),
            Instruction::LocalGet(Index::Num(0))
        )];
        executor.execute_line(line).unwrap();
        assert_eq!(executor.to_state(), "[42]");

        let line = test_line![()(
            Instruction::Drop,
            Instruction::I32Const(55),
            Instruction::LocalSet(Index::Num(0)),
            Instruction::LocalGet(Index::Num(0))
        )];
        executor.execute_line(line).unwrap();
        assert_eq!(executor.to_state(), "[55]");
    }

    #[test]
    fn test_local_set_local_rollback() {
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
        executor.execute_line(line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_by_id() {
        let mut executor = Executor::new();

        let local = test_local_id!("num", ValType::I32);

        let set_index = Index::Id(String::from("num"));
        let get_index = Index::Id(String::from("num"));

        let line = test_line![(local)(
            Instruction::I32Const(42),
            Instruction::LocalSet(set_index),
            Instruction::LocalGet(get_index)
        )];
        executor.execute_line(line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_by_id_mix() {
        let mut executor = Executor::new();
        let local = test_local_id!("num", ValType::I32);
        let index = Index::Id(String::from("num"));

        let line = test_line![(test_local!(ValType::I32), local)(
            Instruction::I32Const(42),
            Instruction::LocalSet(index),
            Instruction::LocalGet(Index::Num(1))
        )];
        executor.execute_line(line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_set_get_i64() {
        let mut executor = Executor::new();
        let line = test_line![(test_local!(ValType::I64))(
            Instruction::I64Const(42),
            Instruction::LocalSet(Index::Num(0)),
            Instruction::LocalGet(Index::Num(0))
        )];
        executor.execute_line(line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
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
        executor.execute_line(line).unwrap();
        assert_eq!(executor.to_state(), "[3.14]");
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
        executor.execute_line(line).unwrap();
        assert_eq!(executor.to_state(), "[3.14]");
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
                Instruction::LocalGet(Index::Id(String::from("first"))),
                Instruction::LocalGet(Index::Id(String::from("first"))),
                Instruction::LocalGet(Index::Id(String::from("second"))),
                Instruction::I32Sub
            )
        );
        executor.execute_line(func).unwrap();

        let call_sub = test_line![()(
            Instruction::I32Const(7),
            Instruction::I32Const(2),
            Instruction::Call(Index::Id(String::from("subtract")))
        )];
        executor.execute_line(call_sub).unwrap();
        assert_eq!(executor.to_state(), "[7, 5]");
    }

    #[test]
    fn execute_func_error_less_number_of_inputs() {
        let mut executor = Executor::new();
        let func = test_func!("fun", (test_local!(ValType::I32))()());
        executor.execute_line(func).unwrap();

        let call_fun = test_line![()(Instruction::Call(Index::Id(String::from("fun"))))];
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
            Instruction::Call(Index::Id(String::from("fun")))
        )];
        assert!(executor.execute_line(call_fun).is_err());
    }

    #[test]
    fn execute_func_error_less_number_of_outputs() {
        let mut executor = Executor::new();
        let func = test_func!("fun", ()(ValType::I32)());
        executor.execute_line(func).unwrap();

        let call = test_line![()(Instruction::Call(Index::Id(String::from("fun"))))];
        // We expect one output but will get none hence an error
        assert!(executor.execute_line(call).is_err());
    }

    #[test]
    fn execute_func_error_more_number_of_outputs() {
        let mut executor = Executor::new();
        let func = test_func!("fun", ()()(Instruction::I32Const(5)));
        executor.execute_line(func).unwrap();

        let call = test_line![()(Instruction::Call(Index::Id(String::from("fun"))))];
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
            Instruction::Call(Index::Id(String::from("fun")))
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
            Instruction::Call(Index::Id(String::from("fun")))
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

        let call_fun = test_line![()(Instruction::Call(Index::Id(String::from("fun"))))];
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

        let call_fun = test_line![()(Instruction::Call(Index::Id(String::from("fun"))))];

        assert!(executor.execute_line(call_fun).is_err());
    }
}

use anyhow::{anyhow, Result};

use crate::elements::Elements;
use crate::group::{Command, Group};
use crate::handler::Handler;
use crate::locals::Locals;
use crate::model::{Func, FuncType, Index, Instruction, Local, ValType};
use crate::response::{Control, Response};
use crate::value::Value;
use crate::{
    model::{Line, LineExpression},
    stack::Stack,
};

const MAX_STACK_SIZE: i32 = 100;

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
        let result = self.execute_line_expression(&line);
        let state = self.call_stack.last_mut().unwrap();

        match result {
            Ok(response) => {
                if response.control == Control::Return {
                    state.rollback();
                    Err(anyhow!("return is allowed only in func"))
                } else {
                    state.commit();
                    Ok(response)
                }
            }
            Err(err) => {
                state.rollback();
                Err(err)
            }
        }
        .map(|mut resp| {
            resp.add_message(format!("{}", self.to_state()));
            resp
        })
    }

    fn execute_func(&mut self, index: &Index) -> Result<Response> {
        if self.call_stack.len() > MAX_STACK_SIZE as usize {
            return Err(anyhow!("Stack overflow"));
        }

        let func = self.funcs.get(index)?.clone();
        self.push_func_state(&func.ty)?;
        let response = self.execute_line_expression(&func.line_expression)?;

        self.pop_state(&func.ty, response.control != Control::Return)?;
        Ok(Response::new())
    }

    fn push_func_state(&mut self, ty: &FuncType) -> Result<()> {
        let mut func_state = State::new();
        for param in ty.params.iter().rev() {
            let val = self.call_stack.last_mut().unwrap().stack.pop()?;
            val.is_same_type(&param.val_type)?;
            func_state.locals.grow(param.id.clone(), val)?;
        }
        self.call_stack.push(func_state);
        Ok(())
    }

    fn push_group_state(&mut self, ty: &FuncType) -> Result<()> {
        let mut group_state = State::new();
        let mut values = vec![];
        for param in ty.params.iter().rev() {
            let val = self.call_stack.last_mut().unwrap().stack.pop()?;
            val.is_same_type(&param.val_type)?;
            values.push(val);
        }
        while values.len() > 0 {
            group_state.stack.push(values.pop().unwrap());
        }
        self.call_stack.push(group_state);
        Ok(())
    }

    fn pop_state(&mut self, ty: &FuncType, requires_empty: bool) -> Result<()> {
        let mut state = self.call_stack.pop().unwrap();
        let mut values = vec![];
        for result in ty.results.iter().rev() {
            let value = state.stack.pop()?;
            value.is_same_type(&result)?;
            values.push(value);
        }

        let prev_stack = &mut self.call_stack.last_mut().unwrap().stack;
        while values.len() > 0 {
            prev_stack.push(values.pop().unwrap());
        }

        if requires_empty && !state.stack.is_empty() {
            return Err(anyhow!("Too many returns"));
        }

        Ok(())
    }

    fn execute_line_expression(&mut self, line: &LineExpression) -> Result<Response> {
        let mut response = Response::new();
        for lc in line.locals.iter() {
            match self.execute_local(&lc) {
                Ok(resp) => response.extend(resp),
                Err(err) => {
                    return Err(err);
                }
            }
        }

        response.extend(self.execute_group(&line.expr.group)?);
        Ok(response)
    }

    fn execute_group(&mut self, group: &Group) -> Result<Response> {
        for command in &group.commands {
            let response = self.execute_command(command)?;
            if response.control == Control::Return {
                // Return statement break all recursive blocks
                // returning to calling function
                return Ok(response);
            }
        }
        Ok(Response::new())
    }

    fn execute_command(&mut self, command: &Command) -> Result<Response> {
        let (instr, if_group, else_group) = match command {
            Command::Instr(instr) => (instr, None, None),
            Command::If(instr, if_group, else_group) => (instr, Some(if_group), Some(else_group)),
            Command::Block(_, _) => todo!(),
        };
        let response = self.execute_instruction(instr)?;

        match response.control {
            Control::None => Ok(Response::new()),
            Control::ExecFunc(index) => self.execute_func(&index),
            Control::If(b) => {
                let group = (if b { if_group } else { else_group }).unwrap();
                self.execute_if(instr, group)
            }
            Control::Return => Ok(response),
        }
    }

    fn execute_if(&mut self, instr: &Instruction, group: &Group) -> Result<Response> {
        if let Instruction::If(block_type) = instr {
            self.push_group_state(&block_type.ty)?;
            let response = self.execute_group(group)?;
            self.pop_state(&block_type.ty, response.control != Control::Return)?;
            Ok(response)
        } else {
            unreachable!()
        }
    }

    fn execute_local(&mut self, lc: &Local) -> Result<Response> {
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

fn default_value(local: &Local) -> Result<Value> {
    match local.val_type {
        ValType::I32 => Ok(Value::default_i32()),
        ValType::I64 => Ok(Value::default_i64()),
        ValType::F32 => Ok(Value::default_f32()),
        ValType::F64 => Ok(Value::default_f64()),
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{
        Expression, Func, FuncType, Index, Instruction, Line, LineExpression, Local, ValType,
    };

    use crate::executor::Executor;
    use crate::group::{group, Command, Group};
    use crate::test_utils::{test_if, test_index};

    macro_rules! test_line {
        (($( $y:expr ),*)($( $x:expr ),*)) => {
            Line::Expression(LineExpression {
                locals:  vec![$( $y ),*],
                expr: Expression{
                    group: group((
                        vec![$( $x ),*]
                    )).unwrap()
                }
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
                    expr: Expression {
                        group: group((
                            vec![$( $instr ),*]
                        )).unwrap()
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
            ty: FuncType {
                params: vec![test_local!(ValType::I32)],
                results: vec![ValType::I32],
            },
            line_expression: LineExpression {
                locals: vec![],
                expr: Expression {
                    group: Group {
                        commands: vec![Command::Instr(Instruction::LocalGet(Index::Num(0)))],
                    },
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
    fn execute_func_stack_overflow_error() {
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
            )),
            Instruction::I32Add,
            Instruction::Else,
            Instruction::I32Sub,
            Instruction::End,
            Instruction::I32Const(4)
        )];
        assert_eq!(executor.execute_line(line).unwrap().message(), "[15, 4]");
    }

    #[test]
    fn test_if_execution_error() {
        let mut executor = Executor::new();
        let line = test_line![()(
            Instruction::I32Const(1),
            test_if!(()(ValType::I32)),
            Instruction::I32Add,
            Instruction::Else,
            Instruction::I32Const(3),
            Instruction::End,
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
            Instruction::Else,
            Instruction::End,
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
            Instruction::Else,
            Instruction::End,
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
            Instruction::Else,
            Instruction::End,
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
            test_if!(()(ValType::I32)),
            Instruction::F64Const(1.0),
            Instruction::Else,
            Instruction::End,
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
            test_if!(()(ValType::I32)),
            Instruction::I32Const(1),
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::End,
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
            )),
            Instruction::I32Add,
            Instruction::Else,
            Instruction::I32Sub,
            Instruction::End,
            Instruction::I32Const(4)
        )];
        assert_eq!(executor.execute_line(line).unwrap().message(), "[9, 4]");
    }

    #[test]
    fn test_nested_if() {
        let mut executor = Executor::new();
        let line = test_line![()(
            Instruction::I32Const(1),
            test_if!(()(ValType::I32)),
            Instruction::I32Const(2),
            test_if!(()(ValType::I32)),
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::End,
            Instruction::Else,
            Instruction::I32Const(5),
            Instruction::End,
            Instruction::I32Const(6)
        )];
        assert_eq!(executor.execute_line(line).unwrap().message(), "[3, 6]");
    }

    #[test]
    fn test_skip_nested_if() {
        let mut executor = Executor::new();
        let line = test_line![()(
            Instruction::I32Const(-1),
            test_if!(()(ValType::I32)),
            Instruction::I32Const(2),
            test_if!(()(ValType::I32)),
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::End,
            Instruction::Else,
            Instruction::I32Const(5),
            Instruction::End,
            Instruction::I32Const(6)
        )];
        assert_eq!(executor.execute_line(line).unwrap().message(), "[5, 6]");
    }

    #[test]
    fn test_no_if() {
        let mut executor = Executor::new();
        let line = test_line![()(
            Instruction::I32Const(3),
            test_if!(()()),
            Instruction::Else,
            Instruction::I32Const(1),
            Instruction::End,
            Instruction::I32Const(2)
        )];
        assert_eq!(executor.execute_line(line).unwrap().message(), "[2]");
    }

    #[test]
    fn test_no_else() {
        let mut executor = Executor::new();
        let line = test_line![()(
            Instruction::I32Const(-2),
            test_if!(()()),
            Instruction::I32Const(1),
            Instruction::End,
            Instruction::I32Const(2)
        )];
        assert_eq!(executor.execute_line(line).unwrap().message(), "[2]");
    }

    #[test]
    fn execute_nested_return() {
        let mut executor = Executor::new();
        let func = test_func!(
            "fn",
            (test_local!(ValType::I32))(ValType::I32)(
                Instruction::LocalGet(Index::Num(0)),
                test_if!(()(ValType::I32)),
                Instruction::I32Const(1),
                Instruction::Return,
                Instruction::I32Const(2),
                Instruction::Else,
                Instruction::I32Const(3),
                Instruction::End,
                Instruction::I32Const(4),
                Instruction::Return
            )
        );
        executor.execute_line(func).unwrap();

        let call_sub = test_line![()(
            Instruction::I32Const(1),
            Instruction::Call(test_index("fn"))
        )];
        assert_eq!(executor.execute_line(call_sub).unwrap().message(), "[1]");

        let call_sub = test_line![()(
            Instruction::Drop,
            Instruction::I32Const(-1),
            Instruction::Call(test_index("fn"))
        )];
        assert_eq!(executor.execute_line(call_sub).unwrap().message(), "[4]");
    }

    #[test]
    fn test_nested_no_else() {
        let mut executor = Executor::new();
        let line = test_line![()(
            Instruction::I32Const(1),
            test_if!(()()),
            Instruction::I32Const(-1),
            test_if!(()()),
            Instruction::I32Const(3),
            Instruction::End,
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::End
        )];
        assert_eq!(executor.execute_line(line).unwrap().message(), "[]");
    }
}

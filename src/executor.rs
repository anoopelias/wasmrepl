use anyhow::{Error, Result};

use crate::handler::Handler;
use crate::locals::Locals;
use crate::model::{Instruction, Local, ValType};
use crate::value::Value;
use crate::{
    model::{Line, LineExpression},
    stack::Stack,
};

pub struct Executor {
    state: State,
}

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

impl Executor {
    pub fn new() -> Executor {
        Executor {
            state: State::new(),
        }
    }

    pub fn execute(&mut self, line: &Line) -> Result<()> {
        match line {
            Line::Expression(line) => self.execute_line_expression(line),
            Line::Func(_) => Err(Error::msg("Func not supported yet")),
        }
    }

    pub fn to_state(&self) -> String {
        self.state.stack.to_string()
    }

    fn execute_line_expression(&mut self, line_expr: &LineExpression) -> Result<()> {
        for lc in line_expr.locals.iter() {
            self.execute_local(lc)?
        }

        for instr in line_expr.expr.instrs.iter() {
            match self.execute_instruction(instr) {
                Ok(_) => {}
                Err(err) => {
                    self.state.rollback();
                    return Err(err);
                }
            }
        }

        self.state.commit()?;
        Ok(())
    }

    fn execute_local(&mut self, lc: &Local) -> Result<()> {
        match &lc.id {
            Some(id) => self.state.locals.grow_by_id(&id, default_value(lc)?),
            None => {
                self.state.locals.grow(default_value(lc)?);
                Ok(())
            }
        }
    }

    fn execute_instruction(&mut self, instr: &Instruction) -> Result<()> {
        let mut handler = Handler::new(&mut self.state);
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
    use crate::model::{Expression, Index, Instruction, Local, ValType};

    use crate::executor::Executor;
    use crate::model::{Line, LineExpression};

    // An instruction that is not implemented yet,
    // to be used to force an error
    const TODO_INSTRUCTION: Instruction = Instruction::F32Copysign;

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

    macro_rules! test_new_local {
        ($fname:ident, $type:expr) => {
            fn $fname() -> Local {
                Local {
                    id: None,
                    val_type: $type,
                }
            }
        };
    }

    test_new_local!(test_new_local_i32, ValType::I32);
    test_new_local!(test_new_local_i64, ValType::I64);

    #[test]
    fn test_execute_add() {
        let mut executor = Executor::new();
        let line = test_line![()(
            Instruction::I32Const(42),
            Instruction::I32Const(58),
            Instruction::I32Add
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[100]");
    }

    #[test]
    fn test_execute_error_rollback() {
        let mut executor = Executor::new();
        let line = test_line![()(Instruction::I32Const(55))];
        executor.execute(&line).unwrap();

        let line = test_line![()(Instruction::I32Const(42), TODO_INSTRUCTION)];
        assert!(executor.execute(&line).is_err());
        // Ensure rollback
        assert_eq!(executor.state.stack.to_soft_string().unwrap(), "[55]");
    }

    #[test]
    fn test_local_set_get() {
        let mut executor = Executor::new();
        let line = test_line![(test_new_local_i32())(
            Instruction::I32Const(42),
            Instruction::LocalSet(Index::Num(0)),
            Instruction::LocalGet(Index::Num(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_set_commit() {
        let mut executor = Executor::new();
        let line = test_line![(test_new_local_i32())(
            Instruction::I32Const(42),
            Instruction::LocalSet(Index::Num(0)),
            Instruction::LocalGet(Index::Num(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");

        let line = test_line![()(
            Instruction::Drop,
            Instruction::I32Const(55),
            Instruction::LocalSet(Index::Num(0)),
            Instruction::LocalGet(Index::Num(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[55]");
    }

    #[test]
    fn test_local_set_local_rollback() {
        let mut executor = Executor::new();
        let line = test_line![(test_new_local_i32())(
            Instruction::I32Const(42),
            Instruction::LocalSet(Index::Num(0))
        )];
        executor.execute(&line).unwrap();

        let line = test_line![()(
            Instruction::I32Const(55),
            Instruction::LocalSet(Index::Num(0)),
            TODO_INSTRUCTION
        )];
        assert!(executor.execute(&line).is_err());

        let line = test_line![(test_new_local_i32())(Instruction::LocalGet(Index::Num(0)))];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_by_id() {
        let mut executor = Executor::new();

        let local = Local {
            id: Some(String::from("num")),
            val_type: ValType::I32,
        };

        let set_index = Index::Id(String::from("num"));
        let get_index = Index::Id(String::from("num"));

        let line = test_line![(local)(
            Instruction::I32Const(42),
            Instruction::LocalSet(set_index),
            Instruction::LocalGet(get_index)
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_by_id_mix() {
        let mut executor = Executor::new();
        let local = Local {
            id: Some(String::from("num")),
            val_type: ValType::I32,
        };

        let index = Index::Id(String::from("num"));

        let line = test_line![(test_new_local_i32(), local)(
            Instruction::I32Const(42),
            Instruction::LocalSet(index),
            Instruction::LocalGet(Index::Num(1))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_set_get_i64() {
        let mut executor = Executor::new();
        let line = test_line![(test_new_local_i64())(
            Instruction::I64Const(42),
            Instruction::LocalSet(Index::Num(0)),
            Instruction::LocalGet(Index::Num(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_set_get_f32() {
        let mut executor = Executor::new();
        let local = Local {
            id: None,
            val_type: ValType::F32,
        };
        let line = test_line![(local)(
            Instruction::F32Const(3.14),
            Instruction::LocalSet(Index::Num(0)),
            Instruction::LocalGet(Index::Num(0))
        )];
        executor.execute(&line).unwrap();
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
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[3.14]");
    }

    #[test]
    fn test_local_set_get_type_error() {
        let mut executor = Executor::new();
        let line = test_line![(test_new_local_i32())(
            Instruction::I64Const(55),
            Instruction::LocalSet(Index::Num(0))
        )];
        assert!(executor.execute(&line).is_err());
    }
}

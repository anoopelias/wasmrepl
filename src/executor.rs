use anyhow::{Error, Result};
use wast::core::{Instruction, Local, ValType};

use crate::locals::Locals;
use crate::parser::Line;
use crate::value::Value;
use crate::wast_handler::WastHandler;
use crate::{parser::LineExpression, stack::Stack};

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
        match lc.id {
            Some(id) => self.state.locals.grow_by_id(id.name(), default_value(lc)?),
            None => {
                self.state.locals.grow(default_value(lc)?);
                Ok(())
            }
        }
    }

    fn execute_instruction(&mut self, instr: &Instruction) -> Result<()> {
        let mut handler = WastHandler::new(&mut self.state);
        handler.handle(instr)
    }
}

fn default_value(lc: &Local) -> Result<Value> {
    match lc.ty {
        ValType::I32 => Ok(Value::default_i32()),
        ValType::I64 => Ok(Value::default_i64()),
        ValType::F32 => Ok(Value::default_f32()),
        ValType::F64 => Ok(Value::default_f64()),
        _ => Err(Error::msg("Unsupported type")),
    }
}

#[cfg(test)]
mod tests {
    use wast::core::{Expression, Instruction, Local, LocalParser, ValType};
    use wast::parser::{self as wastparser, ParseBuffer};
    use wast::token::{Id, Index, Span};

    use crate::executor::Executor;
    use crate::parser::{Line, LineExpression};
    use crate::test_utils::{float32_for, float64_for};

    // An instruction that is not implemented yet,
    // to be used to force an error
    const TODO_INSTRUCTION: Instruction = Instruction::F32Copysign;

    macro_rules! test_line {
        (($( $y:expr ),*)($( $x:expr ),*)) => {
            Line::Expression(LineExpression {
                locals:  vec![$( $y ),*],
                expr: Expression{
                    instrs: Box::new([
                        $( $x ),*
                    ])
                }
            })
        };
    }

    macro_rules! test_new_local {
        ($fname:ident, $type:expr) => {
            fn $fname<'a>() -> Local<'a> {
                Local {
                    id: None,
                    name: None,
                    ty: $type,
                }
            }
        };
    }

    test_new_local!(test_new_local_i32, ValType::I32);
    test_new_local!(test_new_local_i64, ValType::I64);
    test_new_local!(test_new_local_f32, ValType::F32);
    test_new_local!(test_new_local_f64, ValType::F64);

    fn test_local_command_for(id: &String) -> String {
        let mut command = String::from("local ");
        command.push_str(id);
        command.push_str(" i32");
        command
    }

    fn test_new_local_id<'a>(buf: &'a ParseBuffer) -> Local<'a> {
        wastparser::parse::<LocalParser>(buf)
            .unwrap()
            .locals
            .pop()
            .unwrap()
    }

    fn test_new_index<'a>(n: u32) -> Index<'a> {
        Index::Num(n, Span::from_offset(0))
    }

    fn test_new_index_id<'a>(buf: &'a ParseBuffer) -> Index<'a> {
        let id = wastparser::parse::<Id>(buf).unwrap();
        Index::Id(id)
    }

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
            Instruction::LocalSet(test_new_index(0)),
            Instruction::LocalGet(test_new_index(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_set_commit() {
        let mut executor = Executor::new();
        let line = test_line![(test_new_local_i32())(
            Instruction::I32Const(42),
            Instruction::LocalSet(test_new_index(0)),
            Instruction::LocalGet(test_new_index(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");

        let line = test_line![()(
            Instruction::Drop,
            Instruction::I32Const(55),
            Instruction::LocalSet(test_new_index(0)),
            Instruction::LocalGet(test_new_index(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[55]");
    }

    #[test]
    fn test_local_set_local_rollback() {
        let mut executor = Executor::new();
        let line = test_line![(test_new_local_i32())(
            Instruction::I32Const(42),
            Instruction::LocalSet(test_new_index(0))
        )];
        executor.execute(&line).unwrap();

        let line = test_line![()(
            Instruction::I32Const(55),
            Instruction::LocalSet(test_new_index(0)),
            TODO_INSTRUCTION
        )];
        assert!(executor.execute(&line).is_err());

        let line = test_line![(test_new_local_i32())(Instruction::LocalGet(
            test_new_index(0)
        ))];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_by_id() {
        let mut executor = Executor::new();

        let id = String::from("$num");
        let command = test_local_command_for(&id);

        let buf_command = ParseBuffer::new(&command).unwrap();
        let local = test_new_local_id(&buf_command);

        let buf_id = ParseBuffer::new(&id).unwrap();
        let id = test_new_index_id(&buf_id);

        let line = test_line![(local)(
            Instruction::I32Const(42),
            Instruction::LocalSet(id),
            Instruction::LocalGet(id)
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_by_id_mix() {
        let mut executor = Executor::new();

        let id = String::from("$num");

        let command = test_local_command_for(&id);
        let buf_command = ParseBuffer::new(&command).unwrap();
        let local = test_new_local_id(&buf_command);

        let buf_id = ParseBuffer::new(&id).unwrap();
        let id = test_new_index_id(&buf_id);

        let line = test_line![(test_new_local_i32(), local)(
            Instruction::I32Const(42),
            Instruction::LocalSet(id),
            Instruction::LocalGet(test_new_index(1))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_set_get_i64() {
        let mut executor = Executor::new();
        let line = test_line![(test_new_local_i64())(
            Instruction::I64Const(42),
            Instruction::LocalSet(test_new_index(0)),
            Instruction::LocalGet(test_new_index(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_set_get_f32() {
        let mut executor = Executor::new();
        let wat = "3.14";
        let buf = ParseBuffer::new(wat).unwrap();
        let line = test_line![(test_new_local_f32())(
            Instruction::F32Const(float32_for(&buf)),
            Instruction::LocalSet(test_new_index(0)),
            Instruction::LocalGet(test_new_index(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[3.14]");
    }

    #[test]
    fn test_local_set_get_f64() {
        let mut executor = Executor::new();
        let wat = "3.14";
        let buf = ParseBuffer::new(wat).unwrap();
        let line = test_line![(test_new_local_f64())(
            Instruction::F64Const(float64_for(&buf)),
            Instruction::LocalSet(test_new_index(0)),
            Instruction::LocalGet(test_new_index(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[3.14]");
    }

    #[test]
    fn test_local_set_get_type_error() {
        let mut executor = Executor::new();
        let line = test_line![(test_new_local_i32())(
            Instruction::I64Const(55),
            Instruction::LocalSet(test_new_index(0))
        )];
        assert!(executor.execute(&line).is_err());
    }
}

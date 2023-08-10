use anyhow::{Error, Result};
use wast::core::{Instruction, Local};
use wast::token::Index;

use crate::handler::handler_for;
use crate::{locals::Locals, parser::Line, stack::Stack};

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
        for lc in line.locals.iter() {
            self.execute_local(lc)?
        }

        for instr in line.expr.instrs.iter() {
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

    pub fn to_state(&self) -> String {
        self.state.stack.to_string()
    }

    fn execute_local(&mut self, lc: &Local) -> Result<()> {
        match lc.id {
            Some(id) => self.state.locals.grow_by_id(id.name()),
            None => {
                self.state.locals.grow();
                Ok(())
            }
        }
    }

    fn execute_instruction(&mut self, instr: &Instruction) -> Result<()> {
        match handler_for(instr, &mut self.state) {
            Ok(mut handler) => return handler.handle(),
            Err(_) => {}
        };

        match instr {
            Instruction::I32Clz => {
                let n = self.state.stack.pop()?.leading_zeros().try_into()?;
                self.state.stack.push(n);
                Ok(())
            }
            Instruction::I32Ctz => {
                let n = self.state.stack.pop()?.trailing_zeros().try_into()?;
                self.state.stack.push(n);
                Ok(())
            }
            Instruction::I32Add => {
                let a = self.state.stack.pop()?;
                let b = self.state.stack.pop()?;
                self.state.stack.push(a + b);
                Ok(())
            }
            Instruction::I32Sub => {
                let a = self.state.stack.pop()?;
                let b = self.state.stack.pop()?;
                self.state.stack.push(b - a);
                Ok(())
            }
            Instruction::I32Mul => {
                let a = self.state.stack.pop()?;
                let b = self.state.stack.pop()?;
                self.state.stack.push(a * b);
                Ok(())
            }
            Instruction::I32DivS => {
                let a = self.state.stack.pop()?;
                let b = self.state.stack.pop()?;
                if a == 0 {
                    return Err(Error::msg("Division by zero"));
                }
                self.state.stack.push(b / a);
                Ok(())
            }
            Instruction::LocalGet(Index::Num(i, _)) => {
                let value = self.state.locals.get(*i as usize)?;
                self.state.stack.push(value);
                Ok(())
            }
            Instruction::LocalSet(Index::Num(i, _)) => {
                let value = self.state.stack.pop()?;
                self.state.locals.set(*i as usize, value)?;
                Ok(())
            }
            _ => Err(Error::msg("Unknown instruction")),
        }
    }
}

#[cfg(test)]
mod tests {
    use wast::core::{Expression, Instruction, Local, ValType};
    use wast::token::{Index, Span};

    use crate::executor::Executor;
    use crate::parser::Line;

    // An instruction that is not implemented yet,
    // to be used to force an error
    const TODO_INSTRUCTION: Instruction = Instruction::F32Copysign;

    macro_rules! test_line {
        (($( $y:expr ),*)($( $x:expr ),*)) => {
            Line {
                locals:  vec![$( $y ),*],
                expr: Expression{
                    instrs: Box::new([
                        $( $x ),*
                    ])
                }
            }
        };
    }

    // TODO: Combine with the previous one
    macro_rules! test_local {
        ($id:expr, $ty:expr) => {
            Local {
                id: $id,
                name: None,
                ty: $ty,
            }
        };
    }

    macro_rules! test_index {
        ($n:expr) => {
            Index::Num($n, Span::from_offset(0))
        };
    }

    #[test]
    fn test_execute_i32_const() {
        let mut executor = Executor::new();
        let line = test_line![()(Instruction::I32Const(42), Instruction::I32Const(58))];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42, 58]");
    }

    #[test]
    fn test_execute_drop() {
        let mut executor = Executor::new();
        let line = test_line![()(
            Instruction::I32Const(42),
            Instruction::I32Const(58),
            Instruction::Drop
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
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
    fn test_clz() {
        let mut executor = Executor::new();
        let line = test_line![()(Instruction::I32Const(1023), Instruction::I32Clz)];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[22]");
    }

    #[test]
    fn test_clz_max() {
        let mut executor = Executor::new();
        let line = test_line![()(Instruction::I32Const(0), Instruction::I32Clz)];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[32]");
    }

    #[test]
    fn test_ctz() {
        let mut executor = Executor::new();
        let line = test_line![()(Instruction::I32Const(1024), Instruction::I32Ctz)];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[10]");
    }

    #[test]
    fn test_ctz_max() {
        let mut executor = Executor::new();
        let line = test_line![()(Instruction::I32Const(0), Instruction::I32Ctz)];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[32]");
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
    fn test_sub() {
        let mut executor = Executor::new();
        let line = test_line![()(
            Instruction::I32Const(78),
            Instruction::I32Const(58),
            Instruction::I32Sub
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[20]");
    }

    #[test]
    fn test_mul() {
        let mut executor = Executor::new();
        let line = test_line![()(
            Instruction::I32Const(78),
            Instruction::I32Const(58),
            Instruction::I32Mul
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[4524]");
    }

    #[test]
    fn test_div_s() {
        let mut executor = Executor::new();
        let line = test_line![()(
            Instruction::I32Const(16),
            Instruction::I32Const(3),
            Instruction::I32DivS
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[5]");
    }

    #[test]
    fn test_div_s_by_zero() {
        let mut executor = Executor::new();
        let line = test_line![()(
            Instruction::I32Const(16),
            Instruction::I32Const(0),
            Instruction::I32DivS
        )];
        assert!(executor.execute(&line).is_err());
    }

    #[test]
    fn test_local_get() {
        let mut executor = Executor::new();
        let line = test_line![(test_local![None, ValType::I32])(Instruction::LocalGet(
            test_index!(0)
        ))];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[0]");
    }

    #[test]
    fn test_local_set() {
        let mut executor = Executor::new();
        let line = test_line![(test_local![None, ValType::I32])(
            Instruction::I32Const(42),
            Instruction::LocalSet(test_index!(0)),
            Instruction::LocalGet(test_index!(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_local_set_commit() {
        let mut executor = Executor::new();
        let line = test_line![(test_local![None, ValType::I32])(
            Instruction::I32Const(42),
            Instruction::LocalSet(test_index!(0)),
            Instruction::LocalGet(test_index!(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");

        let line = test_line![()(
            Instruction::Drop,
            Instruction::I32Const(55),
            Instruction::LocalSet(test_index!(0)),
            Instruction::LocalGet(test_index!(0))
        )];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[55]");
    }

    #[test]
    fn test_local_set_local_rollback() {
        let mut executor = Executor::new();
        let line = test_line![(test_local![None, ValType::I32])(
            Instruction::I32Const(42),
            Instruction::LocalSet(test_index!(0))
        )];
        executor.execute(&line).unwrap();

        let line = test_line![()(
            Instruction::I32Const(55),
            Instruction::LocalSet(test_index!(0)),
            TODO_INSTRUCTION
        )];
        assert!(executor.execute(&line).is_err());

        let line = test_line![(test_local![None, ValType::I32])(Instruction::LocalGet(
            test_index!(0)
        ))];
        executor.execute(&line).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }
}

use wast::core::{Expression, Instruction};
use anyhow::{Result, Error};

use crate::stack::Stack;

pub struct Executor {
    stack: Stack,
}

impl Executor {

    pub fn new() -> Executor {
        Executor {
            stack: Stack::new(),
        }
    }

    pub fn execute(&mut self, expr: &Expression) -> Result<()> {
        for instr in expr.instrs.iter() {
            match self.execute_instruction(instr) {
                Ok(_) => {},
                Err(err) => {
                    self.stack.rollback();
                    return Err(err);
                }
            }
        }
        self.stack.commit().unwrap();
        Ok(())
    }

    pub fn to_state(&self) -> String {
        self.stack.to_string()
    }

    fn execute_instruction(&mut self, instr: &Instruction) -> Result<()> {
        match instr {
            Instruction::I32Const(value) => {
                self.stack.push(*value);
                Ok(())
            },
            Instruction::Drop => {
                self.stack.pop()?;
                Ok(())
            },
            Instruction::I32Add => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(a + b);
                Ok(())
            },
            _ => {
                Err(Error::msg("Unknown instruction"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use wast::core::{Expression, Instruction};

    use crate::executor::Executor;

    macro_rules! test_expression {
        ($( $x:expr ),*) => {
            Expression{
                instrs: Box::new([
                    $( $x ),*
                ])
            }
        };
    }

    #[test]
    fn test_execute_i32_const() {
        let mut executor = Executor::new();
        let expr = test_expression![
            Instruction::I32Const(42),
            Instruction::I32Const(58)
        ];
        executor.execute(&expr).unwrap();
        assert_eq!(executor.to_state(), "[42, 58]");
    }

    #[test]
    fn test_execute_drop() {
        let mut executor = Executor::new();
        let expr = test_expression![
            Instruction::I32Const(42),
            Instruction::I32Const(58),
            Instruction::Drop
        ];
        executor.execute(&expr).unwrap();
        assert_eq!(executor.to_state(), "[42]");
    }

    #[test]
    fn test_execute_add() {
        let mut executor = Executor::new();
        let expr = test_expression![
            Instruction::I32Const(42),
            Instruction::I32Const(58),
            Instruction::I32Add
        ];
        executor.execute(&expr).unwrap();
        assert_eq!(executor.to_state(), "[100]");
    }

    #[test]
    fn test_execute_error_rollback() {
        let mut executor = Executor::new();
        let expr = test_expression![
            Instruction::I32Const(55)
        ];
        executor.execute(&expr).unwrap();

        let expr = test_expression![
            Instruction::I32Const(42),
            // Use an unimplimented instruction to force an error
            Instruction::F32Copysign
        ];
        assert!(executor.execute(&expr).is_err());
        // Ensure rollback
        assert_eq!(executor
            .stack
            .to_soft_string()
            .unwrap(), "[55]");
    }
}
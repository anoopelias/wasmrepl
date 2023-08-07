use anyhow::{Error, Result};
use wast::core::{Expression, Instruction};

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
                Ok(_) => {}
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
            }
            Instruction::Drop => {
                self.stack.pop()?;
                Ok(())
            }
            Instruction::I32Clz => {
                let n = self.stack.pop()?.leading_zeros().try_into()?;
                self.stack.push(n);
                Ok(())
            }
            Instruction::I32Ctz => {
                let n = self.stack.pop()?.trailing_zeros().try_into()?;
                self.stack.push(n);
                Ok(())
            }
            Instruction::I32Add => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(a + b);
                Ok(())
            }
            Instruction::I32Sub => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(b - a);
                Ok(())
            }
            Instruction::I32Mul => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(a * b);
                Ok(())
            }
            Instruction::I32DivS => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                if a == 0 {
                    return Err(Error::msg("Division by zero"));
                }
                self.stack.push(b / a);
                Ok(())
            }
            _ => Err(Error::msg("Unknown instruction")),
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
        let expr = test_expression![Instruction::I32Const(42), Instruction::I32Const(58)];
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
    fn test_execute_error_rollback() {
        let mut executor = Executor::new();
        let expr = test_expression![Instruction::I32Const(55)];
        executor.execute(&expr).unwrap();

        let expr = test_expression![
            Instruction::I32Const(42),
            // Use an unimplimented instruction to force an error
            Instruction::F32Copysign
        ];
        assert!(executor.execute(&expr).is_err());
        // Ensure rollback
        assert_eq!(executor.stack.to_soft_string().unwrap(), "[55]");
    }

    #[test]
    fn test_clz() {
        let mut executor = Executor::new();
        let expr = test_expression![Instruction::I32Const(1023), Instruction::I32Clz];
        executor.execute(&expr).unwrap();
        assert_eq!(executor.to_state(), "[22]");
    }

    #[test]
    fn test_clz_max() {
        let mut executor = Executor::new();
        let expr = test_expression![Instruction::I32Const(0), Instruction::I32Clz];
        executor.execute(&expr).unwrap();
        assert_eq!(executor.to_state(), "[32]");
    }

    #[test]
    fn test_ctz() {
        let mut executor = Executor::new();
        let expr = test_expression![Instruction::I32Const(1024), Instruction::I32Ctz];
        executor.execute(&expr).unwrap();
        assert_eq!(executor.to_state(), "[10]");
    }

    #[test]
    fn test_ctz_max() {
        let mut executor = Executor::new();
        let expr = test_expression![Instruction::I32Const(0), Instruction::I32Ctz];
        executor.execute(&expr).unwrap();
        assert_eq!(executor.to_state(), "[32]");
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
    fn test_sub() {
        let mut executor = Executor::new();
        let expr = test_expression![
            Instruction::I32Const(78),
            Instruction::I32Const(58),
            Instruction::I32Sub
        ];
        executor.execute(&expr).unwrap();
        assert_eq!(executor.to_state(), "[20]");
    }

    #[test]
    fn test_mul() {
        let mut executor = Executor::new();
        let expr = test_expression![
            Instruction::I32Const(78),
            Instruction::I32Const(58),
            Instruction::I32Mul
        ];
        executor.execute(&expr).unwrap();
        assert_eq!(executor.to_state(), "[4524]");
    }

    #[test]
    fn test_div_s() {
        let mut executor = Executor::new();
        let expr = test_expression![
            Instruction::I32Const(16),
            Instruction::I32Const(3),
            Instruction::I32DivS
        ];
        executor.execute(&expr).unwrap();
        assert_eq!(executor.to_state(), "[5]");
    }

    #[test]
    fn test_div_s_by_zero() {
        let mut executor = Executor::new();
        let expr = test_expression![
            Instruction::I32Const(16),
            Instruction::I32Const(0),
            Instruction::I32DivS
        ];
        assert!(executor.execute(&expr).is_err());
    }
}

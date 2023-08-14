use anyhow::{Error, Result};
use wast::{core::Instruction, token::Index};

use crate::executor::State;

pub trait Handler<'a> {
    fn handle(&mut self) -> Result<()>;
}

struct I32ConstInstr<'a> {
    value: i32,
    state: &'a mut State,
}

impl<'a> Handler<'a> for I32ConstInstr<'a> {
    fn handle(&mut self) -> Result<()> {
        self.state.stack.push(self.value);
        Ok(())
    }
}

struct DropInstr<'a> {
    state: &'a mut State,
}

impl<'a> Handler<'a> for DropInstr<'a> {
    fn handle(&mut self) -> Result<()> {
        self.state.stack.pop()?;
        Ok(())
    }
}

struct I32ClzInstr<'a> {
    state: &'a mut State,
}

impl<'a> Handler<'a> for I32ClzInstr<'a> {
    fn handle(&mut self) -> Result<()> {
        let value = self.state.stack.pop()?;
        self.state.stack.push(value.leading_zeros() as i32);
        Ok(())
    }
}

struct I32CtzInstr<'a> {
    state: &'a mut State,
}

impl<'a> Handler<'a> for I32CtzInstr<'a> {
    fn handle(&mut self) -> Result<()> {
        let value = self.state.stack.pop()?;
        self.state.stack.push(value.trailing_zeros() as i32);
        Ok(())
    }
}

struct I32AddInstr<'a> {
    state: &'a mut State,
}

impl<'a> Handler<'a> for I32AddInstr<'a> {
    fn handle(&mut self) -> Result<()> {
        let a = self.state.stack.pop()?;
        let b = self.state.stack.pop()?;
        self.state.stack.push(a + b);
        Ok(())
    }
}

struct I32SubInstr<'a> {
    state: &'a mut State,
}

impl<'a> Handler<'a> for I32SubInstr<'a> {
    fn handle(&mut self) -> Result<()> {
        let a = self.state.stack.pop()?;
        let b = self.state.stack.pop()?;
        self.state.stack.push(b - a);
        Ok(())
    }
}

struct I32MulInstr<'a> {
    state: &'a mut State,
}

impl<'a> Handler<'a> for I32MulInstr<'a> {
    fn handle(&mut self) -> Result<()> {
        let a = self.state.stack.pop()?;
        let b = self.state.stack.pop()?;
        self.state.stack.push(a * b);
        Ok(())
    }
}

struct I32DivSInstr<'a> {
    state: &'a mut State,
}

impl<'a> Handler<'a> for I32DivSInstr<'a> {
    fn handle(&mut self) -> Result<()> {
        let value1 = self.state.stack.pop()?;
        let value2 = self.state.stack.pop()?;
        if value1 == 0 {
            return Err(Error::msg("Division by zero"));
        }
        self.state.stack.push(value2 / value1);
        Ok(())
    }
}

struct LocalGetInstr<'a> {
    index: u32,
    state: &'a mut State,
}

impl<'a> Handler<'a> for LocalGetInstr<'a> {
    fn handle(&mut self) -> Result<()> {
        let value = self.state.locals.get(self.index as usize)?;
        self.state.stack.push(value);
        Ok(())
    }
}

struct LocalSetInstr<'a> {
    index: u32,
    state: &'a mut State,
}

impl<'a> Handler<'a> for LocalSetInstr<'a> {
    fn handle(&mut self) -> Result<()> {
        let value = self.state.stack.pop()?;
        self.state.locals.set(self.index as usize, value)?;
        Ok(())
    }
}

pub fn handler_for<'a>(
    instr: &Instruction,
    state: &'a mut State,
) -> Result<Box<dyn Handler<'a> + 'a>> {
    match instr {
        Instruction::I32Const(value) => Ok(Box::new(I32ConstInstr {
            value: *value,
            state,
        })),
        Instruction::Drop => Ok(Box::new(DropInstr { state })),
        Instruction::I32Clz => Ok(Box::new(I32ClzInstr { state })),
        Instruction::I32Ctz => Ok(Box::new(I32CtzInstr { state })),
        Instruction::I32Add => Ok(Box::new(I32AddInstr { state })),
        Instruction::I32Sub => Ok(Box::new(I32SubInstr { state })),
        Instruction::I32Mul => Ok(Box::new(I32MulInstr { state })),
        Instruction::I32DivS => Ok(Box::new(I32DivSInstr { state })),
        Instruction::LocalGet(Index::Num(index, _)) => Ok(Box::new(LocalGetInstr {
            index: *index,
            state,
        })),
        Instruction::LocalSet(Index::Num(index, _)) => Ok(Box::new(LocalSetInstr {
            index: *index,
            state,
        })),
        _ => Err(Error::msg("Unknown instruction")),
    }
}

#[cfg(test)]
mod tests {
    use crate::{executor::State, handler::handler_for};
    use anyhow::Result;

    use wast::{
        core::Instruction,
        token::{Index, Span},
    };

    fn exec_instr(instr: &Instruction, state: &mut State) -> Result<()> {
        let mut handler = handler_for(instr, state).unwrap();
        handler.handle()
    }

    #[test]
    fn test_unknown_instr() {
        let mut state = State::new();
        assert!(handler_for(&Instruction::Nop, &mut state).is_err());
    }

    #[test]
    fn test_i32_const() {
        let mut state = State::new();
        exec_instr(&Instruction::I32Const(42), &mut state).unwrap();
        assert_eq!(state.stack.pop().unwrap(), 42);
    }

    #[test]
    fn test_drop() {
        let mut state = State::new();
        state.stack.push(42);
        exec_instr(&Instruction::Drop, &mut state).unwrap();
        assert!(state.stack.pop().is_err());
    }

    #[test]
    fn test_drop_error() {
        let mut state = State::new();
        assert!(exec_instr(&Instruction::Drop, &mut state).is_err());
    }

    #[test]
    fn test_i32_clz() {
        let mut state = State::new();
        state.stack.push(1023);
        exec_instr(&Instruction::I32Clz, &mut state).unwrap();
        assert_eq!(state.stack.pop().unwrap(), 22);
    }

    #[test]
    fn test_i32_clz_max() {
        let mut state = State::new();
        state.stack.push(0);
        exec_instr(&Instruction::I32Clz, &mut state).unwrap();
        assert_eq!(state.stack.pop().unwrap(), 32);
    }

    #[test]
    fn test_i32_clz_error() {
        let mut state = State::new();
        assert!(exec_instr(&Instruction::I32Clz, &mut state).is_err());
    }

    #[test]
    fn test_i32_ctz() {
        let mut state = State::new();
        state.stack.push(1024);
        exec_instr(&Instruction::I32Ctz, &mut state).unwrap();
        assert_eq!(state.stack.pop().unwrap(), 10);
    }

    #[test]
    fn test_i32_ctz_max() {
        let mut state = State::new();
        state.stack.push(0);
        exec_instr(&Instruction::I32Ctz, &mut state).unwrap();
        assert_eq!(state.stack.pop().unwrap(), 32);
    }

    #[test]
    fn test_i32_ctz_error() {
        let mut state = State::new();
        assert!(exec_instr(&Instruction::I32Ctz, &mut state).is_err());
    }

    #[test]
    fn test_i32_add() {
        let mut state = State::new();
        state.stack.push(1);
        state.stack.push(2);
        exec_instr(&Instruction::I32Add, &mut state).unwrap();
        assert_eq!(state.stack.pop().unwrap(), 3);
    }

    #[test]
    fn test_i32_add_error() {
        let mut state = State::new();
        state.stack.push(1);
        assert!(exec_instr(&Instruction::I32Add, &mut state).is_err());
    }

    #[test]
    fn test_i32_sub() {
        let mut state = State::new();
        state.stack.push(2);
        state.stack.push(1);
        exec_instr(&Instruction::I32Sub, &mut state).unwrap();
        assert_eq!(state.stack.pop().unwrap(), 1);
    }

    #[test]
    fn test_i32_sub_error() {
        let mut state = State::new();
        state.stack.push(1);
        assert!(exec_instr(&Instruction::I32Sub, &mut state).is_err());
    }

    #[test]
    fn test_i32_mul() {
        let mut state = State::new();
        state.stack.push(2);
        state.stack.push(3);
        exec_instr(&Instruction::I32Mul, &mut state).unwrap();
        assert_eq!(state.stack.pop().unwrap(), 6);
    }

    #[test]
    fn test_i32_mul_error() {
        let mut state = State::new();
        state.stack.push(1);
        assert!(exec_instr(&Instruction::I32Mul, &mut state).is_err());
    }

    #[test]
    fn test_i32_div_s() {
        let mut state = State::new();
        state.stack.push(7);
        state.stack.push(3);
        exec_instr(&Instruction::I32DivS, &mut state).unwrap();
        assert_eq!(state.stack.pop().unwrap(), 2);
    }

    #[test]
    fn test_i32_div_s_error() {
        let mut state = State::new();
        state.stack.push(1);
        assert!(exec_instr(&Instruction::I32DivS, &mut state).is_err());
    }

    #[test]
    fn test_i32_div_s_div_by_zero() {
        let mut state = State::new();
        state.stack.push(1);
        state.stack.push(0);
        assert!(exec_instr(&Instruction::I32DivS, &mut state).is_err());
    }

    #[test]
    fn test_local_get() {
        let mut state = State::new();
        state.locals.grow();
        state.locals.set(0, 42).unwrap();
        exec_instr(
            &Instruction::LocalGet(Index::Num(0, Span::from_offset(0))),
            &mut state,
        )
        .unwrap();
        assert_eq!(state.stack.pop().unwrap(), 42);
    }

    #[test]
    fn test_local_get_error() {
        let mut state = State::new();
        assert!(exec_instr(
            &Instruction::LocalGet(Index::Num(0, Span::from_offset(0))),
            &mut state,
        )
        .is_err());
    }

    #[test]
    fn test_local_set() {
        let mut state = State::new();
        state.stack.push(15);
        state.locals.grow();
        state.locals.grow();
        exec_instr(
            &Instruction::LocalSet(Index::Num(1, Span::from_offset(0))),
            &mut state,
        )
        .unwrap();
        assert_eq!(state.locals.get(1).unwrap(), 15);
    }

    #[test]
    fn test_local_set_locals_error() {
        let mut state = State::new();
        state.stack.push(15);
        assert!(exec_instr(
            &Instruction::LocalSet(Index::Num(0, Span::from_offset(0))),
            &mut state,
        )
        .is_err());
    }

    #[test]
    fn test_local_set_stack_error() {
        let mut state = State::new();
        assert!(exec_instr(
            &Instruction::LocalSet(Index::Num(0, Span::from_offset(0))),
            &mut state,
        )
        .is_err());
    }
}

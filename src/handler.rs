use anyhow::{Error, Result};
use wast::core::Instruction;

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
        let value1 = self.state.stack.pop()?;
        let value2 = self.state.stack.pop()?;
        self.state.stack.push(value1 + value2);
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
        _ => Err(Error::msg("Unknown instruction")),
    }
}

#[cfg(test)]
mod tests {
    use crate::{executor::State, handler::handler_for};
    use anyhow::Result;

    use wast::core::Instruction;

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
}

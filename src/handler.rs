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

pub fn handler_for<'a>(instr: &Instruction, state: &'a mut State) -> Result<impl Handler<'a>> {
    match instr {
        Instruction::I32Const(value) => Ok(I32ConstInstr {
            value: *value,
            state,
        }),
        _ => Err(Error::msg("Unknown instruction")),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        executor::State,
        handler::{handler_for, Handler},
    };

    use wast::core::Instruction;

    #[test]
    fn test_i32_const() {
        let mut state = State::new();
        let instr = Instruction::I32Const(42);
        {
            let mut handler = handler_for(&instr, &mut state).unwrap();
            handler.handle().unwrap();
        }
        assert_eq!(state.stack.pop().unwrap(), 42);
    }
}

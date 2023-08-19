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
        self.state.stack.push(a.wrapping_add(b));
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
        self.state.stack.push(b.wrapping_sub(a));
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
        self.state.stack.push(a.wrapping_mul(b));
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
    id: Option<&'a str>,
    index: Option<u32>,
    state: &'a mut State,
}

impl<'a> Handler<'a> for LocalGetInstr<'a> {
    fn handle(&mut self) -> Result<()> {
        match self.id {
            Some(id) => {
                let val = self.state.locals.get_by_id(id)?;
                self.state.stack.push(val);
            }
            None => {
                let value = self.state.locals.get(self.index.unwrap() as usize)?;
                self.state.stack.push(value);
            }
        }
        Ok(())
    }
}

struct LocalSetInstr<'a> {
    id: Option<&'a str>,
    index: Option<u32>,
    state: &'a mut State,
}

impl<'a> Handler<'a> for LocalSetInstr<'a> {
    fn handle(&mut self) -> Result<()> {
        let value = self.state.stack.pop()?;
        match self.id {
            Some(id) => {
                self.state.locals.set_by_id(id, value)?;
            }
            None => self.state.locals.set(self.index.unwrap() as usize, value)?,
        }
        Ok(())
    }
}

pub struct InstructionHandler<'a> {
    state: &'a mut State,
}

impl<'a> InstructionHandler<'a> {
    pub fn new(state: &'a mut State) -> Self {
        InstructionHandler { state }
    }

    fn i32_const(&mut self, value: i32) -> Result<()> {
        self.state.stack.push(value);
        Ok(())
    }

    pub fn handle(&mut self, instr: &Instruction) -> Result<()> {
        match instr {
            Instruction::I32Const(value) => self.i32_const(*value),
            _ => Err(Error::msg("Unknown instruction")),
        }
    }
}

pub fn handler_for<'a>(
    instr: &'a Instruction,
    state: &'a mut State,
) -> Result<Box<dyn Handler<'a> + 'a>> {
    match instr {
        Instruction::Drop => Ok(Box::new(DropInstr { state })),
        Instruction::I32Clz => Ok(Box::new(I32ClzInstr { state })),
        Instruction::I32Ctz => Ok(Box::new(I32CtzInstr { state })),
        Instruction::I32Add => Ok(Box::new(I32AddInstr { state })),
        Instruction::I32Sub => Ok(Box::new(I32SubInstr { state })),
        Instruction::I32Mul => Ok(Box::new(I32MulInstr { state })),
        Instruction::I32DivS => Ok(Box::new(I32DivSInstr { state })),
        Instruction::LocalGet(Index::Num(index, _)) => Ok(Box::new(LocalGetInstr {
            id: None,
            index: Some(*index),
            state,
        })),
        Instruction::LocalSet(Index::Num(index, _)) => Ok(Box::new(LocalSetInstr {
            id: None,
            index: Some(*index),
            state,
        })),
        Instruction::LocalGet(Index::Id(id)) => Ok(Box::new(LocalGetInstr {
            id: Some(id.name()),
            index: None,
            state,
        })),
        Instruction::LocalSet(Index::Id(id)) => Ok(Box::new(LocalSetInstr {
            id: Some(id.name()),
            index: None,
            state,
        })),
        _ => Err(Error::msg("Unknown instruction")),
    }
}

#[cfg(test)]
#[path = "./handler_test.rs"]
mod handler_test;

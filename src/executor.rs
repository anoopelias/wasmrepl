use anyhow::{anyhow, Result};

use crate::elements::Elements;
use crate::handler::Handler;
use crate::locals::Locals;
use crate::model::{BlockType, Expression, Func, FuncType, Index, Instruction, Local, ValType};
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
        let result = self.execute_line_expression(line);
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
        let response = self.execute_line_expression(func.line_expression)?;

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

    fn execute_line_expression(&mut self, line: LineExpression) -> Result<Response> {
        let mut response = Response::new();
        for lc in line.locals.iter() {
            match self.execute_local(&lc) {
                Ok(resp) => response.extend(resp),
                Err(err) => {
                    return Err(err);
                }
            }
        }

        response.extend(self.execute_expr(line.expr)?);
        Ok(response)
    }

    fn execute_expr(&mut self, expr: Expression) -> Result<Response> {
        for instr in expr.instrs {
            let response = self.execute_instr(instr)?;
            // Break all recursive blocks
            // returning to calling block
            match response.control {
                Control::Return => return Ok(response),
                Control::Branch(_) => return Ok(response),
                _ => {}
            }
        }
        Ok(Response::new())
    }

    fn execute_instr(&mut self, instr: Instruction) -> Result<Response> {
        let mut handler = Handler::new(self.call_stack.last_mut().unwrap());
        let response = handler.handle(instr)?;

        match response.control {
            Control::None => Ok(response),
            Control::ExecFunc(index) => self.execute_func(&index),
            Control::ExecBlock(block_type, block) => self.execute_block(block_type, block),
            Control::Return => Ok(response),
            Control::Branch(_) => Ok(response),
        }
    }

    fn execute_block(&mut self, block_type: BlockType, expr: Expression) -> Result<Response> {
        self.push_group_state(&block_type.ty)?;
        let mut response = self.execute_expr(expr)?;

        let requires_empty = match response.control {
            Control::Branch(Index::Num(0)) => {
                response.control = Control::None;
                false
            }
            Control::Branch(Index::Num(num)) => {
                response.control = Control::Branch(Index::Num(num - 1));
                false
            }
            Control::Return => false,
            _ => true,
        };

        self.pop_state(&block_type.ty, requires_empty)?;
        Ok(response)
    }

    fn execute_local(&mut self, lc: &Local) -> Result<Response> {
        let id = lc.id.clone();
        let state = self.call_stack.last_mut().unwrap();
        state
            .locals
            .grow(lc.id.clone(), default_value(lc)?)
            .map(|i| Response::new_index("local", i, id))
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
#[path = "./executor_test.rs"]
mod executor_test;

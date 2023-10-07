use anyhow::{anyhow, Result};

use crate::call_stack::CallStack;
use crate::elements::Elements;
use crate::handler::Handler;
use crate::model::{BlockType, Expression, Func, Index, Instruction, Local, ValType};
use crate::model::{Line, LineExpression};
use crate::response::{Control, Response};
use crate::value::Value;

const MAX_STACK_SIZE: i32 = 100;

pub struct Executor {
    call_stack: CallStack,
    funcs: Elements<Func>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            call_stack: CallStack::new(),
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
        self.call_stack.to_string()
    }

    fn execute_add_func(&mut self, func: Func) -> Result<Response> {
        let id = func.id.clone();
        self.funcs
            .grow(func.id.clone(), func)
            .map(|i| Response::new_index("func", i, id))
    }

    fn execute_repl_line(&mut self, line: LineExpression) -> Result<Response> {
        let result = self.execute_line_expression(line);

        match verify_repl_result(result) {
            Ok(mut response) => {
                self.call_stack.commit();
                response.add_message(format!("{}", self.to_state()));
                Ok(response)
            }
            Err(err) => {
                self.call_stack.rollback();
                Err(err)
            }
        }
    }

    fn execute_func(&mut self, index: &Index) -> Result<Response> {
        if self.call_stack.len() > MAX_STACK_SIZE as usize {
            return Err(anyhow!("Stack overflow"));
        }

        let func = self.funcs.get(index)?.clone();
        self.call_stack.add_func_stack(&func.ty)?;
        let response = self.execute_line_expression(func.line_expression)?;

        verify_func_response(&response)?;

        self.call_stack
            .remove_func_stack(&func.ty, response.requires_empty)?;
        Ok(Response::new())
    }

    fn execute_line_expression(&mut self, line: LineExpression) -> Result<Response> {
        let mut response = Response::new();
        for lc in line.locals.into_iter() {
            match self.execute_local(lc) {
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
        let mut handler = Handler::new(self.call_stack.get_func_stack()?);
        let response = handler.handle(instr)?;

        match response.control {
            Control::ExecFunc(index) => self.execute_func(&index),
            Control::ExecBlock(block_type, block) => self.execute_block(block_type, block),
            Control::ExecLoop(block_type, block) => self.execute_loop(block_type, block),
            _ => Ok(response),
        }
    }

    fn execute_block(&mut self, block_type: BlockType, expr: Expression) -> Result<Response> {
        self.call_stack.add_block_stack(&block_type.ty)?;
        let mut response = self.execute_expr(expr)?;
        self.call_stack
            .remove_block_stack(&block_type.ty, response.requires_empty)?;

        response.control = match response.control {
            Control::Branch(Index::Num(0)) => Control::None,
            Control::Branch(Index::Num(num)) => Control::Branch(Index::Num(num - 1)),
            Control::Branch(Index::Id(ref id)) => match block_type.label {
                Some(ref block_id) if id == block_id => Control::None,
                _ => response.control,
            },
            _ => response.control,
        };

        response.requires_empty = true;
        Ok(response)
    }

    fn execute_loop(&mut self, block_type: BlockType, expr: Expression) -> Result<Response> {
        loop {
            self.call_stack.add_block_stack(&block_type.ty)?;
            let mut response = self.execute_expr(expr.clone())?;
            self.call_stack
                .remove_block_stack(&block_type.ty, response.requires_empty)?;

            response.control = match response.control {
                Control::Branch(Index::Num(0)) => continue,
                Control::Branch(Index::Num(num)) => Control::Branch(Index::Num(num - 1)),
                Control::Branch(Index::Id(ref id)) => match block_type.label {
                    Some(ref block_id) if id == block_id => continue,
                    _ => response.control,
                },
                _ => response.control,
            };
            response.requires_empty = true;
            break Ok(response);
        }
    }

    fn execute_local(&mut self, lc: Local) -> Result<Response> {
        let func_stack = self.call_stack.get_func_stack()?;
        let (id, val_type) = (lc.id, lc.val_type);
        let print_id = id.clone();
        func_stack
            .locals
            .grow(id, default_value(&val_type)?)
            .map(|i| Response::new_index("local", i, print_id))
    }
}

fn verify_func_response(response: &Response) -> Result<()> {
    match response.control {
        Control::Branch(Index::Num(0)) => Ok(()),
        Control::Branch(Index::Num(_)) => Err(anyhow!("br leaking out")),
        Control::Branch(_) => Err(anyhow!("br leaking out")),
        _ => Ok(()),
    }
}

fn verify_repl_result(result: Result<Response>) -> Result<Response> {
    match result {
        Ok(response) => match response.control {
            Control::Return => Err(anyhow!("return is allowed only in func")),
            Control::Branch(_) => Err(anyhow!("br leaking out")),
            _ => Ok(response),
        },
        Err(err) => Err(err),
    }
}

fn default_value(val_type: &ValType) -> Result<Value> {
    match val_type {
        ValType::I32 => Ok(Value::default_i32()),
        ValType::I64 => Ok(Value::default_i64()),
        ValType::F32 => Ok(Value::default_f32()),
        ValType::F64 => Ok(Value::default_f64()),
    }
}

#[cfg(test)]
#[path = "./executor_test.rs"]
mod executor_test;

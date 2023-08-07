
mod stack;
mod executor;

use anyhow::{Result, Error};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use stack::Stack;
use wast::core::{Expression, Instruction};
use wast::parser::{ParseBuffer, self};


fn main() -> rustyline::Result<()> {
    let mut rl = DefaultEditor::new()?;
    let mut stack = Stack::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                println!("{}", parse_and_execute(&mut stack, line.as_str()));
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    Ok(())
}

fn parse_and_execute(stack: &mut Stack, str: &str) -> String {
    let buf = ParseBuffer::new(str).unwrap();
    let expr = parse(&buf);

    match expr {
        Ok(expr) => {
            match execute(stack, expr) {
                Ok(_) => {
                    format!("{}", stack.to_string())
                },
                Err(err) => {
                    format!("Error: {}", err.to_string())
                }
            }
        },
        Err(err) => {
            format!("Error: {}", err.message())
        }
    }
}

fn execute(stack: &mut Stack, expr: Expression) -> Result<()> {
    for instr in expr.instrs.iter() {
        match execute_instruction(stack, instr) {
            Ok(_) => {},
            Err(err) => {
                stack.rollback();
                return Err(err);
            }
        }
    }
    stack.commit().unwrap();
    Ok(())
}

fn execute_instruction(stack: &mut Stack, instr: &Instruction) -> Result<()> {
    match instr {
        Instruction::I32Const(value) => {
            stack.push(*value);
            Ok(())
        },
        Instruction::Drop => {
            stack.pop()?;
            Ok(())
        },
        Instruction::I32Add => {
            let a = stack.pop()?;
            let b = stack.pop()?;
            stack.push(a + b);
            Ok(())
        },
        _ => {
            Err(Error::msg("Unknown instruction"))
        }
    }
}

fn parse<'a>(buf: &'a ParseBuffer) -> wast::parser::Result<Expression<'a>> {
    parser::parse::<Expression>(&buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_execute() {
        let mut stack = Stack::new();
        assert_eq!(parse_and_execute(&mut stack, "(i32.const 42)"), "[42]");
        assert_eq!(parse_and_execute(&mut stack, "(i32.const 1)"), "[42, 1]");
    }

    #[test]
    fn test_parse_error() {
        let str = "(i32.const 42";
        let buf = ParseBuffer::new(&str).unwrap();
        assert!(parse(&buf).is_err());
    }

    #[test]
    fn test_execute_unknown_error() {
        let mut stack = Stack::new();
        let str = "(f32.const 32.0)";
        let buf = ParseBuffer::new(&str).unwrap();
        let expr = parse(&buf).unwrap();
        assert!(execute(&mut stack, expr).is_err());
    }

    #[test]
    fn test_execute_preserve_stack_on_error() {
        let mut stack = Stack::new();
        parse_and_execute(&mut stack, "(i32.const 42) (f32.const 35.0)");
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_drop() {
        let mut stack = Stack::new();
        assert_eq!(parse_and_execute(&mut stack, "(i32.const 42) (drop)"), "[]");
        assert_eq!(parse_and_execute(&mut stack, "(i32.const 42) (i32.const 45) (drop)"), "[42]");
    }

    #[test]
    fn test_add() {
        let mut stack = Stack::new();
        assert_eq!(parse_and_execute(&mut stack, "(i32.const 42) (i32.const 45) (i32.add)"), "[87]");
    }

}
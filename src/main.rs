
use anyhow::{Result, Error};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use wast::core::{Expression, Instruction};
use wast::parser::{ParseBuffer, self};


fn main() -> rustyline::Result<()> {
    let mut rl = DefaultEditor::new()?;
    let mut stack = vec![];
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


fn parse_and_execute(stack: &mut Vec<i32>, str: &str) -> String {
    let buf = ParseBuffer::new(str).unwrap();
    let expr = parse(&buf);

    match expr {
        Ok(expr) => {
            match execute(stack, expr) {
                Ok(_) => {
                    format!("{:?}", stack)
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

fn execute(stack: &mut Vec<i32>, expr: Expression) -> Result<()> {
    for instr in expr.instrs.iter() {
        match instr {
            Instruction::I32Const(value) => {
                stack.push(*value);
            },
            _ => {
                return Err(Error::msg("Unknown instruction"));
            }
        }
    }
    Ok(())
}

fn parse<'a>(buf: &'a ParseBuffer) -> wast::parser::Result<Expression<'a>> {
    parser::parse::<Expression>(&buf)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_and_execute() {
        let mut stack = vec![];
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
        let mut stack = vec![];
        let str = "(f32.const 32.0)";
        let buf = ParseBuffer::new(&str).unwrap();
        let expr = parse(&buf).unwrap();
        assert!(execute(&mut stack, expr).is_err());
    }

}

mod stack;
mod executor;

use executor::Executor;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use wast::core::Expression;
use wast::parser::{ParseBuffer, self};


fn main() -> rustyline::Result<()> {
    let mut rl = DefaultEditor::new()?;
    let mut executor = Executor::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                println!("{}", parse_and_execute(&mut executor, line.as_str()));
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

fn parse_and_execute(executor: &mut Executor, str: &str) -> String {
    let buf = ParseBuffer::new(str).unwrap();
    let expr = parse(&buf);

    match expr {
        Ok(expr) => {
            match executor.execute(&expr) {
                Ok(_) => {
                    format!("{}", executor.to_state())
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

fn parse<'a>(buf: &'a ParseBuffer) -> wast::parser::Result<Expression<'a>> {
    parser::parse::<Expression>(&buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_execute() {
        let mut executor = Executor::new();
        assert_eq!(parse_and_execute(&mut executor, "(i32.const 42)"), "[42]");
        assert_eq!(parse_and_execute(&mut executor, "(i32.const 1)"), "[42, 1]");
    }

    #[test]
    fn test_parse_error() {
        let str = "(i32.const 42";
        let buf = ParseBuffer::new(&str).unwrap();
        assert!(parse(&buf).is_err());
    }

    #[test]
    fn test_drop() {
        let mut executor = Executor::new();
        assert_eq!(parse_and_execute(&mut executor, "(i32.const 42) (drop)"), "[]");
        assert_eq!(parse_and_execute(&mut executor, "(i32.const 42) (i32.const 45) (drop)"), "[42]");
    }

}
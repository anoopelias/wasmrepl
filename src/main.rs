mod dict;
mod executor;
mod list;
mod parser;
mod stack;

use executor::Executor;
use parser::Line;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use wast::parser::{self as wastparser, ParseBuffer};

fn main() -> rustyline::Result<()> {
    let mut rl = DefaultEditor::new()?;
    let mut executor = Executor::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                println!("{}", parse_and_execute(&mut executor, line.as_str()));
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

fn parse_and_execute(executor: &mut Executor, str: &str) -> String {
    let buf = ParseBuffer::new(str).unwrap();
    let lp = wastparser::parse::<Line>(&buf);

    match lp {
        Ok(line) => match executor.execute(&line) {
            Ok(_) => {
                format!("{}", executor.to_state())
            }
            Err(err) => {
                format!("Error: {}", err.to_string())
            }
        },
        Err(err) => {
            format!("Error: {}", err.message())
        }
    }
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
        let mut executor = Executor::new();
        let resp = parse_and_execute(&mut executor, "(i32.const 1");
        assert_eq!(&resp[..7], "Error: ");
    }

    #[test]
    fn test_execute_error() {
        let mut executor = Executor::new();
        let resp = parse_and_execute(&mut executor, "(i32.add)");
        assert_eq!(&resp[..7], "Error: ");
    }
}

mod dict;
mod elements;
mod executor;
mod handler;
mod list;
mod locals;
mod model;
mod ops;
mod parser;
mod response;
mod stack;
mod value;

#[cfg(test)]
mod test_utils;

use executor::Executor;
use model::Line;
use parser::parse_line;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

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

fn parse_and_execute(executor: &mut Executor, line_str: &str) -> String {
    let buf = wast::parser::ParseBuffer::new(line_str).unwrap();
    match parse_line(&buf) {
        Ok(wast_line) => match Line::try_from(&wast_line) {
            Ok(line) => match executor.execute_line(line) {
                Ok(response) => response.message,
                Err(err) => {
                    format!("Error: {}", err.to_string())
                }
            },
            Err(err) => {
                format!("Error: {}", err.to_string())
            }
        },
        Err(err) => {
            format!("Error: {}", err.to_string())
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
    fn test_convert_error() {
        let mut executor = Executor::new();
        let resp = parse_and_execute(&mut executor, "(nop)");
        assert_eq!(&resp[..7], "Error: ");
    }

    #[test]
    fn test_execute_error() {
        let mut executor = Executor::new();
        let resp = parse_and_execute(&mut executor, "(i32.add)");
        assert_eq!(&resp[..7], "Error: ");
    }
}

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use wast::core::Expression;
use wast::parser::{ParseBuffer, self};


fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                parse_line(&line.as_str());
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

fn parse_line(str: &str) {
    let buf = ParseBuffer::new(str).unwrap();
    match parser::parse::<Expression>(&buf) {
        Ok(_) => println!("Ok"),
        Err(e) => {
            println!("Error: {}", e.message());
            return
        }
    };
}

#![allow(unused)]
use crate::model::Instruction;
use anyhow::Result;

#[derive(PartialEq, Debug)]
pub struct Chunk<'a> {
    commands: Vec<Command<'a>>,
}

#[derive(PartialEq, Debug)]
enum Command<'a> {
    Instr(&'a Instruction),
    If(Chunk<'a>, Chunk<'a>),
}

#[derive(PartialEq, Debug)]
enum ChunkEnd {
    None,
    Else,
    End,
}

pub fn preprocess(instrs: &Vec<Instruction>) -> Result<Chunk> {
    let (chunk, end) = chunk(instrs, &mut 0)?;
    if end != ChunkEnd::None {
        return Err(anyhow::anyhow!("Unexpected end of block"));
    }
    Ok(chunk)
}

fn chunk<'a>(instrs: &'a Vec<Instruction>, i: &mut usize) -> Result<(Chunk<'a>, ChunkEnd)> {
    let mut commands = Vec::new();
    let mut end = ChunkEnd::None;
    while *i < instrs.len() {
        let instr = &instrs[*i];
        match instr {
            Instruction::If => {
                *i += 1;
                let (if_chunk, if_end) = chunk(instrs, i)?;
                commands.push(match if_end {
                    ChunkEnd::Else => {
                        let (else_chunk, end) = chunk(instrs, i)?;
                        if end != ChunkEnd::End {
                            return Err(anyhow::anyhow!("Expected End"));
                        }
                        Command::If(if_chunk, else_chunk)
                    }
                    _ => {
                        let else_chunk = Chunk { commands: vec![] };
                        Command::If(if_chunk, else_chunk)
                    }
                });
            }
            Instruction::Else => {
                *i += 1;
                end = ChunkEnd::Else;
                break;
            }
            Instruction::End => {
                *i += 1;
                end = ChunkEnd::End;
                break;
            }
            _ => {
                *i += 1;
                commands.push(Command::Instr(instr));
            }
        }
    }
    Ok((Chunk { commands }, end))
}

#[cfg(test)]
mod tests {
    use crate::model::Instruction;
    use crate::preprocessor::{preprocess, Command};

    #[test]
    fn test_simple() {
        let instrs = vec![Instruction::I32Const(1), Instruction::I32Const(5)];

        let chunk = preprocess(&instrs).unwrap();
        assert_eq!(chunk.commands.len(), 2);
        assert_eq!(chunk.commands[0], Command::Instr(&Instruction::I32Const(1)));
        assert_eq!(chunk.commands[1], Command::Instr(&Instruction::I32Const(5)));
    }

    #[test]
    fn test_if_else() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::If,
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::End,
            Instruction::I32Const(5),
        ];

        let chunk = preprocess(&instrs).unwrap();
        assert_eq!(chunk.commands.len(), 3);
        assert_eq!(chunk.commands[0], Command::Instr(&Instruction::I32Const(1)));

        let (ifch, elsech) = match &chunk.commands[1] {
            Command::If(ifch, elsech) => (ifch, elsech),
            _ => panic!("Expected Command::If"),
        };

        assert!(ifch.commands.len() == 2);
        assert_eq!(ifch.commands[0], Command::Instr(&Instruction::I32Const(2)));
        assert_eq!(ifch.commands[1], Command::Instr(&Instruction::I32Const(3)));

        assert!(elsech.commands.len() == 1);
        assert_eq!(
            elsech.commands[0],
            Command::Instr(&Instruction::I32Const(4))
        );
    }

    #[test]
    fn test_only_if() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::If,
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::End,
            Instruction::I32Const(5),
        ];

        let chunk = preprocess(&instrs).unwrap();
        assert_eq!(chunk.commands.len(), 3);
        assert_eq!(chunk.commands[0], Command::Instr(&Instruction::I32Const(1)));

        let (ifch, elsech) = match &chunk.commands[1] {
            Command::If(ifch, elsech) => (ifch, elsech),
            _ => panic!("Expected Command::If"),
        };

        assert!(ifch.commands.len() == 2);
        assert_eq!(ifch.commands[0], Command::Instr(&Instruction::I32Const(2)));
        assert_eq!(ifch.commands[1], Command::Instr(&Instruction::I32Const(3)));

        assert!(elsech.commands.len() == 0);
    }

    #[test]
    fn test_only_end_error() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::I32Const(5),
            Instruction::End,
        ];

        assert!(preprocess(&instrs).is_err());
    }

    #[test]
    fn test_only_else_error() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::Else,
            Instruction::I32Const(5),
        ];

        assert!(preprocess(&instrs).is_err());
    }

    #[test]
    fn test_nested_end_error() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::If,
            Instruction::I32Const(2),
            Instruction::End,
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::End,
            Instruction::I32Const(5),
        ];

        assert!(preprocess(&instrs).is_err());
    }

    #[test]
    fn test_if_no_end_error() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::If,
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::I32Const(5),
        ];

        assert!(preprocess(&instrs).is_err());
    }
}

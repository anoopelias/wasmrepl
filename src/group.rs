#![allow(unused)]
use crate::model::Instruction;
use anyhow::Result;

#[derive(PartialEq, Debug, Clone)]
pub struct Group {
    pub commands: Vec<Command>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Command {
    Instruction(Instruction),
    If(Instruction, Group, Group),
}

#[derive(PartialEq, Debug)]
enum GroupEnd {
    None,
    Else,
    End,
}

pub fn preprocess(instrs: Vec<Instruction>) -> Result<Group> {
    let (group, end) = group(instrs, &mut 0)?;
    if end != GroupEnd::None {
        return Err(anyhow::anyhow!("Unexpected end of block"));
    }
    Ok(group)
}

fn group(mut instrs: Vec<Instruction>, i: &mut usize) -> Result<(Group, GroupEnd)> {
    let mut commands = Vec::new();
    let mut end = GroupEnd::None;
    instrs.drain(0..(*i));
    for instr in instrs.clone().into_iter() {
        match instr {
            Instruction::If => {
                *i += 1;
                let (if_group, if_end) = group(instrs.clone(), i)?;
                commands.push(match if_end {
                    GroupEnd::Else => {
                        let (else_group, end) = group(instrs.clone(), i)?;
                        if end != GroupEnd::End {
                            return Err(anyhow::anyhow!("Expected End"));
                        }
                        Command::If(instr, if_group, else_group)
                    }
                    _ => {
                        let else_group = Group { commands: vec![] };
                        Command::If(instr, if_group, else_group)
                    }
                });
            }
            Instruction::Else => {
                *i += 1;
                end = GroupEnd::Else;
                break;
            }
            Instruction::End => {
                *i += 1;
                end = GroupEnd::End;
                break;
            }
            _ => {
                *i += 1;
                commands.push(Command::Instruction(instr));
            }
        }
    }
    Ok((Group { commands }, end))
}

#[cfg(test)]
mod tests {
    use crate::group::{preprocess, Command};
    use crate::model::Instruction;

    #[test]
    fn test_simple() {
        let instrs = vec![Instruction::I32Const(1), Instruction::I32Const(5)];

        let group = preprocess(instrs).unwrap();
        assert_eq!(group.commands.len(), 2);
        assert_eq!(
            group.commands[0],
            Command::Instruction(Instruction::I32Const(1))
        );
        assert_eq!(
            group.commands[1],
            Command::Instruction(Instruction::I32Const(5))
        );
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

        let group = preprocess(instrs).unwrap();
        assert_eq!(group.commands.len(), 3);
        assert_eq!(
            group.commands[0],
            Command::Instruction(Instruction::I32Const(1))
        );

        let (ifch, elsech) = match &group.commands[1] {
            Command::If(_, ifch, elsech) => (ifch, elsech),
            _ => panic!("Expected Command::If"),
        };

        assert!(ifch.commands.len() == 2);
        assert_eq!(
            ifch.commands[0],
            Command::Instruction(Instruction::I32Const(2))
        );
        assert_eq!(
            ifch.commands[1],
            Command::Instruction(Instruction::I32Const(3))
        );

        assert!(elsech.commands.len() == 1);
        assert_eq!(
            elsech.commands[0],
            Command::Instruction(Instruction::I32Const(4))
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

        let group = preprocess(instrs).unwrap();
        assert_eq!(group.commands.len(), 3);
        assert_eq!(
            group.commands[0],
            Command::Instruction(Instruction::I32Const(1))
        );

        let (ifch, elsech) = match &group.commands[1] {
            Command::If(_, ifch, elsech) => (ifch, elsech),
            _ => panic!("Expected Command::If"),
        };

        assert!(ifch.commands.len() == 2);
        assert_eq!(
            ifch.commands[0],
            Command::Instruction(Instruction::I32Const(2))
        );
        assert_eq!(
            ifch.commands[1],
            Command::Instruction(Instruction::I32Const(3))
        );

        assert!(elsech.commands.len() == 0);
    }

    #[test]
    fn test_only_end_error() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::I32Const(5),
            Instruction::End,
        ];

        assert!(preprocess(instrs).is_err());
    }

    #[test]
    fn test_only_else_error() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::Else,
            Instruction::I32Const(5),
        ];

        assert!(preprocess(instrs).is_err());
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

        assert!(preprocess(instrs).is_err());
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

        assert!(preprocess(instrs).is_err());
    }
}

#![allow(unused)]
use crate::model::Instruction;
use anyhow::Result;

#[derive(PartialEq, Debug, Clone)]
pub struct Group {
    pub commands: Vec<Command>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Command {
    Instr(Instruction),
    If(Instruction, Group, Group),
}

#[derive(PartialEq, Debug)]
enum GroupEnd {
    None,
    Else,
    End,
}

pub fn preprocess(mut instrs: Vec<Instruction>) -> Result<Group> {
    instrs.reverse();
    let (group, end) = group(&mut instrs)?;
    if end != GroupEnd::None {
        return Err(anyhow::anyhow!("Unexpected end of block"));
    }
    Ok(group)
}

fn group(instrs: &mut Vec<Instruction>) -> Result<(Group, GroupEnd)> {
    let mut commands = Vec::new();
    while instrs.len() > 0 {
        let instr = instrs.pop().unwrap();
        commands.push(match instr {
            Instruction::If(_) => group_if(instrs, instr)?,
            Instruction::Else => {
                return Ok((Group { commands }, GroupEnd::Else));
            }
            Instruction::End => {
                return Ok((Group { commands }, GroupEnd::End));
            }
            _ => Command::Instr(instr),
        });
    }

    Ok((Group { commands }, GroupEnd::None))
}

fn group_if(instrs: &mut Vec<Instruction>, if_instr: Instruction) -> Result<Command> {
    let (if_group, if_end) = group(instrs)?;
    match if_end {
        GroupEnd::Else => {
            let (else_group, end) = group(instrs)?;
            if end != GroupEnd::End {
                return Err(anyhow::anyhow!("Expected End"));
            }
            Ok(Command::If(if_instr, if_group, else_group))
        }
        _ => {
            let else_group = Group { commands: vec![] };
            Ok(Command::If(if_instr, if_group, else_group))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::group::{preprocess, Command};
    use crate::model::{Instruction, ValType};
    use crate::test_utils::test_if;

    #[test]
    fn test_simple() {
        let instrs = vec![Instruction::I32Const(1), Instruction::I32Const(5)];

        let group = preprocess(instrs).unwrap();
        assert_eq!(group.commands.len(), 2);
        assert_eq!(group.commands[0], Command::Instr(Instruction::I32Const(1)));
        assert_eq!(group.commands[1], Command::Instr(Instruction::I32Const(5)));
    }

    #[test]
    fn test_if_else() {
        let instrs = vec![
            Instruction::I32Const(1),
            test_if!(()(ValType::I32)),
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::End,
            Instruction::I32Const(5),
        ];

        let group = preprocess(instrs).unwrap();
        assert_eq!(group.commands.len(), 3);
        assert_eq!(group.commands[0], Command::Instr(Instruction::I32Const(1)));

        let (ifch, elsech) = match &group.commands[1] {
            Command::If(_, ifch, elsech) => (ifch, elsech),
            _ => panic!("Expected Command::If"),
        };

        assert!(ifch.commands.len() == 2);
        assert_eq!(ifch.commands[0], Command::Instr(Instruction::I32Const(2)));
        assert_eq!(ifch.commands[1], Command::Instr(Instruction::I32Const(3)));

        assert!(elsech.commands.len() == 1);
        assert_eq!(elsech.commands[0], Command::Instr(Instruction::I32Const(4)));
    }

    #[test]
    fn test_only_if() {
        let instrs = vec![
            Instruction::I32Const(1),
            test_if!(()(ValType::I32, ValType::I32)),
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::End,
            Instruction::I32Const(5),
        ];

        let group = preprocess(instrs).unwrap();
        assert_eq!(group.commands.len(), 3);
        assert_eq!(group.commands[0], Command::Instr(Instruction::I32Const(1)));

        let (ifch, elsech) = match &group.commands[1] {
            Command::If(_, ifch, elsech) => (ifch, elsech),
            _ => panic!("Expected Command::If"),
        };

        assert!(ifch.commands.len() == 2);
        assert_eq!(ifch.commands[0], Command::Instr(Instruction::I32Const(2)));
        assert_eq!(ifch.commands[1], Command::Instr(Instruction::I32Const(3)));

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
            test_if!(()(ValType::I32)),
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
            test_if!(()(ValType::I32)),
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::I32Const(5),
        ];

        assert!(preprocess(instrs).is_err());
    }
}

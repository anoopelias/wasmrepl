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
    Block(Instruction, Group),
}

#[derive(PartialEq, Debug)]
enum GroupClose {
    None,
    Else,
    End,
}

pub fn group(mut instrs: Vec<Instruction>) -> Result<Group> {
    // We are reversing the vector first and then popping from the end
    // This is beacuse it is easier to pop from the end than the start
    instrs.reverse();

    let (group, end) = group_rec(&mut instrs)?;
    if end != GroupClose::None {
        return Err(anyhow::anyhow!("Unexpected end of block"));
    }
    Ok(group)
}

fn group_rec(instrs: &mut Vec<Instruction>) -> Result<(Group, GroupClose)> {
    let mut commands = Vec::new();
    while instrs.len() > 0 {
        let instr = instrs.pop().unwrap();
        commands.push(match instr {
            Instruction::If(_) => group_if(instrs, instr)?,
            Instruction::Block(_) => group_block(instrs, instr)?,
            Instruction::Else => return close_else(commands),
            Instruction::End => return close_end(commands),
            _ => Command::Instr(instr),
        });
    }

    Ok((Group { commands }, GroupClose::None))
}

fn close_else(commands: Vec<Command>) -> Result<(Group, GroupClose)> {
    Ok((Group { commands }, GroupClose::Else))
}

fn close_end(commands: Vec<Command>) -> Result<(Group, GroupClose)> {
    Ok((Group { commands }, GroupClose::End))
}

fn group_if(instrs: &mut Vec<Instruction>, if_instr: Instruction) -> Result<Command> {
    let (if_group, if_end) = group_rec(instrs)?;
    match if_end {
        GroupClose::Else => {
            let (else_group, end) = group_rec(instrs)?;
            if end != GroupClose::End {
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

fn group_block(instrs: &mut Vec<Instruction>, block_instr: Instruction) -> Result<Command> {
    let (block_group, end) = group_rec(instrs)?;
    if end != GroupClose::End {
        return Err(anyhow::anyhow!("Expected End"));
    }
    Ok(Command::Block(block_instr, block_group))
}

#[cfg(test)]
mod tests {
    use crate::group::{group, Command};
    use crate::model::{Instruction, ValType};
    use crate::test_utils::{test_block, test_if};

    #[test]
    fn test_simple() {
        let instrs = vec![Instruction::I32Const(1), Instruction::I32Const(5)];

        let group = group(instrs).unwrap();
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

        let group = group(instrs).unwrap();
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

        let group = group(instrs).unwrap();
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

        assert!(group(instrs).is_err());
    }

    #[test]
    fn test_only_else_error() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::Else,
            Instruction::I32Const(5),
        ];

        assert!(group(instrs).is_err());
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

        assert!(group(instrs).is_err());
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

        assert!(group(instrs).is_err());
    }

    #[test]
    fn test_block() {
        let instrs = vec![
            Instruction::I32Const(1),
            test_block!(()(ValType::I32)),
            Instruction::I32Const(2),
            Instruction::End,
            Instruction::I32Const(3),
        ];

        let group = group(instrs).unwrap();
        assert_eq!(group.commands.len(), 3);
        assert_eq!(group.commands[0], Command::Instr(Instruction::I32Const(1)));

        let (block_instr, block_group) = match &group.commands[1] {
            Command::Block(block_instr, block_group) => (block_instr, block_group),
            _ => panic!("Expected Command::Block"),
        };

        assert_eq!(block_group.commands.len(), 1);
        assert_eq!(
            block_group.commands[0],
            Command::Instr(Instruction::I32Const(2))
        );
    }

    #[test]
    fn test_block_else_error() {
        let instrs = vec![
            Instruction::I32Const(1),
            test_block!(()(ValType::I32)),
            Instruction::I32Const(2),
            Instruction::Else,
            Instruction::I32Const(3),
        ];

        assert!(group(instrs).is_err());
    }
}

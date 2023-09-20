// An instructions analyzer, possibly an optimizer in the future.

use crate::model::Instruction;

pub fn skip_block(instrs: &Vec<Instruction>, start: usize) -> usize {
    let mut depth = 1;
    let mut i = start + 1;
    while depth > 0 && i < instrs.len() {
        match instrs[i] {
            Instruction::If => depth += 1,
            Instruction::Else => {
                if depth == 1 {
                    depth -= 1
                }
            }
            Instruction::End => depth -= 1,
            _ => (),
        }
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use crate::{analyzer::skip_block, model::Instruction};

    #[test]
    fn test_skip_block() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::If,
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::I32Const(5),
            Instruction::I32Const(6),
            Instruction::End,
            Instruction::I32Const(7),
        ];

        assert_eq!(skip_block(&instrs, 1), 5);
        assert_eq!(skip_block(&instrs, 4), 9);
    }

    #[test]
    fn test_skip_nested_if_block() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::If,
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::If,
            Instruction::I32Const(5),
            Instruction::I32Const(6),
            Instruction::Else,
            Instruction::I32Const(7),
            Instruction::I32Const(8),
            Instruction::End,
            Instruction::I32Const(9),
            Instruction::End,
            Instruction::I32Const(10),
        ];

        assert_eq!(skip_block(&instrs, 1), 5);
        assert_eq!(skip_block(&instrs, 4), 15);
    }

    #[test]
    fn test_skip_if_without_else() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::If,
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::I32Const(4),
            Instruction::End,
            Instruction::I32Const(5),
        ];

        assert_eq!(skip_block(&instrs, 1), 6);
        assert_eq!(skip_block(&instrs, 5), 7);
    }
}

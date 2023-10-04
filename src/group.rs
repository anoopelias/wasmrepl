#![allow(unused)]
use crate::model::{Expression, Instruction};
use anyhow::Result;

#[derive(PartialEq, Debug)]
enum ExprEnd {
    None,
    Else,
    End,
}

pub fn group_expr(mut instrs: Vec<Instruction>) -> Result<Expression> {
    // We are reversing the vector first and then popping from the end
    // This is beacuse it is easier to pop from the end than the start
    instrs.reverse();

    let (expr, end) = expr(&mut instrs)?;
    if end != ExprEnd::None {
        return Err(anyhow::anyhow!("Unexpected end of block"));
    }
    Ok(expr)
}

fn expr(instrs: &mut Vec<Instruction>) -> Result<(Expression, ExprEnd)> {
    let mut new_instrs = Vec::new();
    while instrs.len() > 0 {
        let instr = instrs.pop().unwrap();
        new_instrs.push(match instr {
            Instruction::If(block_type, if_expr, else_expr) => {
                let (if_ex, else_ex) = expr_if(instrs)?;
                // TODO: Can we mutate the existing object instead?
                Instruction::If(block_type, Some(if_ex), Some(else_ex))
            }
            Instruction::Block(block_type, mut expr) => {
                Instruction::Block(block_type, Some(expr_block(instrs)?))
            }
            Instruction::Else => return close_else(new_instrs),
            Instruction::End => return close_end(new_instrs),
            _ => instr,
        });
    }

    Ok((Expression { instrs: new_instrs }, ExprEnd::None))
}

fn expr_if(instrs: &mut Vec<Instruction>) -> Result<(Expression, Expression)> {
    let (if_group, if_end) = expr(instrs)?;
    match if_end {
        ExprEnd::Else => {
            let (else_group, end) = expr(instrs)?;
            if end != ExprEnd::End {
                return Err(anyhow::anyhow!("Expected End"));
            }
            Ok((if_group, else_group))
        }
        _ => {
            let else_group = Expression { instrs: vec![] };
            Ok((if_group, else_group))
        }
    }
}

fn expr_block(instrs: &mut Vec<Instruction>) -> Result<Expression> {
    let (block_group, end) = expr(instrs)?;
    if end != ExprEnd::End {
        return Err(anyhow::anyhow!("Expected End"));
    }
    Ok(block_group)
}

fn close_else(instrs: Vec<Instruction>) -> Result<(Expression, ExprEnd)> {
    Ok((Expression { instrs }, ExprEnd::Else))
}

fn close_end(instrs: Vec<Instruction>) -> Result<(Expression, ExprEnd)> {
    Ok((Expression { instrs }, ExprEnd::End))
}

#[cfg(test)]
mod tests {
    use crate::group::group_expr;
    use crate::model::{Expression, Instruction, Local, ValType};
    use crate::test_utils::{test_block, test_block_type, test_if, test_local};

    #[test]
    fn test_simple() {
        let instrs = vec![Instruction::I32Const(1), Instruction::I32Const(5)];

        let expr = group_expr(instrs).unwrap();
        assert_eq!(expr.instrs.len(), 2);
        assert_eq!(expr.instrs[0], Instruction::I32Const(1));
        assert_eq!(expr.instrs[1], Instruction::I32Const(5));
    }

    #[test]
    fn test_if_else() {
        let block_type = test_block_type!((test_local!(ValType::I32)), (ValType::I32));
        let instrs = vec![
            Instruction::I32Const(1),
            test_if!((block_type)),
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::End,
            Instruction::I32Const(5),
        ];

        let expr = group_expr(instrs).unwrap();
        assert_eq!(expr.instrs.len(), 3);
        assert_eq!(expr.instrs[0], Instruction::I32Const(1));

        let (if_expr, else_expr) = match &expr.instrs[1] {
            Instruction::If(_, Some(if_expr), Some(else_expr)) => (if_expr, else_expr),
            _ => panic!("Expected Instruction::If"),
        };

        assert!(if_expr.instrs.len() == 2);
        assert_eq!(if_expr.instrs[0], Instruction::I32Const(2));
        assert_eq!(if_expr.instrs[1], Instruction::I32Const(3));

        assert!(else_expr.instrs.len() == 1);
        assert_eq!(else_expr.instrs[0], Instruction::I32Const(4));
    }

    #[test]
    fn test_only_if() {
        let block_type = test_block_type!((test_local!(ValType::I32)), (ValType::I32));
        let instrs = vec![
            Instruction::I32Const(1),
            test_if!((block_type)),
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::End,
            Instruction::I32Const(5),
        ];

        let expr = group_expr(instrs).unwrap();
        assert_eq!(expr.instrs.len(), 3);
        assert_eq!(expr.instrs[0], Instruction::I32Const(1));

        let (if_expr, else_expr) = match &expr.instrs[1] {
            Instruction::If(_, Some(if_expr), Some(else_expr)) => (if_expr, else_expr),
            _ => panic!("Expected Instruction::If"),
        };

        assert!(if_expr.instrs.len() == 2);
        assert_eq!(if_expr.instrs[0], Instruction::I32Const(2));
        assert_eq!(if_expr.instrs[1], Instruction::I32Const(3));

        assert!(else_expr.instrs.len() == 0);
    }

    #[test]
    fn test_only_end_error() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::I32Const(5),
            Instruction::End,
        ];

        assert!(group_expr(instrs).is_err());
    }

    #[test]
    fn test_only_else_error() {
        let instrs = vec![
            Instruction::I32Const(1),
            Instruction::Else,
            Instruction::I32Const(5),
        ];

        assert!(group_expr(instrs).is_err());
    }

    #[test]
    fn test_nested_end_error() {
        let block_type = test_block_type!((), (ValType::I32));
        let instrs = vec![
            Instruction::I32Const(1),
            test_if!((block_type)),
            Instruction::I32Const(2),
            Instruction::End,
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::End,
            Instruction::I32Const(5),
        ];

        assert!(group_expr(instrs).is_err());
    }

    #[test]
    fn test_if_no_end_error() {
        let block_type = test_block_type!((), (ValType::I32));
        let instrs = vec![
            Instruction::I32Const(1),
            test_if!((block_type)),
            Instruction::I32Const(2),
            Instruction::I32Const(3),
            Instruction::Else,
            Instruction::I32Const(4),
            Instruction::I32Const(5),
        ];

        assert!(group_expr(instrs).is_err());
    }

    #[test]
    fn test_block() {
        let block_type = test_block_type!((), (ValType::I32));
        let instrs = vec![
            Instruction::I32Const(1),
            test_block!(block_type),
            Instruction::I32Const(2),
            Instruction::End,
            Instruction::I32Const(3),
        ];

        let expr = group_expr(instrs).unwrap();
        assert_eq!(expr.instrs.len(), 3);
        assert_eq!(expr.instrs[0], Instruction::I32Const(1));

        let block = match &expr.instrs[1] {
            Instruction::Block(_, Some(block)) => block,
            _ => panic!("Expected Instruction::Block"),
        };

        assert_eq!(block.instrs.len(), 1);
        assert_eq!(block.instrs[0], Instruction::I32Const(2));
    }

    #[test]
    fn test_block_else_error() {
        let block_type = test_block_type!((), (ValType::I32));
        let instrs = vec![
            Instruction::I32Const(1),
            test_block!(block_type),
            Instruction::I32Const(2),
            Instruction::Else,
            Instruction::I32Const(3),
        ];

        assert!(group_expr(instrs).is_err());
    }
}

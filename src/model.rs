// We need to convert wast object to our own model objects prior to processing.
// This is because wast objects have lifetime of `&'a` and this cannot go past
// the string it is parsing. While for example our `func` has to live past the
// string it is parsing for later execution.
use wast::{
    core::{
        Expression as WastExpression, Func as WastFunc, Instruction as WastInstruction,
        Local as WastLocal, ValType as WastValType,
    },
    token::{Id, Index as WastIndex},
};

use anyhow::{Error, Result};

use crate::parser::{Line as WastLine, LineExpression as WastLineExpression};

pub enum Line {
    Expression(LineExpression),
    Func(Func),
}

impl TryFrom<&WastLine<'_>> for Line {
    type Error = Error;
    fn try_from(line: &WastLine) -> Result<Self> {
        match line {
            WastLine::Expression(line_expr) => Ok(Line::Expression(line_expr.try_into()?)),
            WastLine::Func(func) => Ok(Line::Func(func.try_into()?)),
        }
    }
}

pub struct LineExpression {
    pub locals: Vec<Local>,
    pub expr: Expression,
}

impl TryFrom<&WastLineExpression<'_>> for LineExpression {
    type Error = Error;
    fn try_from(line_expr: &WastLineExpression) -> Result<Self> {
        let mut locals = Vec::new();
        for local in line_expr.locals.iter() {
            locals.push(local.try_into()?);
        }

        let expr: Expression = (&line_expr.expr).try_into()?;
        Ok(LineExpression { locals, expr })
    }
}

pub struct Func {
    pub id: Option<String>,
}

impl TryFrom<&WastFunc<'_>> for Func {
    type Error = Error;
    fn try_from(func: &WastFunc) -> Result<Self> {
        let id = from_id(func.id);
        Ok(Func { id })
    }
}

pub struct Local {
    pub id: Option<String>,
    pub val_type: ValType,
}

impl TryFrom<&WastLocal<'_>> for Local {
    type Error = Error;
    fn try_from(local: &WastLocal) -> Result<Self> {
        let id = from_id(local.id);
        let val_type: ValType = (&local.ty).try_into()?;
        Ok(Local { id, val_type })
    }
}

#[derive(PartialEq, Debug)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
}

impl TryFrom<&WastValType<'_>> for ValType {
    type Error = Error;
    fn try_from(val_type: &WastValType) -> Result<Self> {
        match val_type {
            WastValType::I32 => Ok(ValType::I32),
            WastValType::I64 => Ok(ValType::I64),
            WastValType::F32 => Ok(ValType::F32),
            WastValType::F64 => Ok(ValType::F64),
            _ => Err(Error::msg("Unsupported value type")),
        }
    }
}

pub struct Expression {
    pub instrs: Vec<Instruction>,
}

impl TryFrom<&WastExpression<'_>> for Expression {
    type Error = Error;
    fn try_from(expr: &WastExpression) -> Result<Self> {
        let mut instrs = Vec::new();

        for instr in expr.instrs.iter() {
            instrs.push(instr.try_into()?);
        }
        Ok(Expression { instrs })
    }
}

#[derive(PartialEq, Debug)]
pub enum Index {
    Id(String),
    Num(u32),
}

impl TryFrom<&WastIndex<'_>> for Index {
    type Error = Error;
    fn try_from(index: &WastIndex) -> Result<Self> {
        match index {
            WastIndex::Id(id) => Ok(Index::Id(id.name().to_string())),
            WastIndex::Num(num, _) => Ok(Index::Num(*num)),
        }
    }
}

fn from_id(id: Option<Id>) -> Option<String> {
    id.map(|id| id.name().to_string())
}

#[derive(PartialEq, Debug)]
pub enum Instruction {
    Drop,
    I32Const(i32),
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    I64Const(i64),
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,
    F32Const(f32),
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F64Const(f64),
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,
    LocalGet(Index),
    LocalSet(Index),
    LocalTee(Index),
}

impl TryFrom<&WastInstruction<'_>> for Instruction {
    type Error = Error;
    fn try_from(instruction: &WastInstruction) -> Result<Self> {
        match instruction {
            WastInstruction::Drop => Ok(Instruction::Drop),
            WastInstruction::I32Const(i) => Ok(Instruction::I32Const(*i)),
            WastInstruction::I32Clz => Ok(Instruction::I32Clz),
            WastInstruction::I32Ctz => Ok(Instruction::I32Ctz),
            WastInstruction::I32Popcnt => Ok(Instruction::I32Popcnt),
            WastInstruction::I32Add => Ok(Instruction::I32Add),
            WastInstruction::I32Sub => Ok(Instruction::I32Sub),
            WastInstruction::I32Mul => Ok(Instruction::I32Mul),
            WastInstruction::I32DivS => Ok(Instruction::I32DivS),
            WastInstruction::I32DivU => Ok(Instruction::I32DivU),
            WastInstruction::I32RemS => Ok(Instruction::I32RemS),
            WastInstruction::I32RemU => Ok(Instruction::I32RemU),
            WastInstruction::I32And => Ok(Instruction::I32And),
            WastInstruction::I32Or => Ok(Instruction::I32Or),
            WastInstruction::I32Xor => Ok(Instruction::I32Xor),
            WastInstruction::I32Shl => Ok(Instruction::I32Shl),
            WastInstruction::I32ShrS => Ok(Instruction::I32ShrS),
            WastInstruction::I32ShrU => Ok(Instruction::I32ShrU),
            WastInstruction::I32Rotl => Ok(Instruction::I32Rotl),
            WastInstruction::I32Rotr => Ok(Instruction::I32Rotr),
            WastInstruction::I64Const(i) => Ok(Instruction::I64Const(*i)),
            WastInstruction::I64Clz => Ok(Instruction::I64Clz),
            WastInstruction::I64Ctz => Ok(Instruction::I64Ctz),
            WastInstruction::I64Popcnt => Ok(Instruction::I64Popcnt),
            WastInstruction::I64Add => Ok(Instruction::I64Add),
            WastInstruction::I64Sub => Ok(Instruction::I64Sub),
            WastInstruction::I64Mul => Ok(Instruction::I64Mul),
            WastInstruction::I64DivS => Ok(Instruction::I64DivS),
            WastInstruction::I64DivU => Ok(Instruction::I64DivU),
            WastInstruction::I64RemS => Ok(Instruction::I64RemS),
            WastInstruction::I64RemU => Ok(Instruction::I64RemU),
            WastInstruction::I64And => Ok(Instruction::I64And),
            WastInstruction::I64Or => Ok(Instruction::I64Or),
            WastInstruction::I64Xor => Ok(Instruction::I64Xor),
            WastInstruction::I64Shl => Ok(Instruction::I64Shl),
            WastInstruction::I64ShrS => Ok(Instruction::I64ShrS),
            WastInstruction::I64ShrU => Ok(Instruction::I64ShrU),
            WastInstruction::I64Rotl => Ok(Instruction::I64Rotl),
            WastInstruction::I64Rotr => Ok(Instruction::I64Rotr),
            WastInstruction::F32Const(f) => Ok(Instruction::F32Const(f32::from_bits(f.bits))),
            WastInstruction::F32Abs => Ok(Instruction::F32Abs),
            WastInstruction::F32Neg => Ok(Instruction::F32Neg),
            WastInstruction::F32Ceil => Ok(Instruction::F32Ceil),
            WastInstruction::F32Floor => Ok(Instruction::F32Floor),
            WastInstruction::F32Trunc => Ok(Instruction::F32Trunc),
            WastInstruction::F32Nearest => Ok(Instruction::F32Nearest),
            WastInstruction::F32Sqrt => Ok(Instruction::F32Sqrt),
            WastInstruction::F32Add => Ok(Instruction::F32Add),
            WastInstruction::F32Sub => Ok(Instruction::F32Sub),
            WastInstruction::F32Mul => Ok(Instruction::F32Mul),
            WastInstruction::F32Div => Ok(Instruction::F32Div),
            WastInstruction::F32Min => Ok(Instruction::F32Min),
            WastInstruction::F32Max => Ok(Instruction::F32Max),
            WastInstruction::F32Copysign => Ok(Instruction::F32Copysign),
            WastInstruction::F64Const(f) => Ok(Instruction::F64Const(f64::from_bits(f.bits))),
            WastInstruction::F64Abs => Ok(Instruction::F64Abs),
            WastInstruction::F64Neg => Ok(Instruction::F64Neg),
            WastInstruction::F64Ceil => Ok(Instruction::F64Ceil),
            WastInstruction::F64Floor => Ok(Instruction::F64Floor),
            WastInstruction::F64Trunc => Ok(Instruction::F64Trunc),
            WastInstruction::F64Nearest => Ok(Instruction::F64Nearest),
            WastInstruction::F64Sqrt => Ok(Instruction::F64Sqrt),
            WastInstruction::F64Add => Ok(Instruction::F64Add),
            WastInstruction::F64Sub => Ok(Instruction::F64Sub),
            WastInstruction::F64Mul => Ok(Instruction::F64Mul),
            WastInstruction::F64Div => Ok(Instruction::F64Div),
            WastInstruction::F64Min => Ok(Instruction::F64Min),
            WastInstruction::F64Max => Ok(Instruction::F64Max),
            WastInstruction::F64Copysign => Ok(Instruction::F64Copysign),
            WastInstruction::LocalGet(index) => Ok(Instruction::LocalGet(index.try_into()?)),
            WastInstruction::LocalSet(index) => Ok(Instruction::LocalSet(index.try_into()?)),
            WastInstruction::LocalTee(index) => Ok(Instruction::LocalTee(index.try_into()?)),
            _ => Err(Error::msg("Unsupported instruction")),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{
        model::{Expression, Func, Index, Instruction, Line, LineExpression, Local, ValType},
        parser::{Line as WastLine, LineExpression as WastLineExpression},
        test_utils::{float32_for, float64_for},
    };
    use wast::{
        core::{
            Expression as WastExpression, Func as WastFunc, InlineExport,
            Instruction as WastInstruction, Local as WastLocal, TypeUse, ValType as WastValType,
        },
        parser::{self, ParseBuffer},
        token::{Id, Index as WastIndex, Span},
    };

    fn test_new_local_i32<'a>() -> WastLocal<'a> {
        WastLocal {
            id: None,
            name: None,
            ty: WastValType::I32,
        }
    }

    #[test]
    fn test_from_wast_instruction() {
        let instr = Instruction::try_from(&WastInstruction::I32Const(2)).unwrap();
        assert_eq!(instr, Instruction::I32Const(2));
    }

    #[test]
    fn test_from_wast_instruction_f32_const() {
        let str_f32 = String::from("3.14");
        let buf_f32 = ParseBuffer::new(&str_f32).unwrap();

        let instr =
            Instruction::try_from(&WastInstruction::F32Const(float32_for(&buf_f32))).unwrap();
        assert_eq!(instr, Instruction::F32Const(3.14));
    }

    #[test]
    fn test_from_wast_instruction_f64_const() {
        let str_f64 = String::from("3.14");
        let buf_f64 = ParseBuffer::new(&str_f64).unwrap();

        let instr =
            Instruction::try_from(&WastInstruction::F64Const(float64_for(&buf_f64))).unwrap();
        assert_eq!(instr, Instruction::F64Const(3.14));
    }

    #[test]
    fn test_from_wast_instruction_local_get() {
        let instr = Instruction::try_from(&WastInstruction::LocalGet(WastIndex::Num(
            1,
            Span::from_offset(0),
        )))
        .unwrap();
        assert_eq!(instr, Instruction::LocalGet(Index::Num(1)));
    }

    #[test]
    fn test_from_wast_instruction_error() {
        let instr = Instruction::try_from(&WastInstruction::Nop);
        assert!(instr.is_err());
    }

    #[test]
    fn test_from_wast_expression() {
        let expr = Expression::try_from(&WastExpression {
            instrs: Box::new([WastInstruction::I32Const(2)]),
        })
        .unwrap();

        assert_eq!(expr.instrs.len(), 1);
        assert_eq!(expr.instrs[0], Instruction::I32Const(2));
    }

    #[test]
    fn test_from_val_type() {
        let val_type = ValType::try_from(&WastValType::I64).unwrap();
        assert_eq!(val_type, ValType::I64);
    }

    #[test]
    fn test_from_val_type_error() {
        assert!(ValType::try_from(&WastValType::V128).is_err());
    }

    #[test]
    fn test_from_wast_local() {
        let local = Local::try_from(&test_new_local_i32()).unwrap();
        assert_eq!(local.id, None);
        assert_eq!(local.val_type, crate::model::ValType::I32);
    }

    #[test]
    fn test_from_wast_index() {
        let index = Index::try_from(&WastIndex::Num(1, Span::from_offset(0))).unwrap();
        assert_eq!(index, Index::Num(1));
    }

    #[test]
    fn test_from_wast_index_id() {
        let str_id = String::from("$id1");
        let buf_id = ParseBuffer::new(&str_id).unwrap();
        let id = parser::parse::<Id>(&buf_id).unwrap();

        let index = Index::try_from(&WastIndex::Id(id)).unwrap();
        assert_eq!(index, Index::Id(String::from("id1")));
    }

    #[test]
    fn test_from_wast_func() {
        let str_id = String::from("$fun1");
        let buf_id = ParseBuffer::new(&str_id).unwrap();
        let id = parser::parse::<Id>(&buf_id).unwrap();
        let index = WastIndex::Id(id);

        let func = Func::try_from(&WastFunc {
            id: Some(id),
            name: None,
            exports: InlineExport { names: vec![] },
            ty: TypeUse::new_with_index(index),
            span: Span::from_offset(0),
            kind: wast::core::FuncKind::Inline {
                locals: Box::new([]),
                expression: WastExpression {
                    instrs: Box::new([]),
                },
            },
        })
        .unwrap();

        assert_eq!(func.id, Some(String::from("fun1")));
    }

    #[test]
    fn test_from_wast_line_expression() {
        let line_expr = LineExpression::try_from(&WastLineExpression {
            locals: vec![test_new_local_i32()],
            expr: WastExpression {
                instrs: Box::new([WastInstruction::I32Const(2)]),
            },
        })
        .unwrap();

        assert_eq!(line_expr.locals.len(), 1);
        assert_eq!(line_expr.locals[0].val_type, ValType::I32);
        assert_eq!(line_expr.expr.instrs.len(), 1);
        assert_eq!(line_expr.expr.instrs[0], Instruction::I32Const(2));
    }

    #[test]
    fn test_from_wast_line_for_line_expression() {
        let line_expression = Line::try_from(&WastLine::Expression(WastLineExpression {
            locals: vec![test_new_local_i32()],
            expr: WastExpression {
                instrs: Box::new([WastInstruction::I32Const(2)]),
            },
        }))
        .unwrap();

        if let Line::Expression(line_expr) = line_expression {
            assert_eq!(line_expr.locals.len(), 1);
            assert_eq!(line_expr.locals[0].val_type, ValType::I32);
            assert_eq!(line_expr.expr.instrs.len(), 1);
            assert_eq!(line_expr.expr.instrs[0], Instruction::I32Const(2));
        } else {
            panic!("Expected Line::Expression");
        }
    }

    #[test]
    fn test_from_wast_line_for_func() {
        let str_id = String::from("$fun1");
        let buf_id = ParseBuffer::new(&str_id).unwrap();
        let id = parser::parse::<Id>(&buf_id).unwrap();
        let index = WastIndex::Id(id);

        let line_func = Line::try_from(&WastLine::Func(WastFunc {
            id: Some(id),
            name: None,
            exports: InlineExport { names: vec![] },
            ty: TypeUse::new_with_index(index),
            span: Span::from_offset(0),
            kind: wast::core::FuncKind::Inline {
                locals: Box::new([]),
                expression: WastExpression {
                    instrs: Box::new([]),
                },
            },
        }))
        .unwrap();

        if let Line::Func(func) = line_func {
            assert_eq!(func.id, Some(String::from("fun1")));
        } else {
            panic!("Expected Line::Func");
        }
    }
}

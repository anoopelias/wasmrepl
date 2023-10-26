// We need to convert wast object to our own model objects prior to processing.
// This is because wast objects have lifetime of `&'a` and this cannot go past
// the string it is parsing. While for example our `func` has to live past the
// string it is parsing for later execution.
//
use wast::{
    core::{
        BlockType as WastBlockType, Expression as WastExpression, Func as WastFunc, FuncKind,
        FunctionType, Instruction as WastInstruction, Local as WastLocal, TypeUse,
        ValType as WastValType,
    },
    token::{Id, Index as WastIndex},
};

use anyhow::{Error, Result};

use crate::{
    group::group_expr,
    parser::{Line as WastLine, LineExpression as WastLineExpression},
};

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

#[derive(Clone)]
pub struct Func {
    pub id: Option<String>,
    pub ty: FuncType,
    pub line_expression: LineExpression,
}

impl TryFrom<&WastFunc<'_>> for Func {
    type Error = Error;
    fn try_from(func: &WastFunc) -> Result<Self> {
        let id = from_id(func.id);
        let ty = FuncType::try_from(&func.ty)?;

        if !func.exports.names.is_empty() {
            return Err(Error::msg("Unsupported export"));
        }

        let line_expression = match &func.kind {
            FuncKind::Inline { locals, expression } => {
                let mut lcls = Vec::new();

                for local in locals.iter() {
                    lcls.push(local.try_into()?);
                }

                LineExpression {
                    locals: lcls,
                    expr: expression.try_into()?,
                }
            }
            _ => {
                return Err(Error::msg("Unsupported function kind"));
            }
        };

        Ok(Func {
            id,
            ty,
            line_expression,
        })
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct FuncType {
    pub params: Vec<Local>,
    pub results: Vec<ValType>,
}

impl TryFrom<&TypeUse<'_, FunctionType<'_>>> for FuncType {
    type Error = Error;
    fn try_from(type_use: &TypeUse<'_, FunctionType<'_>>) -> Result<Self> {
        let mut params = Vec::new();
        let mut results = Vec::new();

        if type_use.index.is_some() {
            return Err(Error::msg("Unsupported type index"));
        }

        match &type_use.inline {
            Some(func_type) => {
                for param in func_type.params.iter() {
                    params.push(Local {
                        id: from_id(param.0),
                        val_type: (&param.2).try_into()?,
                    });
                }

                for result in func_type.results.iter() {
                    results.push(result.try_into()?);
                }
                Ok(FuncType { params, results })
            }
            None => Ok(FuncType {
                params: vec![],
                results: vec![],
            }),
        }
    }
}

#[derive(Clone)]
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

#[derive(PartialEq, Clone, Debug)]
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

#[derive(PartialEq, Debug, Clone)]
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

#[derive(PartialEq, Debug, Clone)]
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
        group_expr(instrs)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct BlockType {
    pub label: Option<String>,
    pub ty: FuncType,
}

impl TryFrom<&Box<WastBlockType<'_>>> for BlockType {
    type Error = Error;
    fn try_from(block_type: &Box<WastBlockType<'_>>) -> Result<Self> {
        let label = from_id(block_type.label);
        let ty = FuncType::try_from(&block_type.ty)?;

        Ok(BlockType { label, ty })
    }
}

#[derive(PartialEq, Debug, Clone)]
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

#[derive(PartialEq, Debug, Clone)]
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
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
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
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
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
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
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
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    LocalGet(Index),
    LocalSet(Index),
    LocalTee(Index),
    Call(Index),
    Return,
    Nop,
    If(BlockType, Option<Expression>, Option<Expression>),
    Else,
    End,
    Block(BlockType, Option<Expression>),
    Br(Index),
    Loop(BlockType, Option<Expression>),
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
            WastInstruction::I32Eqz => Ok(Instruction::I32Eqz),
            WastInstruction::I32Eq => Ok(Instruction::I32Eq),
            WastInstruction::I32Ne => Ok(Instruction::I32Ne),
            WastInstruction::I32LtS => Ok(Instruction::I32LtS),
            WastInstruction::I32LtU => Ok(Instruction::I32LtU),
            WastInstruction::I32GtS => Ok(Instruction::I32GtS),
            WastInstruction::I32GtU => Ok(Instruction::I32GtU),
            WastInstruction::I32LeS => Ok(Instruction::I32LeS),
            WastInstruction::I32LeU => Ok(Instruction::I32LeU),
            WastInstruction::I32GeS => Ok(Instruction::I32GeS),
            WastInstruction::I32GeU => Ok(Instruction::I32GeU),
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
            WastInstruction::I64Eqz => Ok(Instruction::I64Eqz),
            WastInstruction::I64Eq => Ok(Instruction::I64Eq),
            WastInstruction::I64Ne => Ok(Instruction::I64Ne),
            WastInstruction::I64LtS => Ok(Instruction::I64LtS),
            WastInstruction::I64LtU => Ok(Instruction::I64LtU),
            WastInstruction::I64GtS => Ok(Instruction::I64GtS),
            WastInstruction::I64GtU => Ok(Instruction::I64GtU),
            WastInstruction::I64LeS => Ok(Instruction::I64LeS),
            WastInstruction::I64LeU => Ok(Instruction::I64LeU),
            WastInstruction::I64GeS => Ok(Instruction::I64GeS),
            WastInstruction::I64GeU => Ok(Instruction::I64GeU),
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
            WastInstruction::F32Eq => Ok(Instruction::F32Eq),
            WastInstruction::F32Ne => Ok(Instruction::F32Ne),
            WastInstruction::F32Lt => Ok(Instruction::F32Lt),
            WastInstruction::F32Gt => Ok(Instruction::F32Gt),
            WastInstruction::F32Le => Ok(Instruction::F32Le),
            WastInstruction::F32Ge => Ok(Instruction::F32Ge),
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
            WastInstruction::F64Eq => Ok(Instruction::F64Eq),
            WastInstruction::F64Ne => Ok(Instruction::F64Ne),
            WastInstruction::F64Lt => Ok(Instruction::F64Lt),
            WastInstruction::F64Gt => Ok(Instruction::F64Gt),
            WastInstruction::F64Le => Ok(Instruction::F64Le),
            WastInstruction::F64Ge => Ok(Instruction::F64Ge),
            WastInstruction::LocalGet(index) => Ok(Instruction::LocalGet(index.try_into()?)),
            WastInstruction::LocalSet(index) => Ok(Instruction::LocalSet(index.try_into()?)),
            WastInstruction::LocalTee(index) => Ok(Instruction::LocalTee(index.try_into()?)),
            WastInstruction::Call(index) => Ok(Instruction::Call(index.try_into()?)),
            WastInstruction::Return => Ok(Instruction::Return),
            WastInstruction::Nop => Ok(Instruction::Nop),
            WastInstruction::If(ty) => Ok(Instruction::If(ty.try_into()?, None, None)),
            WastInstruction::Else(_) => Ok(Instruction::Else),
            WastInstruction::End(_) => Ok(Instruction::End),
            WastInstruction::Block(ty) => Ok(Instruction::Block(ty.try_into()?, None)),
            WastInstruction::Br(index) => Ok(Instruction::Br(index.try_into()?)),
            WastInstruction::Loop(ty) => Ok(Instruction::Loop(ty.try_into()?, None)),
            _ => Err(Error::msg("Unsupported instruction")),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{
        model::{
            BlockType, Expression, Func, FuncType, Index, Instruction, Line, LineExpression, Local,
            ValType,
        },
        parser::{Line as WastLine, LineExpression as WastLineExpression},
        test_utils::test_index,
    };
    use wast::{
        core::{
            BlockType as WastBlockType, Expression as WastExpression, Func as WastFunc,
            FunctionType, InlineExport, InlineImport, Instruction as WastInstruction,
            Local as WastLocal, TypeUse, ValType as WastValType,
        },
        parser::{self, ParseBuffer},
        token::{Float32, Float64, Id, Index as WastIndex, Span},
    };

    macro_rules! test_id {
        ($var:ident, $id:expr) => {
            let str_id = String::from($id);
            let buf_id = ParseBuffer::new(&str_id).unwrap();
            let $var = parser::parse::<Id>(&buf_id).unwrap();
        };
    }

    macro_rules! test_index {
        ($var:ident, $id:expr) => {
            let str_id = String::from($id);
            let buf_id = ParseBuffer::new(&str_id).unwrap();
            let id = parser::parse::<Id>(&buf_id).unwrap();
            let $var = WastIndex::Id(id);
        };
    }

    macro_rules! float_for {
        ($fname:ident, $type:ty) => {
            pub fn $fname(buf: &ParseBuffer) -> $type {
                parser::parse::<$type>(&buf).unwrap()
            }
        };
    }

    float_for!(float32_for, Float32);
    float_for!(float64_for, Float64);

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
        let instr = Instruction::try_from(&WastInstruction::RefI31);
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
        test_index!(index, "$id1");
        let index = Index::try_from(&index).unwrap();
        assert_eq!(index, Index::Id(String::from("id1")));
    }

    #[test]
    fn test_from_wast_func() {
        test_id!(fun_id, "$fun1");
        test_id!(param_id, "$param1");
        let func = Func::try_from(&WastFunc {
            id: Some(fun_id),
            name: None,
            exports: InlineExport { names: vec![] },
            ty: TypeUse {
                index: None,
                inline: Some(FunctionType {
                    params: Box::new([(Some(param_id), None, WastValType::I32)]),
                    results: Box::new([WastValType::I32]),
                }),
            },
            span: Span::from_offset(0),
            kind: wast::core::FuncKind::Inline {
                locals: Box::new([test_new_local_i32()]),
                expression: WastExpression {
                    instrs: Box::new([WastInstruction::I32Const(2)]),
                },
            },
        })
        .unwrap();

        assert_eq!(func.id, Some(String::from("fun1")));
        assert_eq!(func.ty.params.len(), 1);
        assert_eq!(func.ty.params[0].val_type, ValType::I32);
        assert_eq!(func.line_expression.locals.len(), 1);
        assert_eq!(func.line_expression.locals[0].val_type, ValType::I32);
        assert_eq!(func.line_expression.expr.instrs.len(), 1);
        assert_eq!(
            func.line_expression.expr.instrs[0],
            Instruction::I32Const(2)
        );
    }

    #[test]
    fn test_from_wast_import_error() {
        assert!(Func::try_from(&WastFunc {
            id: None,
            name: None,
            exports: InlineExport { names: vec![] },
            ty: TypeUse {
                index: None,
                inline: Some(FunctionType {
                    params: Box::new([]),
                    results: Box::new([]),
                }),
            },
            span: Span::from_offset(0),
            kind: wast::core::FuncKind::Import(InlineImport {
                module: "mod1",
                field: "fun1",
            }),
        })
        .is_err());
    }

    #[test]
    fn test_from_wast_export_error() {
        assert!(Func::try_from(&WastFunc {
            id: None,
            name: None,
            exports: InlineExport { names: vec!["fn"] },
            ty: TypeUse {
                index: None,
                inline: Some(FunctionType {
                    params: Box::new([]),
                    results: Box::new([]),
                }),
            },
            span: Span::from_offset(0),
            kind: wast::core::FuncKind::Inline {
                locals: Box::new([]),
                expression: WastExpression {
                    instrs: Box::new([]),
                },
            },
        })
        .is_err());
    }

    #[test]
    fn test_wast_func_type() {
        test_id!(param_id, "$param1");
        let ty = FuncType::try_from(&TypeUse {
            index: None,
            inline: Some(FunctionType {
                params: Box::new([(Some(param_id), None, WastValType::I32)]),
                results: Box::new([WastValType::I32]),
            }),
        })
        .unwrap();
        assert_eq!(ty.params.len(), 1);
        assert_eq!(ty.params[0].val_type, ValType::I32);
        assert_eq!(ty.params[0].id, Some(String::from("param1")));
        assert_eq!(ty.results.len(), 1);
        assert_eq!(ty.results[0], ValType::I32);
    }

    #[test]
    fn test_wast_func_type_type_error() {
        test_id!(param_id, "$param1");
        test_id!(ty_index, "$ty1");
        assert!(FuncType::try_from(&TypeUse {
            index: Some(WastIndex::Id(ty_index)),
            inline: Some(FunctionType {
                params: Box::new([(Some(param_id), None, WastValType::I32)]),
                results: Box::new([WastValType::I32]),
            }),
        })
        .is_err());
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
        test_id!(fun_id, "$fun1");
        let line_func = Line::try_from(&WastLine::Func(WastFunc {
            id: Some(fun_id),
            name: None,
            exports: InlineExport { names: vec![] },
            ty: TypeUse {
                index: None,
                inline: None,
            },
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

    #[test]
    fn test_wast_block_type() {
        test_id!(block_id, "$block1");
        test_id!(param_id, "$param1");
        let ty = BlockType::try_from(&Box::new(WastBlockType {
            label: Some(block_id),
            label_name: None,
            ty: TypeUse {
                // TODO: What is this index? Is this in use?
                index: None,
                inline: Some(FunctionType {
                    params: Box::new([(Some(param_id), None, WastValType::I32)]),
                    results: Box::new([WastValType::I32]),
                }),
            },
        }))
        .unwrap();

        assert_eq!(ty.label, Some(String::from("block1")));
        assert_eq!(ty.ty.params.len(), 1);
        assert_eq!(ty.ty.results.len(), 1);
    }

    #[test]
    fn test_from_wast_if_instruction() {
        let instr = Instruction::try_from(&WastInstruction::If(Box::new(WastBlockType {
            label: None,
            label_name: None,
            ty: TypeUse {
                index: None,
                inline: Some(FunctionType {
                    params: Box::new([]),
                    results: Box::new([WastValType::I32]),
                }),
            },
        })))
        .unwrap();
        assert_eq!(
            instr,
            Instruction::If(
                BlockType {
                    label: None,
                    ty: FuncType {
                        params: vec![],
                        results: vec![ValType::I32],
                    }
                },
                None,
                None
            )
        );
    }

    #[test]
    fn test_from_wast_block_instruction() {
        let instr = Instruction::try_from(&WastInstruction::Block(Box::new(WastBlockType {
            label: None,
            label_name: None,
            ty: TypeUse {
                index: None,
                inline: Some(FunctionType {
                    params: Box::new([]),
                    results: Box::new([WastValType::I32]),
                }),
            },
        })))
        .unwrap();
        assert_eq!(
            instr,
            Instruction::Block(
                BlockType {
                    label: None,
                    ty: FuncType {
                        params: vec![],
                        results: vec![ValType::I32],
                    }
                },
                None
            )
        );
    }

    #[test]
    fn test_from_wast_branch_instruction() {
        test_index!(index, "$id1");
        let instr = Instruction::try_from(&WastInstruction::Br(index)).unwrap();
        assert_eq!(instr, Instruction::Br(test_index("id1")));
    }

    #[test]
    fn test_from_wast_loop_instruction() {
        let instr = Instruction::try_from(&WastInstruction::Loop(Box::new(WastBlockType {
            label: None,
            label_name: None,
            ty: TypeUse {
                index: None,
                inline: Some(FunctionType {
                    params: Box::new([]),
                    results: Box::new([WastValType::I32]),
                }),
            },
        })))
        .unwrap();
        assert_eq!(
            instr,
            Instruction::Loop(
                BlockType {
                    label: None,
                    ty: FuncType {
                        params: vec![],
                        results: vec![ValType::I32],
                    }
                },
                None
            )
        );
    }
}

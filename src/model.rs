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

macro_rules! instrs {
    ({
        $(
           ($name:ident $(($($arg:tt)*))?, $wast:pat $(, ($capt:tt))?)
        ),*
    }) => {
        #[derive(PartialEq, Debug, Clone)]
        pub enum Instruction {
            $(
                $name $(($($arg)*))?,
            )*
        }

        impl TryFrom<&WastInstruction<'_>> for Instruction {
            type Error = Error;
            fn try_from(instruction: &WastInstruction) -> Result<Self> {
                match instruction {
                    $(
                        $wast => Ok(Instruction::$name $($capt)?),
                    )*
                    _ => Err(Error::msg("Unsupported instruction")),
                }
            }
        }

    };
}

instrs! {{
    (Drop, WastInstruction::Drop),
    (I32Const(i32), WastInstruction::I32Const(i), ((*i))),
    (I32Clz, WastInstruction::I32Clz),
    (I32Ctz, WastInstruction::I32Ctz),
    (I32Popcnt, WastInstruction::I32Popcnt),
    (I32Add, WastInstruction::I32Add),
    (I32Sub, WastInstruction::I32Sub),
    (I32Mul, WastInstruction::I32Mul),
    (I32DivS, WastInstruction::I32DivS),
    (I32DivU, WastInstruction::I32DivU),
    (I32RemS, WastInstruction::I32RemS),
    (I32RemU, WastInstruction::I32RemU),
    (I32And, WastInstruction::I32And),
    (I32Or, WastInstruction::I32Or),
    (I32Xor, WastInstruction::I32Xor),
    (I32Shl, WastInstruction::I32Shl),
    (I32ShrS, WastInstruction::I32ShrS),
    (I32ShrU, WastInstruction::I32ShrU),
    (I32Rotl, WastInstruction::I32Rotl),
    (I32Rotr, WastInstruction::I32Rotr),
    (I32Eqz, WastInstruction::I32Eqz),
    (I32Eq, WastInstruction::I32Eq),
    (I32Ne, WastInstruction::I32Ne),
    (I32LtS, WastInstruction::I32LtS),
    (I32LtU, WastInstruction::I32LtU),
    (I32GtS, WastInstruction::I32GtS),
    (I32GtU, WastInstruction::I32GtU),
    (I32LeS, WastInstruction::I32LeS),
    (I32LeU, WastInstruction::I32LeU),
    (I32GeS, WastInstruction::I32GeS),
    (I32GeU, WastInstruction::I32GeU),
    (I64Const(i64), WastInstruction::I64Const(i), ((*i))),
    (I64Clz, WastInstruction::I64Clz),
    (I64Ctz, WastInstruction::I64Ctz),
    (I64Popcnt, WastInstruction::I64Popcnt),
    (I64Add, WastInstruction::I64Add),
    (I64Sub, WastInstruction::I64Sub),
    (I64Mul, WastInstruction::I64Mul),
    (I64DivS, WastInstruction::I64DivS),
    (I64DivU, WastInstruction::I64DivU),
    (I64RemS, WastInstruction::I64RemS),
    (I64RemU, WastInstruction::I64RemU),
    (I64And, WastInstruction::I64And),
    (I64Or, WastInstruction::I64Or),
    (I64Xor, WastInstruction::I64Xor),
    (I64Shl, WastInstruction::I64Shl),
    (I64ShrS, WastInstruction::I64ShrS),
    (I64ShrU, WastInstruction::I64ShrU),
    (I64Rotl, WastInstruction::I64Rotl),
    (I64Rotr, WastInstruction::I64Rotr),
    (I64Eqz, WastInstruction::I64Eqz),
    (I64Eq, WastInstruction::I64Eq),
    (I64Ne, WastInstruction::I64Ne),
    (I64LtS, WastInstruction::I64LtS),
    (I64LtU, WastInstruction::I64LtU),
    (I64GtS, WastInstruction::I64GtS),
    (I64GtU, WastInstruction::I64GtU),
    (I64LeS, WastInstruction::I64LeS),
    (I64LeU, WastInstruction::I64LeU),
    (I64GeS, WastInstruction::I64GeS),
    (I64GeU, WastInstruction::I64GeU),
    (F32Const(f32), WastInstruction::F32Const(f), ((f32::from_bits(f.bits)))),
    (F32Abs, WastInstruction::F32Abs),
    (F32Neg, WastInstruction::F32Neg),
    (F32Ceil, WastInstruction::F32Ceil),
    (F32Floor, WastInstruction::F32Floor),
    (F32Trunc, WastInstruction::F32Trunc),
    (F32Nearest, WastInstruction::F32Nearest),
    (F32Sqrt, WastInstruction::F32Sqrt),
    (F32Add, WastInstruction::F32Add),
    (F32Sub, WastInstruction::F32Sub),
    (F32Mul, WastInstruction::F32Mul),
    (F32Div, WastInstruction::F32Div),
    (F32Min, WastInstruction::F32Min),
    (F32Max, WastInstruction::F32Max),
    (F32Copysign, WastInstruction::F32Copysign),
    (F32Eq, WastInstruction::F32Eq),
    (F32Ne, WastInstruction::F32Ne),
    (F32Lt, WastInstruction::F32Lt),
    (F32Gt, WastInstruction::F32Gt),
    (F32Le, WastInstruction::F32Le),
    (F32Ge, WastInstruction::F32Ge),
    (F64Const(f64), WastInstruction::F64Const(f), ((f64::from_bits(f.bits)))),
    (F64Abs, WastInstruction::F64Abs),
    (F64Neg, WastInstruction::F64Neg),
    (F64Ceil, WastInstruction::F64Ceil),
    (F64Floor, WastInstruction::F64Floor),
    (F64Trunc, WastInstruction::F64Trunc),
    (F64Nearest, WastInstruction::F64Nearest),
    (F64Sqrt, WastInstruction::F64Sqrt),
    (F64Add, WastInstruction::F64Add),
    (F64Sub, WastInstruction::F64Sub),
    (F64Mul, WastInstruction::F64Mul),
    (F64Div, WastInstruction::F64Div),
    (F64Min, WastInstruction::F64Min),
    (F64Max, WastInstruction::F64Max),
    (F64Copysign, WastInstruction::F64Copysign),
    (F64Eq, WastInstruction::F64Eq),
    (F64Ne, WastInstruction::F64Ne),
    (F64Lt, WastInstruction::F64Lt),
    (F64Gt, WastInstruction::F64Gt),
    (F64Le, WastInstruction::F64Le),
    (F64Ge, WastInstruction::F64Ge),
    (LocalGet(Index), WastInstruction::LocalGet(index), ((index.try_into()?))),
    (LocalSet(Index), WastInstruction::LocalSet(index), ((index.try_into()?))),
    (LocalTee(Index), WastInstruction::LocalTee(index), ((index.try_into()?))),
    (Call(Index), WastInstruction::Call(index), ((index.try_into()?))),
    (Return, WastInstruction::Return),
    (Nop, WastInstruction::Nop),
    (If(BlockType, Option<Expression>, Option<Expression>), WastInstruction::If(ty), ((ty.try_into()?, None, None))),
    (Else , WastInstruction::Else(_)),
    (End, WastInstruction::End(_)),
    (Block(BlockType, Option<Expression>), WastInstruction::Block(ty), ((ty.try_into()?, None))),
    (Loop(BlockType, Option<Expression>), WastInstruction::Loop(ty), ((ty.try_into()?, None))),
    (Br(Index), WastInstruction::Br(index), ((index.try_into()?)))
}}

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

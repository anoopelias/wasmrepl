use crate::{model::Index, value::Value};

macro_rules! test_val {
    ($fname:ident, $type:ty) => {
        pub fn $fname(n: $type) -> Value {
            n.into()
        }
    };
}

test_val!(test_val_i32, i32);
test_val!(test_val_i64, i64);
test_val!(test_val_f32, f32);
test_val!(test_val_f64, f64);

pub fn test_index(id: &str) -> Index {
    Index::Id(String::from(id))
}

macro_rules! test_block_type {
    () => {
        crate::model::BlockType {
            label: None,
            ty: crate::model::FuncType {
                params: vec![],
                results: vec![]

            }
        }
    };
    (($( $param:expr ),*), ($( $res:expr ),*)) => {
        crate::model::BlockType {
            label: None,
            ty: crate::model::FuncType {
                params: vec![
                    $( $param ),*
                ],
                results: vec![$( $res ),*]

            }
        }
    };
}

macro_rules! test_if {
    ($bt:expr) => {
        Instruction::If($bt, Some(Expression {instrs: vec![]}), Some(Expression {instrs: vec![]}))
    };
    ($bt:expr, ($( $if_instr:expr ),*), ($( $else_instr:expr ),*)) => {
        Instruction::If($bt, Some(Expression {instrs: vec![
                $( $if_instr ),*
            ]}), Some(Expression {instrs: vec![
                $( $else_instr ),*
            ]}))
    };
}

macro_rules! test_block {
    ($bt:expr) => {
        Instruction::Block($bt, Some(Expression {instrs: vec![]}))
    };
    ($bt:expr, ($( $block_instr:expr ),*)) => {
        Instruction::Block($bt, Some(Expression {instrs: vec![
            $( $block_instr ),*
        ]}))
    };
}

macro_rules! test_local_id {
    ($id:expr, $type:expr) => {
        Local {
            id: Some(String::from($id)),
            val_type: $type,
        }
    };
}

macro_rules! test_local {
    ($type:expr) => {
        Local {
            id: None,
            val_type: $type,
        }
    };
}

pub(crate) use test_block;
pub(crate) use test_block_type;
pub(crate) use test_if;
pub(crate) use test_local;
pub(crate) use test_local_id;

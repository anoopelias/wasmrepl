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

macro_rules! test_if {
    (($( $param:expr ),*)($( $res:expr ),*)) => {
        Instruction::If(crate::model::BlockType {
            label: None,
            ty: crate::model::FuncType {
                params: vec![
                    $( $param ),*
                ],
                results: vec![$( $res ),*]

            }
        })
    };
}

pub(crate) use test_if;

use crate::value::Value;

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

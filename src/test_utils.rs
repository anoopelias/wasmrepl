use wast::{
    parser::{self as wastparser, ParseBuffer},
    token::{Float32, Float64},
};

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

macro_rules! float_for {
    ($fname:ident, $type:ty) => {
        pub fn $fname(buf: &ParseBuffer) -> $type {
            wastparser::parse::<$type>(&buf).unwrap()
        }
    };
}

float_for!(float32_for, Float32);
float_for!(float64_for, Float64);

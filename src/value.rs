use anyhow::{Error, Result};
use std::fmt::{self, Display};

use crate::utils::IsSame;

#[derive(PartialEq, Debug)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::I32(n) => write!(f, "{}", n),
            Self::I64(n) => write!(f, "{}", n),
            Self::F32(n) => write!(f, "{}", n),
            Self::F64(n) => write!(f, "{}", n),
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::I32(n) => Self::I32(*n),
            Self::I64(n) => Self::I64(*n),
            Self::F32(n) => Self::F32(*n),
            Self::F64(n) => Self::F64(*n),
        }
    }
}

impl IsSame for Value {
    fn is_same(&self, other: &Self) -> Result<()> {
        match (self, other) {
            (Self::I32(_), Self::I32(_)) => Ok(()),
            (Self::I64(_), Self::I64(_)) => Ok(()),
            (Self::F32(_), Self::F32(_)) => Ok(()),
            (Self::F64(_), Self::F64(_)) => Ok(()),
            _ => Err(Error::msg("Type mismatch")),
        }
    }
}

macro_rules! map_num_types {
    ($type:ty, $e:path) => {
        impl From<$type> for Value {
            fn from(n: $type) -> Self {
                $e(n)
            }
        }
        impl TryInto<$type> for Value {
            type Error = Error;
            fn try_into(self) -> Result<$type> {
                match self {
                    $e(n) => Ok(n),
                    _ => Err(Error::msg("Type mismatch")),
                }
            }
        }
    };
}

map_num_types!(i32, Value::I32);
map_num_types!(i64, Value::I64);
map_num_types!(f32, Value::F32);
map_num_types!(f64, Value::F64);

impl Value {
    pub fn default_i32() -> Value {
        Self::I32(0)
    }

    pub fn default_i64() -> Value {
        Self::I64(0)
    }

    pub fn default_f32() -> Value {
        Self::F32(0.0)
    }

    pub fn default_f64() -> Value {
        Self::F64(0.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{test_val_f32, test_val_f64, test_val_i32, test_val_i64};
    use crate::{utils::IsSame, value::Value};
    use anyhow::Result;

    #[test]
    fn test_value_display() {
        assert_eq!(test_val_i32(1).to_string(), "1");
        assert_eq!(test_val_i64(2).to_string(), "2");
        assert_eq!(test_val_f32(3.14).to_string(), "3.14");
        assert_eq!(test_val_f64(3.14f64).to_string(), "3.14");
    }

    #[test]
    fn test_from_num() {
        assert_eq!(Value::from(1), test_val_i32(1));
        assert_eq!(Value::from(2i64), test_val_i64(2));
        assert_eq!(Value::from(3.14f32), test_val_f32(3.14));
        assert_eq!(Value::from(3.14f64), test_val_f64(3.14));
    }

    #[test]
    fn test_to_num() {
        let v: i32 = test_val_i32(1).try_into().unwrap();
        assert_eq!(v, 1);
        let v: i64 = test_val_i64(2).try_into().unwrap();
        assert_eq!(v, 2);
        let v: f32 = test_val_f32(3.0).try_into().unwrap();
        assert_eq!(v, 3.0);
        let v: f64 = test_val_f64(4.0).try_into().unwrap();
        assert_eq!(v, 4.0);
    }

    #[test]
    fn test_to_num_type_error() {
        let v: Result<i64> = test_val_i32(1).try_into();
        assert!(v.is_err());
    }

    #[test]
    fn test_from_num_type() {
        let v: Value = Value::from(1i32);
        assert_eq!(v, test_val_i32(1));
        let v: Value = Value::from(2i64);
        assert_eq!(v, test_val_i64(2));
        let v: Value = Value::from(3.14f32);
        assert_eq!(v, test_val_f32(3.14));
        let v: Value = Value::from(3.14f64);
        assert_eq!(v, test_val_f64(3.14));
    }

    #[test]
    fn test_into_num_type() {
        let i: Value = test_val_i32(1).try_into().unwrap();
        assert_eq!(i, Value::I32(1));
        let i: Value = test_val_i64(1).try_into().unwrap();
        assert_eq!(i, Value::I64(1));
        let i: Value = test_val_f32(1.0).try_into().unwrap();
        assert_eq!(i, Value::F32(1.0));
    }

    #[test]
    fn test_is_same() {
        test_val_i32(1).is_same(&test_val_i32(5)).unwrap();
        test_val_i64(3).is_same(&test_val_i64(12)).unwrap();
        test_val_f32(1.1).is_same(&test_val_f32(2.3)).unwrap();
        test_val_f64(1.1).is_same(&test_val_f64(2.3)).unwrap();
    }

    #[test]
    fn test_is_same_error() {
        assert!(test_val_i32(1).is_same(&test_val_i64(2)).is_err());
        assert!(test_val_i32(1).is_same(&test_val_f32(2.0)).is_err());
        assert!(test_val_i64(1).is_same(&test_val_f64(2.0)).is_err());
    }
}

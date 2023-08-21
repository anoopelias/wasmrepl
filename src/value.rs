use crate::op::Op;
use anyhow::{Error, Result};
use std::fmt::{self, Display};

#[derive(PartialEq, Debug)]
pub enum Value {
    Integer(Integer),
    Float(Float),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Integer(n) => write!(f, "{}", n),
            Self::Float(n) => write!(f, "{}", n),
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::Integer(n) => Self::Integer(n.clone()),
            Self::Float(n) => Self::Float(n.clone()),
        }
    }
}

macro_rules! map_value_types {
    ($type:ty, $e:path) => {
        impl TryInto<$type> for Value {
            type Error = Error;
            fn try_into(self) -> Result<$type> {
                match self {
                    $e(i) => Ok(i),
                    _ => Err(Error::msg("Type mismatch")),
                }
            }
        }
        impl From<$type> for Value {
            fn from(n: $type) -> Self {
                $e(n)
            }
        }
    };
}

map_value_types!(Integer, Value::Integer);
map_value_types!(Float, Value::Float);

macro_rules! map_num_types {
    ($type:ty, $e:path, $de:path) => {
        impl From<$type> for Value {
            fn from(n: $type) -> Self {
                $e($de(n))
            }
        }
        impl TryInto<$type> for Value {
            type Error = Error;
            fn try_into(self) -> Result<$type> {
                match self {
                    $e($de(n)) => Ok(n),
                    _ => Err(Error::msg("Type mismatch")),
                }
            }
        }
    };
}

map_num_types!(i32, Value::Integer, Integer::I32);
map_num_types!(i64, Value::Integer, Integer::I64);
map_num_types!(f32, Value::Float, Float::F32);

use crate::{float::Float, integer::Integer};

impl Value {
    pub fn default_i32() -> Value {
        Self::Integer(Integer::I32(0))
    }

    pub fn default_i64() -> Value {
        Self::Integer(Integer::I64(0))
    }

    pub fn is_same(&self, other: &Self) -> Result<()> {
        match (self, other) {
            (Self::Integer(Integer::I32(_)), Self::Integer(Integer::I32(_))) => Ok(()),
            (Self::Integer(Integer::I64(_)), Self::Integer(Integer::I64(_))) => Ok(()),
            (Self::Float(Float::F32(_)), Self::Float(Float::F32(_))) => Ok(()),
            _ => Err(Error::msg("Type mismatch")),
        }
    }

    pub fn op(
        &self,
        other: &Self,
        i32_op: fn(i32, i32) -> Result<i32>,
        i64_op: fn(i64, i64) -> Result<i64>,
        f32_op: fn(f32, f32) -> Result<f32>,
    ) -> Result<Self> {
        match (self, other) {
            (Self::Integer(Integer::I32(m)), Self::Integer(Integer::I32(n))) => {
                Ok(i32_op(*m, *n)?.into())
            }
            (Self::Integer(Integer::I64(m)), Self::Integer(Integer::I64(n))) => {
                Ok(i64_op(*m, *n)?.into())
            }
            (Self::Float(Float::F32(m)), Self::Float(Float::F32(n))) => Ok(f32_op(*m, *n)?.into()),
            _ => Err(Error::msg("Type mismatch")),
        }
    }

    pub fn add(&self, other: &Self) -> Result<Self> {
        self.op(other, i32::add, i64::add, f32::add)
    }

    pub fn sub(&self, other: &Self) -> Result<Self> {
        self.op(other, i32::sub, i64::sub, f32::sub)
    }

    pub fn mul(&self, other: &Self) -> Result<Self> {
        self.op(other, i32::mul, i64::mul, f32::mul)
    }

    pub fn div(&self, other: &Self) -> Result<Self> {
        self.op(other, i32::div, i64::div, f32::div)
    }
}

#[cfg(test)]
pub mod test_utils {
    use crate::value::Value;

    pub fn test_val_i32(n: i32) -> Value {
        n.into()
    }

    pub fn test_val_i64(n: i64) -> Value {
        n.into()
    }

    pub fn test_val_f32(n: f32) -> Value {
        n.into()
    }
}

#[cfg(test)]
mod tests {
    use crate::float::Float;
    use crate::integer::Integer;
    use crate::value::test_utils::{test_val_f32, test_val_i32, test_val_i64};
    use crate::value::Value;
    use anyhow::Result;

    #[test]
    fn test_value_display() {
        assert_eq!(test_val_i32(1).to_string(), "1");
        assert_eq!(test_val_f32(3.14).to_string(), "3.14");
    }

    #[test]
    fn test_from_num() {
        assert_eq!(Value::from(1), test_val_i32(1));
        assert_eq!(Value::from(2i64), test_val_i64(2));
        assert_eq!(Value::from(3.14f32), test_val_f32(3.14));
    }

    #[test]
    fn test_to_num() {
        let v: i32 = test_val_i32(1).try_into().unwrap();
        assert_eq!(v, 1);
        let v: i64 = test_val_i64(2).try_into().unwrap();
        assert_eq!(v, 2);
        let v: f32 = test_val_f32(3.0).try_into().unwrap();
        assert_eq!(v, 3.0);
    }

    #[test]
    fn test_to_num_type_error() {
        let v: Result<i64> = test_val_i32(1).try_into();
        assert!(v.is_err());
    }

    #[test]
    fn test_from_num_type() {
        let v: Value = Integer::I32(1).into();
        assert_eq!(v, test_val_i32(1));
    }

    #[test]
    fn test_into_num_type() {
        let i: Integer = test_val_i32(1).try_into().unwrap();
        assert_eq!(i, Integer::I32(1));
        let i: Integer = test_val_i64(1).try_into().unwrap();
        assert_eq!(i, Integer::I64(1));
        let i: Float = test_val_f32(1.0).try_into().unwrap();
        assert_eq!(i, Float::F32(1.0));
    }

    #[test]
    fn test_add() {
        assert_eq!(
            test_val_i32(1).add(&test_val_i32(2)).unwrap(),
            test_val_i32(3)
        );
        assert_eq!(
            test_val_i64(1).add(&test_val_i64(2)).unwrap(),
            test_val_i64(3)
        );
        assert_eq!(
            test_val_f32(1.1).add(&test_val_f32(2.3)).unwrap(),
            test_val_f32(3.4)
        );
    }

    #[test]
    fn test_op_error() {
        assert!(test_val_i32(1).add(&test_val_i64(2)).is_err());
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            test_val_i32(1).sub(&test_val_i32(2)).unwrap(),
            test_val_i32(-1)
        );
        assert_eq!(
            test_val_i64(1).sub(&test_val_i64(2)).unwrap(),
            test_val_i64(-1)
        );
        assert_eq!(
            test_val_f32(1.1).sub(&test_val_f32(2.3)).unwrap(),
            // Due to floating point error, the result is not exactly -1.2
            // TODO: Is there a better way to do this?
            test_val_f32(-1.1999999)
        );
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            test_val_i32(2).mul(&test_val_i32(3)).unwrap(),
            test_val_i32(6)
        );
        assert_eq!(
            test_val_i64(2).mul(&test_val_i64(3)).unwrap(),
            test_val_i64(6)
        );
        assert_eq!(
            test_val_f32(1.1).mul(&test_val_f32(2.3)).unwrap(),
            test_val_f32(2.53)
        );
    }

    #[test]
    fn test_div() {
        assert_eq!(
            test_val_i32(6).div(&test_val_i32(2)).unwrap(),
            test_val_i32(3)
        );
        assert_eq!(
            test_val_i64(6).div(&test_val_i64(2)).unwrap(),
            test_val_i64(3)
        );
        assert_eq!(
            test_val_f32(1.1).div(&test_val_f32(2.3)).unwrap(),
            test_val_f32(0.4782609)
        );
    }

    #[test]
    fn test_is_same() {
        test_val_i32(1).is_same(&test_val_i32(5)).unwrap();
        test_val_i64(3).is_same(&test_val_i64(12)).unwrap();
        test_val_f32(1.1).is_same(&test_val_f32(2.3)).unwrap();
    }

    #[test]
    fn test_is_same_error() {
        assert!(test_val_i32(1).is_same(&test_val_i64(2)).is_err());
        assert!(test_val_i32(1).is_same(&test_val_f32(2.0)).is_err());
    }
}

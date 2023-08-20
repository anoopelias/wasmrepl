use anyhow::{Error, Result};
use std::{
    fmt::{self, Display},
    mem::discriminant,
};

#[derive(PartialEq, Debug)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::I32(n) => write!(f, "{}", n.to_string()),
            Value::I64(n) => write!(f, "{}", n.to_string()),
            Value::F32(n) => write!(f, "{}", n.to_string()),
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::I32(n) => Self::I32(*n),
            Self::I64(n) => Self::I64(*n),
            Self::F32(n) => Self::F32(*n),
        }
    }
}

macro_rules! map_types {
    ($type:ty, $e:path) => {
        impl From<$type> for Value {
            fn from(n: $type) -> Self {
                $e(n)
            }
        }
        impl TryInto<$type> for Value {
            type Error = anyhow::Error;

            fn try_into(self) -> Result<$type, Self::Error> {
                match self {
                    $e(n) => Ok(n),
                    _ => Err(anyhow::Error::msg("Type mismatch")),
                }
            }
        }
    };
}

map_types!(i64, Value::I64);
map_types!(i32, Value::I32);
map_types!(f32, Value::F32);

macro_rules! match_type {
    ($val:ident, $e:path) => {
        if matches!($val, $e(_)) {
            Ok(())
        } else {
            Err(Error::msg("Type mismatch"))
        }
    };
}

pub(crate) use match_type;

impl Value {
    pub fn default_i32() -> Value {
        Value::I32(0)
    }

    pub fn default_i64() -> Value {
        Value::I64(0)
    }

    pub fn is_same(&self, other: &Self) -> Result<()> {
        if discriminant(self) == discriminant(other) {
            Ok(())
        } else {
            Err(Error::msg("Type mismatch"))
        }
    }

    pub fn leading_zeros(&self) -> Result<Self> {
        match self {
            Self::I32(n) => Ok(Self::I32(n.leading_zeros() as i32)),
            Self::I64(n) => Ok(Self::I64(n.leading_zeros() as i64)),
            _ => Err(Error::msg("Operation not supported")),
        }
    }
    pub fn trailing_zeros(&self) -> Result<Self> {
        match self {
            Self::I32(n) => Ok(Self::I32(n.trailing_zeros() as i32)),
            Self::I64(n) => Ok(Self::I64(n.trailing_zeros() as i64)),
            _ => Err(Error::msg("Operation not supported")),
        }
    }

    pub fn add(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::I32(n), Self::I32(m)) => Ok(Self::I32(n.wrapping_add(*m))),
            (Self::I64(n), Self::I64(m)) => Ok(Self::I64(n.wrapping_add(*m))),
            (Self::F32(n), Self::F32(m)) => Ok(Self::F32(*n + *m)),
            _ => Err(Error::msg("Type mismatch")),
        }
    }

    pub fn sub(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::I32(n), Self::I32(m)) => Ok(Self::I32(n.wrapping_sub(*m))),
            (Self::I64(n), Self::I64(m)) => Ok(Self::I64(n.wrapping_sub(*m))),
            (Self::F32(n), Self::F32(m)) => Ok(Self::F32(*n - *m)),
            _ => Err(Error::msg("Type mismatch")),
        }
    }

    pub fn mul(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::I32(n), Self::I32(m)) => Ok(Self::I32(n.wrapping_mul(*m))),
            (Self::I64(n), Self::I64(m)) => Ok(Self::I64(n.wrapping_mul(*m))),
            (Self::F32(n), Self::F32(m)) => Ok(Self::F32(*n * *m)),
            _ => Err(Error::msg("Type mismatch")),
        }
    }

    pub fn div(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::I32(n), Self::I32(m)) => {
                if *m == 0 {
                    Err(Error::msg("Divide by zero"))
                } else {
                    Ok(Self::I32(n.wrapping_div(*m)))
                }
            }
            (Self::I64(n), Self::I64(m)) => {
                if *m == 0 {
                    Err(Error::msg("Divide by zero"))
                } else {
                    Ok(Self::I64(n.wrapping_div(*m)))
                }
            }
            (Self::F32(n), Self::F32(m)) => {
                if *m == 0.0 {
                    Err(Error::msg("Divide by zero"))
                } else {
                    Ok(Self::F32(*n / *m))
                }
            }

            _ => Err(Error::msg("Type mismatch")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::value::Value;

    #[test]
    fn test_value_display() {
        assert_eq!(Value::I32(1).to_string(), "1");
        assert_eq!(Value::I64(1).to_string(), "1");
    }

    #[test]
    fn test_i32_value_try_into() {
        let i32val = Value::I32(4);

        let num: i32 = i32val.try_into().unwrap();
        assert_eq!(num, 4);
    }

    #[test]
    fn test_value_try_into_error() {
        let i64val = Value::I64(4);
        assert!(<Value as TryInto<i32>>::try_into(i64val).is_err());
    }

    #[test]
    fn test_value_from() {
        assert_eq!(Value::from(1), Value::I32(1));
        assert_eq!(Value::from(2i64), Value::I64(2));
    }

    #[test]
    fn test_leading_zeros() {
        assert_eq!(Value::I32(1).leading_zeros().unwrap(), Value::I32(31));
        assert_eq!(Value::I64(1).leading_zeros().unwrap(), Value::I64(63));
    }

    #[test]
    fn test_leading_zeros_error() {
        assert!(Value::F32(3.0).leading_zeros().is_err());
    }

    #[test]
    fn test_trailing_zeros() {
        assert_eq!(Value::I32(1024).trailing_zeros().unwrap(), Value::I32(10));
        assert_eq!(Value::I64(2048).trailing_zeros().unwrap(), Value::I64(11));
    }

    #[test]
    fn test_trailing_zeros_error() {
        assert!(Value::F32(3.0).trailing_zeros().is_err());
    }

    #[test]
    fn test_add() {
        assert_eq!(Value::I32(1).add(&Value::I32(2)).unwrap(), Value::I32(3));
        assert_eq!(Value::I64(1).add(&Value::I64(2)).unwrap(), Value::I64(3));
        assert_eq!(
            Value::F32(1.1).add(&Value::F32(2.3)).unwrap(),
            Value::F32(3.4)
        );
    }

    #[test]
    fn test_add_overflow() {
        assert_eq!(
            Value::I32(i32::MAX).add(&Value::I32(1)).unwrap(),
            Value::I32(i32::MIN)
        );
        assert_eq!(
            Value::I64(i64::MAX).add(&Value::I64(1)).unwrap(),
            Value::I64(i64::MIN)
        );
    }

    #[test]
    fn test_add_error() {
        assert!(Value::I32(1).add(&Value::I64(2)).is_err());
    }

    #[test]
    fn test_sub() {
        assert_eq!(Value::I32(1).sub(&Value::I32(2)).unwrap(), Value::I32(-1));
        assert_eq!(Value::I64(1).sub(&Value::I64(2)).unwrap(), Value::I64(-1));
        assert_eq!(
            Value::F32(1.1).sub(&Value::F32(2.3)).unwrap(),
            // Due to floating point error, the result is not exactly -1.2
            // TODO: Is there a better way to do this?
            Value::F32(-1.1999999)
        );
    }

    #[test]
    fn test_sub_overflow() {
        assert_eq!(
            Value::I32(i32::MIN).sub(&Value::I32(1)).unwrap(),
            Value::I32(i32::MAX)
        );
        assert_eq!(
            Value::I64(i64::MIN).sub(&Value::I64(1)).unwrap(),
            Value::I64(i64::MAX)
        );
    }

    #[test]
    fn test_sub_error() {
        assert!(Value::I32(1).sub(&Value::I64(2)).is_err());
    }

    #[test]
    fn test_mul() {
        assert_eq!(Value::I32(2).mul(&Value::I32(3)).unwrap(), Value::I32(6));
        assert_eq!(Value::I64(2).mul(&Value::I64(3)).unwrap(), Value::I64(6));
        assert_eq!(
            Value::F32(1.1).mul(&Value::F32(2.3)).unwrap(),
            Value::F32(2.53)
        );
    }

    #[test]
    fn test_mul_overflow() {
        assert_eq!(
            Value::I32(i32::MIN).mul(&Value::I32(2)).unwrap(),
            Value::I32(0)
        );
        assert_eq!(
            Value::I64(i64::MIN).mul(&Value::I64(2)).unwrap(),
            Value::I64(0)
        );
    }

    #[test]
    fn test_mul_error() {
        assert!(Value::I32(1).mul(&Value::I64(2)).is_err());
    }

    #[test]
    fn test_div() {
        assert_eq!(Value::I32(6).div(&Value::I32(2)).unwrap(), Value::I32(3));
        assert_eq!(Value::I64(6).div(&Value::I64(2)).unwrap(), Value::I64(3));
        assert_eq!(
            Value::F32(1.1).div(&Value::F32(2.3)).unwrap(),
            Value::F32(0.4782609)
        );
    }

    #[test]
    fn test_div_overflow() {
        assert_eq!(
            Value::I32(i32::MIN).div(&Value::I32(-1)).unwrap(),
            Value::I32(i32::MIN)
        );
        assert_eq!(
            Value::I64(i64::MIN).div(&Value::I64(-1)).unwrap(),
            Value::I64(i64::MIN)
        );
    }

    #[test]
    fn test_div_error() {
        assert!(Value::I32(1).div(&Value::I64(2)).is_err());
    }

    #[test]
    fn test_div_by_zero() {
        assert!(Value::I32(1).div(&Value::I32(0)).is_err());
        assert!(Value::I64(1).div(&Value::I64(0)).is_err());
        assert!(Value::F32(1.0).div(&Value::F32(0.0)).is_err());
    }

    #[test]
    fn test_is_same() {
        Value::I32(1).is_same(&Value::I32(5)).unwrap();
        Value::I64(3).is_same(&Value::I64(12)).unwrap();
    }

    #[test]
    fn test_is_same_error() {
        assert!(Value::I32(1).is_same(&Value::I64(2)).is_err());
    }
}

use anyhow::{Error, Result};
use std::fmt::{self, Display};

#[derive(PartialEq, Debug, Eq)]
pub enum Value {
    I32(i32),
    I64(i64),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::I32(n) => write!(f, "{}", n.to_string()),
            Value::I64(n) => write!(f, "{}", n.to_string()),
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::I32(n) => Self::I32(*n),
            Self::I64(n) => Self::I64(*n),
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
    pub fn leading_zeros(&self) -> Self {
        match self {
            Self::I32(n) => Self::I32(n.leading_zeros() as i32),
            Self::I64(n) => Self::I64(n.leading_zeros() as i64),
        }
    }
    pub fn trailing_zeros(&self) -> Self {
        match self {
            Self::I32(n) => Self::I32(n.trailing_zeros() as i32),
            Self::I64(n) => Self::I64(n.trailing_zeros() as i64),
        }
    }

    pub fn add(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::I32(n), Self::I32(m)) => Ok(Self::I32(n.wrapping_add(*m))),
            (Self::I64(n), Self::I64(m)) => Ok(Self::I64(n.wrapping_add(*m))),
            _ => Err(Error::msg("Type mismatch")),
        }
    }

    pub fn sub(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::I32(n), Self::I32(m)) => Ok(Self::I32(n.wrapping_sub(*m))),
            (Self::I64(n), Self::I64(m)) => Ok(Self::I64(n.wrapping_sub(*m))),
            _ => Err(Error::msg("Type mismatch")),
        }
    }

    pub fn mul(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::I32(n), Self::I32(m)) => Ok(Self::I32(n.wrapping_mul(*m))),
            (Self::I64(n), Self::I64(m)) => Ok(Self::I64(n.wrapping_mul(*m))),
            _ => Err(Error::msg("Type mismatch")),
        }
    }

    pub fn div_s(&self, other: &Self) -> Result<Self> {
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
        assert_eq!(Value::I32(1).leading_zeros(), Value::I32(31));
        assert_eq!(Value::I64(1).leading_zeros(), Value::I64(63));
    }

    #[test]
    fn test_trailing_zeros() {
        assert_eq!(Value::I32(1024).trailing_zeros(), Value::I32(10));
        assert_eq!(Value::I64(2048).trailing_zeros(), Value::I64(11));
    }

    #[test]
    fn test_add() {
        assert_eq!(Value::I32(1).add(&Value::I32(2)).unwrap(), Value::I32(3));
        assert_eq!(Value::I64(1).add(&Value::I64(2)).unwrap(), Value::I64(3));
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
    fn test_div_s() {
        assert_eq!(Value::I32(6).div_s(&Value::I32(2)).unwrap(), Value::I32(3));
        assert_eq!(Value::I64(6).div_s(&Value::I64(2)).unwrap(), Value::I64(3));
    }

    #[test]
    fn test_div_s_overflow() {
        assert_eq!(
            Value::I32(i32::MIN).div_s(&Value::I32(-1)).unwrap(),
            Value::I32(i32::MIN)
        );
        assert_eq!(
            Value::I64(i64::MIN).div_s(&Value::I64(-1)).unwrap(),
            Value::I64(i64::MIN)
        );
    }

    #[test]
    fn test_div_s_error() {
        assert!(Value::I32(1).div_s(&Value::I64(2)).is_err());
    }

    #[test]
    fn test_div_s_by_zero() {
        assert!(Value::I32(1).div_s(&Value::I32(0)).is_err());
        assert!(Value::I64(1).div_s(&Value::I64(0)).is_err());
    }
}

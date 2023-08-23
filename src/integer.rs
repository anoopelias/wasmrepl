use crate::op::NumOps;
use anyhow::{Error, Result};
use std::fmt::{self, Display};

#[derive(PartialEq, Debug)]
pub enum Integer {
    I32(i32),
    I64(i64),
}

impl Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::I32(n) => write!(f, "{}", n),
            Self::I64(n) => write!(f, "{}", n),
        }
    }
}

impl Clone for Integer {
    fn clone(&self) -> Self {
        match self {
            Self::I32(n) => Self::I32(*n),
            Self::I64(n) => Self::I64(*n),
        }
    }
}

impl Integer {
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
            (Self::I32(a), Self::I32(b)) => Ok(Self::I32(a.add(*b))),
            (Self::I64(a), Self::I64(b)) => Ok(Self::I64(a.add(*b))),
            _ => Err(Error::msg("Type mismatch")),
        }
    }

    pub fn sub(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::I32(a), Self::I32(b)) => Ok(Self::I32(a.sub(*b))),
            (Self::I64(a), Self::I64(b)) => Ok(Self::I64(a.sub(*b))),
            _ => Err(Error::msg("Type mismatch")),
        }
    }

    pub fn mul(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::I32(a), Self::I32(b)) => Ok(Self::I32(a.mul(*b))),
            (Self::I64(a), Self::I64(b)) => Ok(Self::I64(a.mul(*b))),
            _ => Err(Error::msg("Type mismatch")),
        }
    }

    pub fn div(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::I32(a), Self::I32(b)) => Ok(Self::I32(a.div(*b)?)),
            (Self::I64(a), Self::I64(b)) => Ok(Self::I64(a.div(*b)?)),
            _ => Err(Error::msg("Type mismatch")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::integer::Integer;

    #[test]
    fn test_leading_zeros() {
        assert_eq!(Integer::I32(1).leading_zeros(), Integer::I32(31));
        assert_eq!(Integer::I64(1).leading_zeros(), Integer::I64(63));
    }

    #[test]
    fn test_trailing_zeros() {
        assert_eq!(Integer::I32(1024).trailing_zeros(), Integer::I32(10));
        assert_eq!(Integer::I64(2048).trailing_zeros(), Integer::I64(11));
    }

    #[test]
    fn test_add() {
        assert_eq!(
            Integer::I32(1).add(&Integer::I32(2)).unwrap(),
            Integer::I32(3)
        );
        assert_eq!(
            Integer::I64(1).add(&Integer::I64(2)).unwrap(),
            Integer::I64(3)
        );
    }

    #[test]
    fn test_i32_add_error() {
        assert!(Integer::I32(1).add(&Integer::I64(2)).is_err());
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            Integer::I32(1).sub(&Integer::I32(2)).unwrap(),
            Integer::I32(-1)
        );
        assert_eq!(
            Integer::I64(1).sub(&Integer::I64(2)).unwrap(),
            Integer::I64(-1)
        );
    }

    #[test]
    fn test_i32_sub_error() {
        assert!(Integer::I32(1).sub(&Integer::I64(2)).is_err());
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            Integer::I32(1).mul(&Integer::I32(2)).unwrap(),
            Integer::I32(2)
        );
        assert_eq!(
            Integer::I64(1).mul(&Integer::I64(2)).unwrap(),
            Integer::I64(2)
        );
    }

    #[test]
    fn test_i32_mul_error() {
        assert!(Integer::I32(1).mul(&Integer::I64(2)).is_err());
    }

    #[test]
    fn test_div() {
        assert_eq!(
            Integer::I32(1).div(&Integer::I32(2)).unwrap(),
            Integer::I32(0)
        );
        assert_eq!(
            Integer::I64(1).div(&Integer::I64(2)).unwrap(),
            Integer::I64(0)
        );
    }

    #[test]
    fn test_i32_div_error() {
        assert!(Integer::I32(1).div(&Integer::I64(2)).is_err());
    }
}

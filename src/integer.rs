use crate::ops::IntOps;
use crate::ops::NumOps;
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

macro_rules! impl_unary_op {
    ($fnam:ident) => {
        impl Integer {
            pub fn $fnam(&self) -> Self {
                match self {
                    Self::I32(a) => Self::I32(a.$fnam() as i32),
                    Self::I64(a) => Self::I64(a.$fnam() as i64),
                }
            }
        }
    };
}

impl_unary_op!(leading_zeros);
impl_unary_op!(trailing_zeros);

macro_rules! impl_binary_op {
    ($fnam:ident) => {
        impl Integer {
            pub fn $fnam(&self, other: &Self) -> Result<Self> {
                match (self, other) {
                    (Self::I32(a), Self::I32(b)) => Ok(Self::I32(a.$fnam(*b))),
                    (Self::I64(a), Self::I64(b)) => Ok(Self::I64(a.$fnam(*b))),
                    _ => Err(Error::msg("Type mismatch")),
                }
            }
        }
    };
}

impl_binary_op!(add);
impl_binary_op!(sub);
impl_binary_op!(mul);

// `div` is different from others
impl Integer {
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
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            Integer::I32(1).mul(&Integer::I32(2)).unwrap(),
            Integer::I32(2)
        );
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

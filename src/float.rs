use crate::ops::FloatOps;
use crate::ops::NumOps;
use anyhow::{Error, Result};
use std::fmt::{self, Display};

#[derive(PartialEq, Debug)]
pub enum Float {
    F32(f32),
    F64(f64),
}

impl Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::F32(n) => write!(f, "{}", n),
            Self::F64(n) => write!(f, "{}", n),
        }
    }
}

impl Clone for Float {
    fn clone(&self) -> Self {
        match self {
            Self::F32(n) => Self::F32(*n),
            Self::F64(n) => Self::F64(*n),
        }
    }
}

macro_rules! impl_binary_op {
    ($fnam:ident) => {
        impl Float {
            pub fn $fnam(&self, other: &Self) -> Result<Self> {
                match (self, other) {
                    (Self::F32(a), Self::F32(b)) => Ok(Self::F32(a.$fnam(*b))),
                    (Self::F64(a), Self::F64(b)) => Ok(Self::F64(a.$fnam(*b))),
                    _ => Err(Error::msg("Type mismatch")),
                }
            }
        }
    };
}

impl_binary_op!(add);
impl_binary_op!(sub);
impl_binary_op!(mul);
impl_binary_op!(div);

#[cfg(test)]
mod tests {
    use crate::float::Float;

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Float::F32(3.14)), "3.14");
        assert_eq!(format!("{}", Float::F64(3.14f64)), "3.14");
    }

    #[test]
    fn test_clone() {
        assert_eq!(Float::F32(1.0).clone(), Float::F32(1.0));
        assert_eq!(Float::F64(4.0f64).clone(), Float::F64(4.0f64));
    }

    #[test]
    fn test_add() {
        assert_eq!(
            Float::F32(1.0).add(&Float::F32(2.0)).unwrap(),
            Float::F32(3.0)
        );
        assert_eq!(
            Float::F64(1.0).add(&Float::F64(2.0)).unwrap(),
            Float::F64(3.0)
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            Float::F32(1.0).sub(&Float::F32(2.0)).unwrap(),
            Float::F32(-1.0)
        );
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            Float::F32(4.0).mul(&Float::F32(2.0)).unwrap(),
            Float::F32(8.0)
        );
    }

    #[test]
    fn test_div() {
        assert_eq!(
            Float::F32(1.0).div(&Float::F32(2.0)).unwrap(),
            Float::F32(0.5)
        );
    }
}

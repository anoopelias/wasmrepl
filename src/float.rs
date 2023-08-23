use crate::op::Op;
use anyhow::Result;
use std::fmt::{self, Display};

#[derive(PartialEq, Debug)]
pub enum Float {
    F32(f32),
}

impl Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::F32(n) => write!(f, "{}", n),
        }
    }
}

impl Clone for Float {
    fn clone(&self) -> Self {
        match self {
            Self::F32(n) => Self::F32(*n),
        }
    }
}

impl Float {
    pub fn add(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::F32(a), Self::F32(b)) => Ok(Self::F32(a.add(*b))),
        }
    }

    pub fn sub(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::F32(a), Self::F32(b)) => Ok(Self::F32(a.sub(*b))),
        }
    }

    pub fn mul(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::F32(a), Self::F32(b)) => Ok(Self::F32(a.mul(*b))),
        }
    }

    pub fn div(&self, other: &Self) -> Result<Self> {
        match (self, other) {
            (Self::F32(a), Self::F32(b)) => Ok(Self::F32(a.div(*b)?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::float::Float;

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Float::F32(3.14)), "3.14");
    }

    #[test]
    fn test_clone() {
        assert_eq!(Float::F32(1.0).clone(), Float::F32(1.0));
    }

    #[test]
    fn test_add() {
        assert_eq!(
            Float::F32(1.0).add(&Float::F32(2.0)).unwrap(),
            Float::F32(3.0)
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

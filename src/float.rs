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
}

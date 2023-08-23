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

#[cfg(test)]
mod tests {
    use crate::integer::Integer;

    #[test]
    fn test_clone() {
        assert_eq!(Integer::I32(1).clone(), Integer::I32(1));
        assert_eq!(Integer::I64(1).clone(), Integer::I64(1));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Integer::I32(1)), "1");
        assert_eq!(format!("{}", Integer::I64(4)), "4");
    }
}

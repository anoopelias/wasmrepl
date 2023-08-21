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
}

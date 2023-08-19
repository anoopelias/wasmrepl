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
}

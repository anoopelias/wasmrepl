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

impl TryInto<i32> for Value {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            Value::I32(n) => Ok(n),
            _ => Err(anyhow::Error::msg("Not an i32")),
        }
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::I32(n)
    }
}

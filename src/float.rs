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

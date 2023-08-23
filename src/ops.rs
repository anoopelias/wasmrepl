use anyhow::{Error, Result};

pub trait NumOps {
    fn add(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn sub(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn mul(self, rhs: Self) -> Self
    where
        Self: Sized;
}

macro_rules! impl_integer_num_ops {
    ($t:ty) => {
        impl NumOps for $t {
            fn add(self, rhs: Self) -> Self {
                self.wrapping_add(rhs)
            }
            fn sub(self, rhs: Self) -> Self {
                self.wrapping_sub(rhs)
            }
            fn mul(self, rhs: Self) -> Self {
                self.wrapping_mul(rhs)
            }
        }
    };
}

impl_integer_num_ops!(i32);
impl_integer_num_ops!(i64);

macro_rules! impl_float_num_ops {
    ($t:ty) => {
        impl NumOps for $t {
            fn add(self, rhs: Self) -> Self {
                self + rhs
            }
            fn sub(self, rhs: Self) -> Self {
                self - rhs
            }
            fn mul(self, rhs: Self) -> Self {
                self * rhs
            }
        }
    };
}

impl_float_num_ops!(f32);
impl_float_num_ops!(f64);

pub trait IntOps: NumOps {
    fn clz(self) -> Self
    where
        Self: Sized;
    fn ctz(self) -> Self
    where
        Self: Sized;
    fn div(self, rhs: Self) -> Result<Self>
    where
        Self: Sized;
}

macro_rules! impl_int_ops {
    ($t:ty) => {
        impl IntOps for $t {
            fn clz(self) -> Self {
                self.leading_zeros() as Self
            }
            fn ctz(self) -> Self {
                self.trailing_zeros() as Self
            }
            fn div(self, rhs: Self) -> Result<Self> {
                if rhs == 0 {
                    Err(Error::msg("Divide by zero"))
                } else {
                    let (res, overflow) = self.overflowing_div(rhs);
                    if overflow {
                        Err(Error::msg("Integer Overflow"))
                    } else {
                        Ok(res)
                    }
                }
            }
        }
    };
}

impl_int_ops!(i32);
impl_int_ops!(i64);

pub trait FloatOps: NumOps {
    fn div(self, rhs: Self) -> Self
    where
        Self: Sized;
}

macro_rules! impl_float_ops {
    ($t:ty) => {
        impl FloatOps for $t {
            fn div(self, rhs: Self) -> Self {
                self / rhs
            }
        }
    };
}

impl_float_ops!(f32);
impl_float_ops!(f64);

#[cfg(test)]
mod tests {

    use crate::ops::FloatOps;
    use crate::ops::IntOps;
    use crate::ops::NumOps;

    #[test]
    fn test_i32_add() {
        assert_eq!(1.add(2), 3);
    }

    #[test]
    fn test_i32_add_overflow() {
        assert_eq!(i32::MAX.add(1), i32::MIN);
    }

    #[test]
    fn test_i32_sub() {
        assert_eq!(1.sub(2), -1i32);
    }

    #[test]
    fn test_i32_sub_overflow() {
        assert_eq!(i32::MIN.sub(1), i32::MAX);
    }

    #[test]
    fn test_i32_mul() {
        assert_eq!(4.mul(2), 8);
    }

    #[test]
    fn test_i32_mul_overflow() {
        assert_eq!(i32::MAX.mul(2), -2);
    }

    #[test]
    fn test_i64_add() {
        assert_eq!(1i64.add(2i64), 3i64);
    }

    #[test]
    fn test_leading_zeros() {
        assert_eq!(1i32.leading_zeros(), 31);
        assert_eq!(1i64.leading_zeros(), 63);
    }

    #[test]
    fn test_trailing_zeros() {
        assert_eq!(1024i32.trailing_zeros(), 10);
        assert_eq!(2048i64.trailing_zeros(), 11);
    }

    #[test]
    fn test_i32_div() {
        assert_eq!(7.div(3).unwrap(), 2);
    }

    #[test]
    fn test_i32_div_overflow_error() {
        // Pulled from WASM test suite
        assert!(i32::MIN.div(-1).is_err());
    }

    #[test]
    fn test_i32_div_by_zero() {
        assert!(5i32.div(0i32).is_err());
    }

    #[test]
    fn test_i64_div() {
        assert_eq!(1i64.div(2i64).unwrap(), 0);
    }

    #[test]
    fn test_f32_add() {
        assert_eq!(1.0.add(2.0), 3.0);
    }

    #[test]
    fn test_f32_sub() {
        assert_eq!(2.0.sub(1.5), 0.5);
    }

    #[test]
    fn test_f32_mul() {
        assert_eq!(2.5.mul(2.0), 5.0);
    }

    #[test]
    fn test_f32_div() {
        assert_eq!(7.0.div(2.0), 3.5);
    }

    #[test]
    fn test_f32_div_by_zero() {
        assert_eq!(5.0.div(0.0), f32::INFINITY);
    }

    #[test]
    fn test_f64_add() {
        assert_eq!(1.0f64.add(2.0f64), 3.0f64);
    }

    #[test]
    fn test_f64_div() {
        assert_eq!(7.0f64.div(2.0f64), 3.5f64);
    }
}

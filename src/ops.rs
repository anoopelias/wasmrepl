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
    fn popcnt(self) -> Self
    where
        Self: Sized;
    fn div_s(self, rhs: Self) -> Result<Self>
    where
        Self: Sized;
    fn div_u(self, rhs: Self) -> Result<Self>
    where
        Self: Sized;
    fn rem_s(self, rhs: Self) -> Result<Self>
    where
        Self: Sized;
    fn rem_u(self, rhs: Self) -> Result<Self>
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
            fn popcnt(self) -> Self {
                self.count_ones() as Self
            }
            fn div_s(self, rhs: Self) -> Result<Self> {
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
            fn div_u(self, rhs: Self) -> Result<Self> {
                let a = self.into_unsigned();
                let b = rhs.into_unsigned();
                if b == 0 {
                    Err(Error::msg("Divide by zero"))
                } else {
                    Ok(Self::from_ne_bytes((a / b).to_ne_bytes()))
                }
            }
            fn rem_s(self, rhs: Self) -> Result<Self> {
                if rhs == 0 {
                    Err(Error::msg("Divide by zero"))
                } else {
                    // This is mathematically not possible but is due to
                    // the implementation artifact we need to use `wrapping_rem`
                    Ok(self.wrapping_rem(rhs))
                }
            }
            fn rem_u(self, rhs: Self) -> Result<Self> {
                let a = self.into_unsigned();
                let b = rhs.into_unsigned();
                if b == 0 {
                    Err(Error::msg("Divide by zero"))
                } else {
                    Ok(Self::from_ne_bytes((a % b).to_ne_bytes()))
                }
            }
        }
    };
}

impl_int_ops!(i32);
impl_int_ops!(i64);

trait IntoUnsigned<U> {
    fn into_unsigned(self) -> U;
}

macro_rules! into_unsigned {
    ($s:ty, $t:ty) => {
        impl IntoUnsigned<$t> for $s {
            fn into_unsigned(self) -> $t {
                <$t>::from_ne_bytes(self.to_ne_bytes())
            }
        }
    };
}

into_unsigned!(i32, u32);
into_unsigned!(i64, u64);

pub trait FloatOps: NumOps {
    fn neg(self) -> Self
    where
        Self: Sized;
    fn div(self, rhs: Self) -> Self
    where
        Self: Sized;
}

macro_rules! impl_float_ops {
    ($t:ty) => {
        impl FloatOps for $t {
            fn neg(self) -> Self {
                self * -1.0
            }
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
    fn test_clz() {
        assert_eq!(1i32.clz(), 31);
        assert_eq!(1i64.clz(), 63);
    }

    #[test]
    fn test_ctz() {
        assert_eq!(1024i32.ctz(), 10);
        assert_eq!(2048i64.ctz(), 11);
    }

    #[test]
    fn test_popcnt() {
        assert_eq!(1023i32.popcnt(), 10);
        assert_eq!(1023i64.popcnt(), 10);
    }

    #[test]
    fn test_i32_div_s() {
        assert_eq!(7.div_s(3).unwrap(), 2);
    }

    #[test]
    fn test_i32_div_s_overflow_error() {
        // Pulled from WASM test suite
        assert!(i32::MIN.div_s(-1).is_err());
    }

    #[test]
    fn test_i32_div_s_by_zero() {
        assert!(5i32.div_s(0i32).is_err());
    }

    #[test]
    fn test_i32_div_u() {
        assert_eq!(7.div_s(3).unwrap(), 2);
        // Pulled from WASM test suite
        assert_eq!(i32::MIN.div_u(2).unwrap(), 0x40000000);
    }

    #[test]
    fn test_i32_div_u_div_by_zero_error() {
        assert!(5.div_u(0).is_err());
    }

    #[test]
    fn test_i32_rem_s() {
        assert_eq!(7.rem_s(3).unwrap(), 1);
    }

    #[test]
    fn test_i32_rem_s_overflow_error() {
        assert_eq!(i32::MIN.rem_s(-1).unwrap(), 0);
    }

    #[test]
    fn test_i32_rem_s_by_zero() {
        assert!(5i32.rem_s(0i32).is_err());
    }

    #[test]
    fn test_i32_rem_u() {
        assert_eq!(7.rem_s(3).unwrap(), 1);
        assert_eq!(i32::MIN.rem_u(-1).unwrap(), i32::MIN);
    }

    #[test]
    fn test_i32_rem_u_div_by_zero_error() {
        assert!(5.rem_u(0).is_err());
    }

    #[test]
    fn test_i64_div_s() {
        assert_eq!(1i64.div_s(2i64).unwrap(), 0);
    }

    #[test]
    fn test_i64_div_u() {
        // Pulled from WASM test suite
        assert_eq!(i64::MIN.div_u(2).unwrap(), 0x4000000000000000i64);
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
    fn test_f32_abs() {
        assert_eq!((-2.5f32).abs(), 2.5);
        assert_eq!((2.5f32).abs(), 2.5);
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

    #[test]
    fn test_f64_abs() {
        assert_eq!((-2.5f64).abs(), 2.5f64);
    }
}

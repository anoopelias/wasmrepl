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
    fn shr_s(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn shr_u(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn rotl(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn rotr(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn eqz(self) -> Self
    where
        Self: Sized;
    fn eq(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn ne(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn lt_s(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn lt_u(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn gt_s(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn gt_u(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn le_s(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn le_u(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn ge_s(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn ge_u(self, rhs: Self) -> Self
    where
        Self: Sized;
}

macro_rules! impl_int_ops {
    ($t:ty, $ut:ty) => {
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
                let a = self as $ut;
                let b = rhs as $ut;
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
                let a = self as $ut;
                let b = rhs as $ut;
                if b == 0 {
                    Err(Error::msg("Divide by zero"))
                } else {
                    Ok(Self::from_ne_bytes((a % b).to_ne_bytes()))
                }
            }
            fn shr_s(self, rhs: Self) -> Self {
                self.wrapping_shr(rhs as u32)
            }
            fn shr_u(self, rhs: Self) -> Self {
                let n = self as $ut;
                n.wrapping_shr(rhs as u32) as Self
            }
            fn rotl(self, rhs: Self) -> Self {
                self.rotate_left(rhs as u32)
            }
            fn rotr(self, rhs: Self) -> Self {
                self.rotate_right(rhs as u32)
            }
            fn eqz(self) -> Self {
                if self == 0 {
                    1
                } else {
                    0
                }
            }
            fn eq(self, rhs: Self) -> Self {
                if self == rhs {
                    1
                } else {
                    0
                }
            }
            fn ne(self, rhs: Self) -> Self {
                if self == rhs {
                    0
                } else {
                    1
                }
            }
            fn lt_s(self, rhs: Self) -> Self {
                if self < rhs {
                    1
                } else {
                    0
                }
            }
            fn lt_u(self, rhs: Self) -> Self {
                let a = self as $ut;
                let b = rhs as $ut;
                if a < b {
                    1
                } else {
                    0
                }
            }
            fn gt_s(self, rhs: Self) -> Self {
                if self > rhs {
                    1
                } else {
                    0
                }
            }
            fn gt_u(self, rhs: Self) -> Self {
                let a = self as $ut;
                let b = rhs as $ut;
                if a > b {
                    1
                } else {
                    0
                }
            }
            fn le_s(self, rhs: Self) -> Self {
                if self <= rhs {
                    1
                } else {
                    0
                }
            }
            fn le_u(self, rhs: Self) -> Self {
                let a = self as $ut;
                let b = rhs as $ut;
                if a <= b {
                    1
                } else {
                    0
                }
            }
            fn ge_s(self, rhs: Self) -> Self {
                if self >= rhs {
                    1
                } else {
                    0
                }
            }
            fn ge_u(self, rhs: Self) -> Self {
                let a = self as $ut;
                let b = rhs as $ut;
                if a >= b {
                    1
                } else {
                    0
                }
            }
        }
    };
}

impl_int_ops!(i32, u32);
impl_int_ops!(i64, u64);

pub trait FloatOps: NumOps {
    fn neg(self) -> Self
    where
        Self: Sized;
    fn div(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn eq(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn ne(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn lt(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn gt(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn le(self, rhs: Self) -> Self
    where
        Self: Sized;
    fn ge(self, rhs: Self) -> Self
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
            fn eq(self, rhs: Self) -> Self {
                if self == rhs {
                    1.0
                } else {
                    0.0
                }
            }
            fn ne(self, rhs: Self) -> Self {
                if self == rhs {
                    0.0
                } else {
                    1.0
                }
            }
            fn lt(self, rhs: Self) -> Self {
                if self < rhs {
                    1.0
                } else {
                    0.0
                }
            }
            fn gt(self, rhs: Self) -> Self {
                if self > rhs {
                    1.0
                } else {
                    0.0
                }
            }
            fn le(self, rhs: Self) -> Self {
                if self <= rhs {
                    1.0
                } else {
                    0.0
                }
            }
            fn ge(self, rhs: Self) -> Self {
                if self >= rhs {
                    1.0
                } else {
                    0.0
                }
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
    fn test_i32_shr_s() {
        assert_eq!(1i32.shr_s(2), 0);
    }

    #[test]
    fn test_i32_shr_s_overflow() {
        assert_eq!(2i32.shr_s(33), 1);
    }

    #[test]
    fn test_i32_shr_s_negative() {
        assert_eq!((-2i32).shr_s(1), -1);
    }

    #[test]
    fn test_i32_shr_s_by_negative() {
        assert_eq!(2i32.shr_s(-31), 1);
    }

    #[test]
    fn test_i32_shr_u() {
        assert_eq!(1i32.shr_u(2), 0);
    }

    #[test]
    fn test_i32_shr_u_overflow() {
        assert_eq!(2i32.shr_u(33), 1);
    }

    #[test]
    fn test_i32_shr_u_negative() {
        assert_eq!((-2i32).shr_u(1), 0x7FFFFFFF);
    }

    #[test]
    fn test_i32_shr_u_by_negative() {
        assert_eq!(2i32.shr_u(-31), 1);
    }

    #[test]
    fn test_i32_rotl() {
        assert_eq!(0x10008002.rotl(37), 0x100042);
    }

    #[test]
    fn test_i32_rotr() {
        assert_eq!(0x100042.rotr(37), 0x10008002);
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
    fn test_i64_shr_s() {
        assert_eq!(4i64.shr_s(1), 2i64);
    }

    #[test]
    fn test_i64_shr_u() {
        assert_eq!(1i64.shr_u(2), 0);
    }

    #[test]
    fn test_i64_rotl() {
        assert_eq!(0x10008002i64.rotl(37), 0x10004000000002);
    }

    #[test]
    fn test_i64_rotr() {
        assert_eq!(0x10004000000002i64.rotr(37), 0x10008002i64);
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

    #[test]
    fn test_i32_eqz() {
        assert_eq!(5i32.eqz(), 0);
        assert_eq!(0i32.eqz(), 1);
    }
    #[test]
    fn test_i64_eqz() {
        assert_eq!(5i64.eqz(), 0);
        assert_eq!(0i64.eqz(), 1);
    }

    #[test]
    fn test_i32_eq() {
        assert_eq!(1i32.eq(1), 1);
        assert_eq!(1i32.eq(2), 0);
    }
    #[test]
    fn test_i64_eq() {
        assert_eq!(1i64.eq(1), 1);
        assert_eq!(1i64.eq(2), 0);
    }

    #[test]
    fn test_f32_eq() {
        assert_eq!(1.0f32.eq(1.0), 1.0);
        assert_eq!(1.0f32.eq(2.0), 0.0);
    }

    #[test]
    fn test_f64_eq() {
        assert_eq!(1.0f64.eq(1.0), 1.0);
        assert_eq!(1.0f64.eq(2.0), 0.0);
    }

    #[test]
    fn test_i32_ne() {
        assert_eq!(1i32.ne(1), 0);
        assert_eq!(1i32.ne(2), 1);
    }

    #[test]
    fn test_i64_ne() {
        assert_eq!(1i64.ne(1), 0);
        assert_eq!(1i64.ne(2), 1);
    }

    #[test]
    fn test_f32_ne() {
        assert_eq!(1.0f32.ne(1.0), 0.0);
        assert_eq!(1.0f32.ne(2.0), 1.0);
    }

    #[test]
    fn test_f64_ne() {
        assert_eq!(1.0f64.ne(1.0), 0.0);
        assert_eq!(1.0f64.ne(2.0), 1.0);
    }

    #[test]
    fn test_i32_lt_s() {
        assert_eq!(1i32.lt_s(2), 1);
        assert_eq!(1i32.lt_s(1), 0);
        assert_eq!(2i32.lt_s(1), 0);
        assert_eq!((-1i32).lt_s(1), 1);
    }

    #[test]
    fn test_i64_lt_s() {
        assert_eq!(1i64.lt_s(2), 1);
        assert_eq!(1i64.lt_s(1), 0);
        assert_eq!(2i64.lt_s(1), 0);
        assert_eq!((-1i64).lt_s(1), 1);
    }

    #[test]
    fn test_i32_lt_u() {
        assert_eq!(1i32.lt_u(2), 1);
        assert_eq!(1i32.lt_u(1), 0);
        assert_eq!(2i32.lt_u(1), 0);
        assert_eq!((-1i32).lt_u(1), 0);
    }

    #[test]
    fn test_i64_lt_u() {
        assert_eq!(1i64.lt_u(2), 1);
        assert_eq!(1i64.lt_u(1), 0);
        assert_eq!(2i64.lt_u(1), 0);
        assert_eq!((-1i64).lt_u(1), 0);
    }

    #[test]
    fn test_f32_lt() {
        assert_eq!(1.0f32.lt(2.0), 1.0);
        assert_eq!(1.0f32.lt(1.0), 0.0);
        assert_eq!(2.0f32.lt(1.0), 0.0);
        assert_eq!((-1.0f32).lt(1.0), 1.0);
    }

    #[test]
    fn test_f64_lt() {
        assert_eq!(1.0f64.lt(2.0), 1.0);
        assert_eq!(1.0f64.lt(1.0), 0.0);
        assert_eq!(2.0f64.lt(1.0), 0.0);
        assert_eq!((-1.0f64).lt(1.0), 1.0);
    }

    #[test]
    fn test_i32_gt_s() {
        assert_eq!(1i32.gt_s(2), 0);
        assert_eq!(1i32.gt_s(1), 0);
        assert_eq!(2i32.gt_s(1), 1);
        assert_eq!((-1i32).gt_s(1), 0);
    }

    #[test]
    fn test_i64_gt_s() {
        assert_eq!(1i64.gt_s(2), 0);
        assert_eq!(1i64.gt_s(1), 0);
        assert_eq!(2i64.gt_s(1), 1);
        assert_eq!((-1i64).gt_s(1), 0);
    }

    #[test]
    fn test_i32_gt_u() {
        assert_eq!(1i32.gt_u(2), 0);
        assert_eq!(1i32.gt_u(1), 0);
        assert_eq!(2i32.gt_u(1), 1);
        assert_eq!((-1i32).gt_u(1), 1);
    }

    #[test]
    fn test_i64_gt_u() {
        assert_eq!(1i64.gt_u(2), 0);
        assert_eq!(1i64.gt_u(1), 0);
        assert_eq!(2i64.gt_u(1), 1);
        assert_eq!((-1i64).gt_u(1), 1);
    }

    #[test]
    fn test_f32_gt() {
        assert_eq!(1.0f32.gt(2.0), 0.0);
        assert_eq!(1.0f32.gt(1.0), 0.0);
        assert_eq!(2.0f32.gt(1.0), 1.0);
        assert_eq!((-1.0f32).gt(1.0), 0.0);
    }

    #[test]
    fn test_f64_gt() {
        assert_eq!(1.0f64.gt(2.0), 0.0);
        assert_eq!(1.0f64.gt(1.0), 0.0);
        assert_eq!(2.0f64.gt(1.0), 1.0);
        assert_eq!((-1.0f64).gt(1.0), 0.0);
    }

    #[test]
    fn test_i32_le_s() {
        assert_eq!(1i32.le_s(2), 1);
        assert_eq!(1i32.le_s(1), 1);
        assert_eq!(2i32.le_s(1), 0);
        assert_eq!((-1i32).le_s(1), 1);
    }

    #[test]
    fn test_i64_le_s() {
        assert_eq!(1i64.le_s(2), 1);
        assert_eq!(1i64.le_s(1), 1);
        assert_eq!(2i64.le_s(1), 0);
        assert_eq!((-1i64).le_s(1), 1);
    }

    #[test]
    fn test_i32_le_u() {
        assert_eq!(1i32.le_u(2), 1);
        assert_eq!(1i32.le_u(1), 1);
        assert_eq!(2i32.le_u(1), 0);
        assert_eq!((-1i32).le_u(1), 0);
    }

    #[test]
    fn test_i64_le_u() {
        assert_eq!(1i64.le_u(2), 1);
        assert_eq!(1i64.le_u(1), 1);
        assert_eq!(2i64.le_u(1), 0);
        assert_eq!((-1i64).le_u(1), 0);
    }

    #[test]
    fn test_f32_le() {
        assert_eq!(1.0f32.le(2.0), 1.0);
        assert_eq!(1.0f32.le(1.0), 1.0);
        assert_eq!(2.0f32.le(1.0), 0.0);
        assert_eq!((-1.0f32).le(1.0), 1.0);
    }

    #[test]
    fn test_f64_le() {
        assert_eq!(1.0f64.le(2.0), 1.0);
        assert_eq!(1.0f64.le(1.0), 1.0);
        assert_eq!(2.0f64.le(1.0), 0.0);
        assert_eq!((-1.0f64).le(1.0), 1.0);
    }

    #[test]
    fn test_i32_ge_s() {
        assert_eq!(1i32.ge_s(2), 0);
        assert_eq!(1i32.ge_s(1), 1);
        assert_eq!(2i32.ge_s(1), 1);
        assert_eq!((-1i32).ge_s(1), 0);
    }

    #[test]
    fn test_i64_ge_s() {
        assert_eq!(1i64.ge_s(2), 0);
        assert_eq!(1i64.ge_s(1), 1);
        assert_eq!(2i64.ge_s(1), 1);
        assert_eq!((-1i64).ge_s(1), 0);
    }

    #[test]
    fn test_i32_ge_u() {
        assert_eq!(1i32.ge_u(2), 0);
        assert_eq!(1i32.ge_u(1), 1);
        assert_eq!(2i32.ge_u(1), 1);
        assert_eq!((-1i32).ge_u(1), 1);
    }

    #[test]
    fn test_i64_ge_u() {
        assert_eq!(1i64.ge_u(2), 0);
        assert_eq!(1i64.ge_u(1), 1);
        assert_eq!(2i64.ge_u(1), 1);
        assert_eq!((-1i64).ge_u(1), 1);
    }

    #[test]
    fn test_f32_ge() {
        assert_eq!(1.0f32.ge(2.0), 0.0);
        assert_eq!(1.0f32.ge(1.0), 1.0);
        assert_eq!(2.0f32.ge(1.0), 1.0);
        assert_eq!((-1.0f32).ge(1.0), 0.0);
    }
}

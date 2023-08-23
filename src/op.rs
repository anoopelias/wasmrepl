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
    fn div(self, rhs: Self) -> Result<Self>
    where
        Self: Sized;
}

impl NumOps for i32 {
    fn add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
    fn sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }
    fn mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }
    fn div(self, rhs: Self) -> Result<Self> {
        if rhs == 0 {
            Err(Error::msg("Divide by zero"))
        } else {
            Ok(self.wrapping_div(rhs))
        }
    }
}

impl NumOps for i64 {
    fn add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
    fn sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }
    fn mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }
    fn div(self, rhs: Self) -> Result<Self> {
        if rhs == 0 {
            Err(Error::msg("Divide by zero"))
        } else {
            Ok(self.wrapping_div(rhs))
        }
    }
}

impl NumOps for f32 {
    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
    fn sub(self, rhs: Self) -> Self {
        self - rhs
    }
    fn mul(self, rhs: Self) -> Self {
        self * rhs
    }
    fn div(self, rhs: Self) -> Result<Self> {
        Ok(self / rhs)
    }
}

#[cfg(test)]
mod tests {

    use crate::op::NumOps;

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
    fn test_i32_div() {
        assert_eq!(7.div(3).unwrap(), 2);
    }

    #[test]
    fn test_i32_div_by_zero() {
        assert!(5i32.div(0i32).is_err());
    }

    #[test]
    fn test_i64_add() {
        assert_eq!(1i64.add(2i64), 3i64);
    }

    #[test]
    fn test_i64_add_overflow() {
        assert_eq!(i64::MAX.add(1i64), i64::MIN);
    }

    #[test]
    fn test_i64_sub() {
        assert_eq!(1i64.sub(2i64), -1i64);
    }

    #[test]
    fn test_i64_sub_overflow() {
        assert_eq!(i64::MIN.sub(1i64), i64::MAX);
    }

    #[test]
    fn test_i64_mul() {
        assert_eq!(7i64.mul(2i64), 14i64);
    }

    #[test]
    fn test_i64_mul_overflow() {
        assert_eq!(i64::MAX.mul(2i64), -2);
    }

    #[test]
    fn test_i64_div() {
        assert_eq!(1i64.div(2i64).unwrap(), 0);
        // TODO: Do we need wrapping for div?
        assert_eq!(i64::MIN.div(2i64).unwrap(), i64::MIN / 2);
    }

    #[test]
    fn test_i64_div_by_zero() {
        assert!(5i64.div(0i64).is_err());
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
    fn test_f32_div_by_zero() {
        assert_eq!(5.0.div(0.0).unwrap(), f32::INFINITY);
    }
}

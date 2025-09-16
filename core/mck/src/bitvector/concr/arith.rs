use crate::{
    concr::{PanicResult, RConcreteBitvector},
    forward::HwArith,
    panic::message::{PANIC_NUM_DIV_BY_ZERO, PANIC_NUM_NO_PANIC, PANIC_NUM_REM_BY_ZERO},
};

use super::ConcreteBitvector;

impl HwArith for RConcreteBitvector {
    type DivRemResult = PanicResult<Self>;

    fn arith_neg(self) -> Self {
        let result = self.value.wrapping_neg();
        Self::from_masked_u64(result, self.width)
    }

    fn add(self, rhs: Self) -> Self {
        assert_eq!(self.width, rhs.width);
        let result = self.value.wrapping_add(rhs.value);
        Self::from_masked_u64(result, self.width)
    }

    fn sub(self, rhs: Self) -> Self {
        assert_eq!(self.width, rhs.width);
        let result = self.value.wrapping_sub(rhs.value);
        Self::from_masked_u64(result, self.width)
    }

    fn mul(self, rhs: Self) -> Self {
        assert_eq!(self.width, rhs.width);
        let result = self.value.wrapping_mul(rhs.value);
        Self::from_masked_u64(result, self.width)
    }

    fn udiv(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.width, rhs.width);

        let dividend = self.to_u64();
        let divisor = rhs.to_u64();
        if divisor == 0 {
            // return panic
            // put all-ones in the division result
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_DIV_BY_ZERO),
                result: RConcreteBitvector::from_masked_u64(!0u64, self.width),
            };
        }
        let result = dividend
            .checked_div(divisor)
            .expect("Unsigned division should only return none on zero divisor");
        PanicResult {
            panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC),
            result: RConcreteBitvector::from_masked_u64(result, self.width),
        }
    }

    fn urem(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.width, rhs.width);

        let dividend = self.to_u64();
        let divisor = rhs.to_u64();
        if divisor == 0 {
            // return panic
            // put the dividend in the remainder result
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_REM_BY_ZERO),
                result: self,
            };
        }
        let result = dividend
            .checked_rem(divisor)
            .expect("Unsigned remainder should only return none on zero divisor");
        PanicResult {
            panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC),
            result: RConcreteBitvector::from_masked_u64(result, self.width),
        }
    }

    fn sdiv(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.width, rhs.width);

        let dividend = self.to_i64();
        let divisor = rhs.to_i64();
        if divisor == 0 {
            // return panic
            // put all-ones in the division result
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_DIV_BY_ZERO),
                result: RConcreteBitvector::from_masked_u64(!0u64, self.width),
            };
        }
        let signed_minus_one = self.bit_mask_u64();
        let signed_minimum = self.sign_bit_mask_u64();
        if self.value == signed_minimum && rhs.value == signed_minus_one {
            // division result is dividend on overflow, no panic
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC),
                result: self,
            };
        }
        let result = dividend
            .checked_div(divisor)
            .map(|r| r as u64)
            .expect("Signed division should only return none on zero divisor or overflow");
        PanicResult {
            panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC),
            result: RConcreteBitvector::from_masked_u64(result, self.width),
        }
    }

    fn srem(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.width, rhs.width);

        let dividend = self.to_i64();
        let divisor = rhs.to_i64();
        if divisor == 0 {
            // return panic
            // put the dividend in the remainder result
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_REM_BY_ZERO),
                result: self,
            };
        }
        let signed_minus_one = self.bit_mask_u64();
        let signed_minimum = self.sign_bit_mask_u64();
        if self.value == signed_minimum && rhs.value == signed_minus_one {
            // remainder result is zero on overflow, no panic
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC),
                result: RConcreteBitvector::new(0, self.width),
            };
        }
        // result after division overflow is zero
        let result = dividend
            .checked_rem(divisor)
            .expect("Signed remainder should only return none on zero divisor or overflow");
        PanicResult {
            panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC),
            result: RConcreteBitvector::from_masked_u64(result as u64, self.width),
        }
    }
}

impl RConcreteBitvector {
    pub(crate) fn checked_add(self, rhs: Self) -> Option<Self> {
        assert_eq!(self.width, rhs.width);

        let result = self.value.checked_add(rhs.value)?;
        if result & !self.bit_mask_u64() != 0 {
            return None;
        }
        Some(Self::new(result, self.width))
    }

    pub(crate) fn checked_mul(self, rhs: Self) -> Option<Self> {
        let result = self.value.checked_mul(rhs.value)?;
        if result & !self.bit_mask_u64() != 0 {
            return None;
        }
        Some(Self::new(result, self.width))
    }
}

impl<const W: u32> HwArith for ConcreteBitvector<W> {
    type DivRemResult = PanicResult<Self>;

    fn arith_neg(self) -> Self {
        self.to_runtime().arith_neg().unwrap_typed()
    }

    fn add(self, rhs: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.add(rhs).unwrap_typed()
    }

    fn sub(self, rhs: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.sub(rhs).unwrap_typed()
    }

    fn mul(self, rhs: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.mul(rhs).unwrap_typed()
    }

    fn udiv(self, rhs: Self) -> PanicResult<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.udiv(rhs).unwrap_typed()
    }

    fn urem(self, rhs: Self) -> PanicResult<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.urem(rhs).unwrap_typed()
    }

    fn sdiv(self, rhs: Self) -> PanicResult<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.sdiv(rhs).unwrap_typed()
    }

    fn srem(self, rhs: Self) -> PanicResult<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.srem(rhs).unwrap_typed()
    }
}

impl<const W: u32> ConcreteBitvector<W> {
    pub(crate) fn checked_add(self, rhs: Self) -> Option<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.checked_add(rhs).map(|r| r.unwrap_typed())
    }

    pub(crate) fn checked_mul(self, rhs: Self) -> Option<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.checked_mul(rhs).map(|r| r.unwrap_typed())
    }
}

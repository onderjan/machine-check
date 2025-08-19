use machine_check_common::{PANIC_NUM_DIV_BY_ZERO, PANIC_NUM_NO_PANIC, PANIC_NUM_REM_BY_ZERO};

use crate::{
    concr::{PanicResult, RConcreteBitvector, RPanicResult},
    forward::HwArith,
};

use super::ConcreteBitvector;

impl RConcreteBitvector {
    pub fn arith_neg(self) -> Self {
        let result = self.value.wrapping_neg();
        Self::from_masked_u64(result, self.width)
    }

    pub fn hw_add(self, rhs: Self) -> Self {
        assert_eq!(self.width, rhs.width);
        let result = self.value.wrapping_add(rhs.value);
        Self::from_masked_u64(result, self.width)
    }

    pub fn hw_sub(self, rhs: Self) -> Self {
        assert_eq!(self.width, rhs.width);
        let result = self.value.wrapping_sub(rhs.value);
        Self::from_masked_u64(result, self.width)
    }

    pub fn hw_mul(self, rhs: Self) -> Self {
        assert_eq!(self.width, rhs.width);
        let result = self.value.wrapping_mul(rhs.value);
        Self::from_masked_u64(result, self.width)
    }

    pub fn hw_udiv(self, rhs: Self) -> RPanicResult<Self> {
        assert_eq!(self.width, rhs.width);

        let dividend = self.to_u64();
        let divisor = rhs.to_u64();
        if divisor == 0 {
            // return panic
            // put all-ones in the division result
            return RPanicResult {
                panic: RConcreteBitvector::from_unwrapped_u64(PANIC_NUM_DIV_BY_ZERO, 32),
                result: RConcreteBitvector::from_masked_u64(!0u64, self.width),
            };
        }
        let result = dividend
            .checked_div(divisor)
            .expect("Unsigned division should only return none on zero divisor");
        RPanicResult {
            panic: RConcreteBitvector::from_unwrapped_u64(PANIC_NUM_NO_PANIC, 32),
            result: RConcreteBitvector::from_masked_u64(result, self.width),
        }
    }

    pub fn hw_urem(self, rhs: Self) -> RPanicResult<Self> {
        assert_eq!(self.width, rhs.width);

        let dividend = self.to_u64();
        let divisor = rhs.to_u64();
        if divisor == 0 {
            // return panic
            // put the dividend in the remainder result
            return RPanicResult {
                panic: RConcreteBitvector::from_unwrapped_u64(PANIC_NUM_REM_BY_ZERO, 32),
                result: self,
            };
        }
        let result = dividend
            .checked_rem(divisor)
            .expect("Unsigned remainder should only return none on zero divisor");
        RPanicResult {
            panic: RConcreteBitvector::from_unwrapped_u64(PANIC_NUM_NO_PANIC, 32),
            result: RConcreteBitvector::from_masked_u64(result, self.width),
        }
    }

    pub fn hw_sdiv(self, rhs: Self) -> RPanicResult<Self> {
        assert_eq!(self.width, rhs.width);

        let dividend = self.to_i64();
        let divisor = rhs.to_i64();
        if divisor == 0 {
            // return panic
            // put all-ones in the division result
            return RPanicResult {
                panic: RConcreteBitvector::from_unwrapped_u64(PANIC_NUM_DIV_BY_ZERO, 32),
                result: RConcreteBitvector::from_masked_u64(!0u64, self.width),
            };
        }
        let signed_minus_one = self.bit_mask_u64();
        let signed_minimum = self.sign_bit_mask_u64();
        if self.value == signed_minimum && rhs.value == signed_minus_one {
            // division result is dividend on overflow, no panic
            return RPanicResult {
                panic: RConcreteBitvector::from_unwrapped_u64(PANIC_NUM_NO_PANIC, 32),
                result: self,
            };
        }
        let result = dividend
            .checked_div(divisor)
            .map(|r| r as u64)
            .expect("Signed division should only return none on zero divisor or overflow");
        RPanicResult {
            panic: RConcreteBitvector::from_unwrapped_u64(PANIC_NUM_NO_PANIC, 32),
            result: RConcreteBitvector::from_masked_u64(result, self.width),
        }
    }

    pub fn hw_srem(self, rhs: Self) -> RPanicResult<Self> {
        assert_eq!(self.width, rhs.width);

        let dividend = self.to_i64();
        let divisor = rhs.to_i64();
        if divisor == 0 {
            // return panic
            // put the dividend in the remainder result
            return RPanicResult {
                panic: RConcreteBitvector::from_unwrapped_u64(PANIC_NUM_REM_BY_ZERO, 32),
                result: self,
            };
        }
        let signed_minus_one = self.bit_mask_u64();
        let signed_minimum = self.sign_bit_mask_u64();
        if self.value == signed_minimum && rhs.value == signed_minus_one {
            // remainder result is zero on overflow, no panic
            return RPanicResult {
                panic: RConcreteBitvector::from_unwrapped_u64(PANIC_NUM_NO_PANIC, 32),
                result: RConcreteBitvector::from_unwrapped_u64(0, self.width),
            };
        }
        // result after division overflow is zero
        let result = dividend
            .checked_rem(divisor)
            .expect("Signed remainder should only return none on zero divisor or overflow");
        RPanicResult {
            panic: RConcreteBitvector::from_unwrapped_u64(PANIC_NUM_NO_PANIC, 32),
            result: RConcreteBitvector::from_masked_u64(result as u64, self.width),
        }
    }

    pub(crate) fn checked_add(self, rhs: Self) -> Option<Self> {
        assert_eq!(self.width, rhs.width);

        let result = self.value.checked_add(rhs.value)?;
        if result & !self.bit_mask_u64() != 0 {
            return None;
        }
        Some(Self::from_unwrapped_u64(result, self.width))
    }

    pub(crate) fn checked_mul(self, rhs: Self) -> Option<Self> {
        let result = self.value.checked_mul(rhs.value)?;
        if result & !self.bit_mask_u64() != 0 {
            return None;
        }
        Some(Self::from_unwrapped_u64(result, self.width))
    }
}

impl<const W: u32> HwArith for ConcreteBitvector<W> {
    type DivRemResult = PanicResult<Self>;

    fn arith_neg(self) -> Self {
        self.to_runtime().arith_neg().unwrap_typed()
    }

    fn add(self, rhs: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.hw_add(rhs).unwrap_typed()
    }

    fn sub(self, rhs: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.hw_sub(rhs).unwrap_typed()
    }

    fn mul(self, rhs: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.hw_mul(rhs).unwrap_typed()
    }

    fn udiv(self, rhs: Self) -> PanicResult<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.hw_udiv(rhs).unwrap_typed()
    }

    fn urem(self, rhs: Self) -> PanicResult<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.hw_urem(rhs).unwrap_typed()
    }

    fn sdiv(self, rhs: Self) -> PanicResult<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.hw_sdiv(rhs).unwrap_typed()
    }

    fn srem(self, rhs: Self) -> PanicResult<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.hw_srem(rhs).unwrap_typed()
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

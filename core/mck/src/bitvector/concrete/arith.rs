use machine_check_common::{PANIC_NUM_DIV_BY_ZERO, PANIC_NUM_NO_PANIC, PANIC_NUM_REM_BY_ZERO};

use crate::{concr::PanicResult, forward::HwArith};

use super::ConcreteBitvector;

impl<const L: u32> HwArith for ConcreteBitvector<L> {
    type DivRemResult = PanicResult<Self>;

    fn arith_neg(self) -> Self {
        let result = self.0.wrapping_neg();
        Self::new(result & Self::bit_mask().0)
    }

    fn add(self, rhs: Self) -> Self {
        let result = self.0.wrapping_add(rhs.0);
        Self::new(result & Self::bit_mask().0)
    }

    fn sub(self, rhs: Self) -> Self {
        let result = self.0.wrapping_sub(rhs.0);
        Self::new(result & Self::bit_mask().0)
    }

    fn mul(self, rhs: Self) -> Self {
        let result = self.0.wrapping_mul(rhs.0);
        Self::new(result & Self::bit_mask().0)
    }

    fn udiv(self, rhs: Self) -> PanicResult<Self> {
        let dividend = self.as_unsigned();
        let divisor = rhs.as_unsigned();
        if divisor == 0 {
            // return panic
            // put all-ones in the division result
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_DIV_BY_ZERO),
                result: Self::bit_mask(),
            };
        }
        let result = dividend
            .checked_div(divisor)
            .expect("Unsigned division should only return none on zero divisor");
        PanicResult {
            panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC),
            result: Self::new(result & Self::bit_mask().0),
        }
    }

    fn urem(self, rhs: Self) -> PanicResult<Self> {
        let dividend = self.as_unsigned();
        let divisor = rhs.as_unsigned();
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
            result: Self::new(result & Self::bit_mask().0),
        }
    }

    fn sdiv(self, rhs: Self) -> PanicResult<Self> {
        let dividend = self.as_signed();
        let divisor = rhs.as_signed();
        if divisor == 0 {
            // return panic
            // put all-ones in the division result
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_DIV_BY_ZERO),
                result: Self::bit_mask(),
            };
        }
        let signed_minus_one = Self::bit_mask();
        let signed_minimum = Self::sign_bit_mask();
        if self == signed_minimum && rhs == signed_minus_one {
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
            result: Self::new(result & Self::bit_mask().0),
        }
    }

    fn srem(self, rhs: Self) -> PanicResult<Self> {
        let dividend = self.as_signed();
        let divisor = rhs.as_signed();
        if divisor == 0 {
            // return panic
            // put the dividend in the remainder result
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_REM_BY_ZERO),
                result: self,
            };
        }
        let signed_minus_one = Self::bit_mask();
        let signed_minimum = Self::sign_bit_mask();
        if self == signed_minimum && rhs == signed_minus_one {
            // remainder result is zero on overflow, no panic
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC),
                result: Self::new(0),
            };
        }
        // result after division overflow is zero
        let result = dividend
            .checked_rem(divisor)
            .expect("Signed remainder should only return none on zero divisor or overflow");
        PanicResult {
            panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC),
            result: Self::new(result as u64 & Self::bit_mask().0),
        }
    }
}

impl<const L: u32> ConcreteBitvector<L> {
    pub(crate) fn checked_add(self, rhs: Self) -> Option<Self> {
        let result = self.0.checked_add(rhs.0)?;
        if result & !Self::bit_mask().0 != 0 {
            return None;
        }
        Some(Self::new(result))
    }

    pub(crate) fn checked_mul(self, rhs: Self) -> Option<Self> {
        let result = self.0.checked_mul(rhs.0)?;
        if result & !Self::bit_mask().0 != 0 {
            return None;
        }
        Some(Self::new(result))
    }
}

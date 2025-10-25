use crate::{
    bitvector::bound::BitvectorBound,
    concr::PanicResult,
    forward::HwArith,
    misc::CBound,
    panic::message::{PANIC_NUM_DIV_BY_ZERO, PANIC_NUM_NO_PANIC, PANIC_NUM_REM_BY_ZERO},
};

use super::ConcreteBitvector;

impl<B: BitvectorBound> HwArith for ConcreteBitvector<B> {
    type DivRemResult = PanicResult<Self>;

    fn arith_neg(self) -> Self {
        let result = self.value.wrapping_neg();
        Self::from_masked_u64(result, self.bound)
    }

    fn add(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let result = self.value.wrapping_add(rhs.value);
        Self::from_masked_u64(result, self.bound)
    }

    fn sub(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let result = self.value.wrapping_sub(rhs.value);
        Self::from_masked_u64(result, self.bound)
    }

    fn mul(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let result = self.value.wrapping_mul(rhs.value);
        Self::from_masked_u64(result, self.bound)
    }

    fn udiv(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.bound, rhs.bound);

        let dividend = self.to_u64();
        let divisor = rhs.to_u64();
        if divisor == 0 {
            // return panic
            // put all-ones in the division result
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_DIV_BY_ZERO, CBound),
                result: Self::from_masked_u64(!0u64, self.bound),
            };
        }
        let result = dividend
            .checked_div(divisor)
            .expect("Unsigned division should only return none on zero divisor");
        PanicResult {
            panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC, CBound),
            result: Self::from_masked_u64(result, self.bound),
        }
    }

    fn urem(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.bound, rhs.bound);

        let dividend = self.to_u64();
        let divisor = rhs.to_u64();
        if divisor == 0 {
            // return panic
            // put the dividend in the remainder result
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_REM_BY_ZERO, CBound),
                result: self,
            };
        }
        let result = dividend
            .checked_rem(divisor)
            .expect("Unsigned remainder should only return none on zero divisor");
        PanicResult {
            panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC, CBound),
            result: Self::from_masked_u64(result, self.bound),
        }
    }

    fn sdiv(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.bound, rhs.bound);

        let dividend = self.to_i64();
        let divisor = rhs.to_i64();
        if divisor == 0 {
            // return panic
            // put all-ones in the division result
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_DIV_BY_ZERO, CBound),
                result: Self::from_masked_u64(!0u64, self.bound),
            };
        }
        let signed_minus_one = self.bound.mask();
        let signed_minimum = self.bound.sign_bit_mask();
        if self.value == signed_minimum && rhs.value == signed_minus_one {
            // division result is dividend on overflow, no panic
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC, CBound),
                result: self,
            };
        }
        let result = dividend
            .checked_div(divisor)
            .map(|r| r as u64)
            .expect("Signed division should only return none on zero divisor or overflow");
        PanicResult {
            panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC, CBound),
            result: Self::from_masked_u64(result, self.bound),
        }
    }

    fn srem(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.bound, rhs.bound);

        let dividend = self.to_i64();
        let divisor = rhs.to_i64();
        if divisor == 0 {
            // return panic
            // put the dividend in the remainder result
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_REM_BY_ZERO, CBound),
                result: self,
            };
        }
        let signed_minus_one = self.bound.mask();
        let signed_minimum = self.bound.sign_bit_mask();
        if self.value == signed_minimum && rhs.value == signed_minus_one {
            // remainder result is zero on overflow, no panic
            return PanicResult {
                panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC, CBound),
                result: Self::new(0, self.bound),
            };
        }
        // result after division overflow is zero
        let result = dividend
            .checked_rem(divisor)
            .expect("Signed remainder should only return none on zero divisor or overflow");
        PanicResult {
            panic: ConcreteBitvector::new(PANIC_NUM_NO_PANIC, CBound),
            result: Self::from_masked_u64(result as u64, self.bound),
        }
    }
}

impl<B: BitvectorBound> ConcreteBitvector<B> {
    pub(crate) fn checked_add(self, rhs: Self) -> Option<Self> {
        assert_eq!(self.bound, rhs.bound);

        let result = self.value.checked_add(rhs.value)?;
        if result & !self.bound.mask() != 0 {
            return None;
        }
        Some(Self::new(result, self.bound))
    }

    pub(crate) fn checked_mul(self, rhs: Self) -> Option<Self> {
        let result = self.value.checked_mul(rhs.value)?;
        if result & !self.bound.mask() != 0 {
            return None;
        }
        Some(Self::new(result, self.bound))
    }
}

/*pub fn unwrap_typed<const W: u32>(
    value: PanicResult<RConcreteBitvector>,
) -> PanicResult<ConcreteBitvector<W>> {
    PanicResult {
        panic: value.panic,
        result: value.result.unwrap_typed(),
    }
}*/

use crate::forward::HwArith;

use super::ConcreteBitvector;

impl<const L: u32> HwArith for ConcreteBitvector<L> {
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

    fn udiv(self, rhs: Self) -> Self {
        let dividend = self.as_unsigned();
        let divisor = rhs.as_unsigned();
        if divisor == 0 {
            // result of division by zero is all-ones
            return Self::bit_mask();
        }
        let result = dividend
            .checked_div(divisor)
            .expect("Unsigned division should only return none on zero divisor");
        Self::new(result & Self::bit_mask().0)
    }

    fn urem(self, rhs: Self) -> Self {
        let dividend = self.as_unsigned();
        let divisor = rhs.as_unsigned();
        if divisor == 0 {
            // result of division by zero is the dividend
            return self;
        }
        let result = dividend
            .checked_rem(divisor)
            .expect("Unsigned remainder should only return none on zero divisor");
        Self::new(result & Self::bit_mask().0)
    }

    fn sdiv(self, rhs: Self) -> Self {
        let dividend = self.as_signed();
        let divisor = rhs.as_signed();
        if divisor == 0 {
            // result of division by zero is all-ones
            return Self::bit_mask();
        }
        let signed_minus_one = Self::bit_mask();
        let signed_minimum = Self::sign_bit_mask();
        if self == signed_minimum && rhs == signed_minus_one {
            // result of overflow is dividend
            return self;
        }

        // result of overflow is dividend
        let result = dividend
            .checked_div(divisor)
            .map(|r| r as u64)
            .expect("Signed division should only return none on zero divisor or overflow");
        Self::new(result & Self::bit_mask().0)
    }

    fn srem(self, rhs: Self) -> Self {
        let dividend = self.as_signed();
        let divisor = rhs.as_signed();
        if divisor == 0 {
            // result of zero divisor is the dividend
            return self;
        }
        let signed_minus_one = Self::bit_mask();
        let signed_minimum = Self::sign_bit_mask();
        if self == signed_minimum && rhs == signed_minus_one {
            // result of overflow is zero
            return Self::new(0);
        }
        // result after division overflow is zero
        let result = dividend
            .checked_rem(divisor)
            .expect("Signed remainder should only return none on zero divisor or overflow");
        Self::new(result as u64 & Self::bit_mask().0)
    }
}

impl<const L: u32> ConcreteBitvector<L> {
    pub(crate) fn widening_mul(self, rhs: Self) -> (Self, Self) {
        let wide_lhs = self.0 as u128;
        let wide_rhs = rhs.0 as u128;

        let wide_result = wide_lhs.wrapping_mul(wide_rhs);
        let low_result = (wide_result as u64) & Self::bit_mask().0;
        let high_result = ((wide_result >> L) as u64) & Self::bit_mask().0;
        (Self::new(low_result), Self::new(high_result))
    }

    pub(crate) fn checked_add(self, rhs: Self) -> Option<Self> {
        let Some(result) = self.0.checked_add(rhs.0) else {
            return None;
        };
        if result & !Self::bit_mask().0 != 0 {
            return None;
        }
        Some(Self::new(result))
    }

    pub(crate) fn checked_mul(self, rhs: Self) -> Option<Self> {
        let Some(result) = self.0.checked_mul(rhs.0) else {
            return None;
        };
        if result & !Self::bit_mask().0 != 0 {
            return None;
        }
        Some(Self::new(result))
    }
}

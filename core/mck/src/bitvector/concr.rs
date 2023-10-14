use std::fmt::Debug;
use std::fmt::Display;

use crate::forward::Bitwise;
use crate::forward::Ext;
use crate::forward::HwArith;
use crate::forward::HwShift;
use crate::forward::TypedCmp;
use crate::forward::TypedEq;

use super::util;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bitvector<const L: u32>(u64);

impl<const L: u32> Bitvector<L> {
    #[allow(dead_code)]
    pub fn new(value: u64) -> Self {
        let mask: u64 = Self::bit_mask().0;
        if (value & !mask) != 0 {
            panic!(
                "Machine bitvector value {} does not fit into {} bits",
                value, L
            );
        }

        Self(value)
    }

    // not for use where it may be replaced by abstraction
    pub fn as_unsigned(&self) -> u64 {
        self.0
    }

    pub fn as_signed(&self) -> i64 {
        let mut result = self.0;
        if self.bitand(Self::sign_bit_mask()).is_nonzero() {
            // add signed extension
            result |= !Self::bit_mask().0;
        }
        result as i64
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn is_nonzero(&self) -> bool {
        self.0 != 0
    }

    pub fn is_full_mask(&self) -> bool {
        self == &Self::bit_mask()
    }

    pub fn is_sign_bit_set(&self) -> bool {
        util::is_u64_highest_bit_set(self.0, L)
    }

    pub fn sign_bit_mask() -> Bitvector<L> {
        Bitvector(util::compute_u64_sign_bit_mask(L))
    }

    pub fn bit_mask() -> Bitvector<L> {
        Bitvector(util::compute_u64_mask(L))
    }

    pub fn all_of_length_iter() -> impl Iterator<Item = Self> {
        (0..=Self::bit_mask().as_unsigned()).map(Self)
    }
}

impl<const L: u32> TypedEq for Bitvector<L> {
    type Output = Bitvector<1>;
    fn typed_eq(self, rhs: Self) -> Self::Output {
        let result = self.0 == rhs.0;
        Bitvector::<1>::new(result as u64)
    }
}

impl<const L: u32> TypedCmp for Bitvector<L> {
    type Output = Bitvector<1>;

    fn typed_slt(self, rhs: Self) -> Self::Output {
        let result = self.as_signed() < rhs.as_signed();
        Bitvector::<1>::new(result as u64)
    }

    fn typed_ult(self, rhs: Self) -> Self::Output {
        let result = self.as_unsigned() < rhs.as_unsigned();
        Bitvector::<1>::new(result as u64)
    }

    fn typed_slte(self, rhs: Self) -> Self::Output {
        let result = self.as_signed() <= rhs.as_signed();
        Bitvector::<1>::new(result as u64)
    }

    fn typed_ulte(self, rhs: Self) -> Self::Output {
        let result = self.as_unsigned() <= rhs.as_unsigned();
        Bitvector::<1>::new(result as u64)
    }
}

impl<const L: u32> Bitwise for Bitvector<L> {
    fn not(self) -> Self {
        Self::new((!self.0) & Self::bit_mask().0)
    }
    fn bitand(self, rhs: Self) -> Self {
        Self::new((self.0 & rhs.0) & Self::bit_mask().0)
    }
    fn bitor(self, rhs: Self) -> Self {
        Self::new((self.0 | rhs.0) & Self::bit_mask().0)
    }
    fn bitxor(self, rhs: Self) -> Self {
        Self::new((self.0 ^ rhs.0) & Self::bit_mask().0)
    }
}

impl<const L: u32> HwArith for Bitvector<L> {
    fn neg(self) -> Self {
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
            return rhs;
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
            return rhs;
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

impl<const L: u32> HwShift for Bitvector<L> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        if amount.0 >= L as u64 {
            // zero if the shift is too big
            Bitvector::new(0)
        } else {
            // apply mask after shifting
            let res = self.0 << (amount.0);
            Bitvector::new(res & Self::bit_mask().0)
        }
    }

    fn logic_shr(self, amount: Self) -> Self {
        if amount.0 >= L as u64 {
            // zero if the shift is too big
            Bitvector::new(0)
        } else {
            Bitvector::new(self.0 >> amount.0)
        }
    }

    fn arith_shr(self, amount: Self) -> Self {
        if amount.0 >= L as u64 {
            // fill with sign bit if the shift is too big
            if self.is_sign_bit_set() {
                return Bitvector::new(Self::bit_mask().0);
            }
            return Bitvector::new(0);
        };

        let mut result = self.0 >> amount.0;
        // copy sign bit if necessary
        if self.is_sign_bit_set() {
            let old_mask = Self::bit_mask().0;
            let new_mask = old_mask >> amount.0;
            let sign_bit_copy_mask = old_mask & !new_mask;
            result |= sign_bit_copy_mask;
        }
        Bitvector::new(result)
    }
}

impl<const L: u32, const X: u32> Ext<X> for Bitvector<L> {
    type Output = Bitvector<X>;

    fn uext(self) -> Self::Output {
        // shorten if needed, lengthening is fine
        Bitvector::<X>::new(self.0 & util::compute_u64_mask(X))
    }

    fn sext(self) -> Self::Output {
        // shorten if needed
        let mut v = self.0 & util::compute_u64_mask(X);
        // copy sign bit if necessary
        if self.is_sign_bit_set() {
            let old_mask = Self::bit_mask().0;
            let new_mask = util::compute_u64_mask(X);
            let lengthening_mask = !old_mask & new_mask;
            v |= lengthening_mask;
        }
        Bitvector::<X>::new(v)
    }
}

impl<const L: u32> Debug for Bitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'")?;
        for little_k in 0..L {
            let big_k = L - little_k - 1;
            let bit = (self.0 >> (big_k)) & 1;
            write!(f, "{}", bit)?;
        }
        write!(f, "'")
    }
}

impl<const L: u32> Display for Bitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

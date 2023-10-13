use std::num::Wrapping;

use std::fmt::Debug;
use std::fmt::Display;

use crate::bitvector::util::{compute_mask, compute_sign_bit_mask, is_highest_bit_set};
use crate::forward::Bitwise;
use crate::forward::Ext;
use crate::forward::HwArith;
use crate::forward::HwShift;
use crate::forward::TypedCmp;
use crate::forward::TypedEq;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bitvector<const L: u32> {
    v: Wrapping<u64>,
}

impl<const L: u32> Bitvector<L> {
    #[allow(dead_code)]
    pub fn new(value: u64) -> Self {
        Self::w_new(Wrapping(value))
    }

    pub fn w_new(value: Wrapping<u64>) -> Self {
        let mask = compute_mask(L);
        if (value & !mask) != Wrapping(0) {
            panic!(
                "Machine bitvector value {} does not fit into {} bits",
                value, L
            );
        }

        Self { v: value }
    }

    // not for use where it may be replaced by abstraction
    pub fn as_unsigned(&self) -> Wrapping<u64> {
        self.v
    }

    pub fn as_signed(&self) -> Wrapping<i64> {
        let mut result = self.v;
        if (result & compute_sign_bit_mask(L)) != Wrapping(0) {
            // add signed extension
            result |= !compute_mask(L);
        }
        Wrapping(result.0 as i64)
    }

    fn is_sign_bit_set(&self) -> bool {
        is_highest_bit_set(self.v, L)
    }
}

impl<const L: u32> Debug for Bitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'")?;
        for little_k in 0..L {
            let big_k = L - little_k - 1;
            let bit = (self.v >> (big_k as usize)) & Wrapping(1);
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

impl<const L: u32> Bitwise for Bitvector<L> {
    fn not(self) -> Self {
        Self::w_new((!self.v) & compute_mask(L))
    }
    fn bitand(self, rhs: Self) -> Self {
        Self::w_new((self.v & rhs.v) & compute_mask(L))
    }
    fn bitor(self, rhs: Self) -> Self {
        Self::w_new((self.v | rhs.v) & compute_mask(L))
    }
    fn bitxor(self, rhs: Self) -> Self {
        Self::w_new((self.v ^ rhs.v) & compute_mask(L))
    }
}

impl<const L: u32> TypedEq for Bitvector<L> {
    type Output = Bitvector<1>;
    fn typed_eq(self, rhs: Self) -> Self::Output {
        let result = self.v == rhs.v;
        Bitvector::<1>::w_new(Wrapping(result as u64))
    }
}

impl<const L: u32> TypedCmp for Bitvector<L> {
    type Output = Bitvector<1>;

    fn typed_slt(self, rhs: Self) -> Self::Output {
        let result = self.as_signed() < rhs.as_signed();
        Bitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_ult(self, rhs: Self) -> Self::Output {
        let result = self.as_unsigned() < rhs.as_unsigned();
        Bitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_slte(self, rhs: Self) -> Self::Output {
        let result = self.as_signed() <= rhs.as_signed();
        Bitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_ulte(self, rhs: Self) -> Self::Output {
        let result = self.as_unsigned() <= rhs.as_unsigned();
        Bitvector::<1>::w_new(Wrapping(result as u64))
    }
}

impl<const L: u32, const X: u32> Ext<X> for Bitvector<L> {
    type Output = Bitvector<X>;

    fn uext(self) -> Self::Output {
        // shorten if needed, lengthening is fine
        Bitvector::<X>::w_new(self.v & compute_mask(X))
    }

    fn sext(self) -> Self::Output {
        // shorten if needed
        let mut v = self.v & compute_mask(X);
        // copy sign bit if necessary
        if self.is_sign_bit_set() {
            let old_mask = compute_mask(L);
            let new_mask = compute_mask(X);
            let lengthening_mask = !old_mask & new_mask;
            v |= lengthening_mask;
        }
        Bitvector::<X>::w_new(v)
    }
}

impl<const L: u32> HwShift for Bitvector<L> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        if amount.v.0 >= L as u64 {
            // zero if the shift is too big
            Bitvector::w_new(Wrapping(0))
        } else {
            // apply mask after shifting
            let res = self.v << (amount.v.0 as usize);
            Bitvector::w_new(res & compute_mask(L))
        }
    }

    fn logic_shr(self, amount: Self) -> Self {
        if amount.v.0 >= L as u64 {
            // zero if the shift is too big
            Bitvector::w_new(Wrapping(0))
        } else {
            Bitvector::w_new(self.v >> amount.v.0 as usize)
        }
    }

    fn arith_shr(self, amount: Self) -> Self {
        if amount.v.0 >= L as u64 {
            // fill with sign bit if the shift is too big
            if self.is_sign_bit_set() {
                return Bitvector::w_new(compute_mask(L));
            }
            return Bitvector::w_new(Wrapping(0));
        };

        let mut result = self.v >> amount.v.0 as usize;
        // copy sign bit if necessary
        if self.is_sign_bit_set() {
            let old_mask = compute_mask(L);
            let new_mask = old_mask >> amount.v.0 as usize;
            let sign_bit_copy_mask = old_mask & !new_mask;
            result |= sign_bit_copy_mask;
        }
        Bitvector::w_new(result)
    }
}

impl<const L: u32> HwArith for Bitvector<L> {
    fn neg(self) -> Self {
        Self::w_new((-self.v) & compute_mask(L))
    }

    fn add(self, rhs: Self) -> Self {
        Self::w_new((self.v + rhs.v) & compute_mask(L))
    }

    fn sub(self, rhs: Self) -> Self {
        Self::w_new((self.v - rhs.v) & compute_mask(L))
    }

    fn mul(self, rhs: Self) -> Self {
        Self::w_new((self.v * rhs.v) & compute_mask(L))
    }

    fn sdiv(self, rhs: Self) -> Self {
        // result of division by zero is the mask
        let dividend = self.as_signed().0;
        let divisor = rhs.as_signed().0;
        let result = dividend
            .checked_div(divisor)
            .map(|r| r as u64)
            .unwrap_or(compute_mask(L).0);
        Self::w_new(Wrapping(result) & compute_mask(L))
    }

    fn udiv(self, rhs: Self) -> Self {
        // result of division by zero is the mask
        // see https://github.com/Boolector/btor2tools/blob/037f1fa88fb439dca6f648ad48a3463256d69d8b/src/btorsim/btorsimbv.c#L1819

        let dividend = self.as_unsigned().0;
        let divisor = rhs.as_unsigned().0;
        let result = dividend.checked_div(divisor).unwrap_or(compute_mask(L).0);
        Self::w_new(Wrapping(result) & compute_mask(L))
    }

    fn smod(self, rhs: Self) -> Self {
        // result of modulo (Euclidean remainder) by zero is the dividend

        let dividend = self.as_signed().0;
        let divisor = rhs.as_signed().0;
        let result = dividend.checked_rem_euclid(divisor).unwrap_or(dividend);
        Self::w_new(Wrapping(result as u64) & compute_mask(L))
    }

    fn seuc(self, rhs: Self) -> Self {
        // result of remainder by zero is the dividend

        let dividend = self.as_signed().0;
        let divisor = rhs.as_signed().0;
        let result = dividend.checked_rem(divisor).unwrap_or(dividend);
        Self::w_new(Wrapping(result as u64) & compute_mask(L))
    }

    fn urem(self, rhs: Self) -> Self {
        // result of division by zero is the dividend
        // see https://github.com/Boolector/btor2tools/blob/037f1fa88fb439dca6f648ad48a3463256d69d8b/src/btorsim/btorsimbv.c#L1818

        let dividend = self.as_unsigned().0;
        let divisor = rhs.as_unsigned().0;
        let result = dividend.checked_rem(divisor).unwrap_or(dividend);
        Self::w_new(Wrapping(result) & compute_mask(L))
    }
}

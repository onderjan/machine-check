use std::{
    num::Wrapping,
    ops::{Add, BitAnd, BitOr, BitXor, Mul, Neg, Not, Sub},
};

use crate::{
    traits::{MachineExt, MachineShift, TypedCmp, TypedEq},
    util::{compute_mask, compute_sign_bit_mask, is_sign_bit_set},
};

use super::Bitvector;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MachineBitvector<const L: u32> {
    v: Wrapping<u64>,
}

impl<const L: u32> Bitvector<L> for MachineBitvector<L> {
    fn new(value: u64) -> Self {
        Self::w_new(Wrapping(value))
    }
}

impl<const L: u32> MachineBitvector<L> {
    #[allow(dead_code)]
    pub fn new(value: u64) -> Self {
        <MachineBitvector<L> as Bitvector<L>>::new(value)
    }

    fn w_new(value: Wrapping<u64>) -> Self {
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
    pub fn concrete_unsigned(&self) -> Wrapping<u64> {
        self.v
    }

    pub fn concrete_signed(&self) -> Wrapping<i64> {
        let mut result = self.v;
        if (result & compute_sign_bit_mask(L)) != Wrapping(0) {
            // add signed extension
            result |= !compute_mask(L);
        }
        Wrapping(result.0 as i64)
    }

    fn is_sign_bit_set(&self) -> bool {
        is_sign_bit_set(self.v, L)
    }
}

impl<const L: u32> Neg for MachineBitvector<L> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::w_new((-self.v) & compute_mask(L))
    }
}

impl<const L: u32> Add for MachineBitvector<L> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::w_new((self.v + rhs.v) & compute_mask(L))
    }
}

impl<const L: u32> Sub for MachineBitvector<L> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::w_new((self.v - rhs.v) & compute_mask(L))
    }
}

impl<const L: u32> Mul for MachineBitvector<L> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::w_new((self.v * rhs.v) & compute_mask(L))
    }
}

impl<const L: u32> Not for MachineBitvector<L> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::w_new((!self.v) & compute_mask(L))
    }
}

impl<const L: u32> BitAnd for MachineBitvector<L> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::w_new((self.v & rhs.v) & compute_mask(L))
    }
}

impl<const L: u32> BitOr for MachineBitvector<L> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::w_new((self.v | rhs.v) & compute_mask(L))
    }
}

impl<const L: u32> BitXor for MachineBitvector<L> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::w_new((self.v ^ rhs.v) & compute_mask(L))
    }
}

impl<const L: u32> TypedEq for MachineBitvector<L> {
    type Output = MachineBitvector<1>;
    fn typed_eq(self, rhs: Self) -> Self::Output {
        let result = self.v == rhs.v;
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }
}

impl<const L: u32> TypedCmp for MachineBitvector<L> {
    type Output = MachineBitvector<1>;

    fn typed_slt(self, rhs: Self) -> Self::Output {
        let result = self.concrete_signed() < rhs.concrete_signed();
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_ult(self, rhs: Self) -> Self::Output {
        let result = self.concrete_unsigned() < rhs.concrete_unsigned();
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_slte(self, rhs: Self) -> Self::Output {
        let result = self.concrete_signed() <= rhs.concrete_signed();
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_ulte(self, rhs: Self) -> Self::Output {
        let result = self.concrete_unsigned() <= rhs.concrete_unsigned();
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }
}

impl<const L: u32, const X: u32> MachineExt<X> for MachineBitvector<L> {
    type Output = MachineBitvector<X>;

    fn uext(self) -> Self::Output {
        // shorten if needed, lengthening is fine
        MachineBitvector::<X>::w_new(self.v & compute_mask(X))
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
        MachineBitvector::<X>::w_new(v)
    }
}

impl<const L: u32> MachineShift for MachineBitvector<L> {
    type Output = Self;

    fn sll(self, amount: Self) -> Self {
        if amount.v.0 >= L as u64 {
            // zero if the shift is too big
            MachineBitvector::w_new(Wrapping(0))
        } else {
            // apply mask after shifting
            let res = self.v << (amount.v.0 as usize);
            MachineBitvector::w_new(res & compute_mask(L))
        }
    }

    fn srl(self, amount: Self) -> Self {
        if amount.v.0 >= L as u64 {
            // zero if the shift is too big
            MachineBitvector::w_new(Wrapping(0))
        } else {
            MachineBitvector::w_new(self.v >> amount.v.0 as usize)
        }
    }

    fn sra(self, amount: Self) -> Self {
        if amount.v.0 >= L as u64 {
            // fill with sign bit if the shift is too big
            if self.is_sign_bit_set() {
                return MachineBitvector::w_new(compute_mask(L));
            }
            return MachineBitvector::w_new(Wrapping(0));
        };

        let mut result = self.v >> amount.v.0 as usize;
        // copy sign bit if necessary
        if self.is_sign_bit_set() {
            let old_mask = compute_mask(L);
            let new_mask = old_mask >> amount.v.0 as usize;
            let sign_bit_copy_mask = !old_mask & new_mask;
            result |= sign_bit_copy_mask;
        }
        MachineBitvector::w_new(result)
    }
}

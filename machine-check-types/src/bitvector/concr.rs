use std::{
    num::Wrapping,
    ops::{Add, BitAnd, BitOr, BitXor, Mul, Neg, Not, Sub},
};

use crate::traits::{Sext, Sll, Sra, Srl, TypedCmp, TypedEq, Uext};

#[derive(Debug, Clone, Copy)]
pub struct MachineBitvector<const L: u32> {
    v: Wrapping<u64>,
}

impl<const L: u32> MachineBitvector<L> {
    pub fn value(&self) -> Wrapping<u64> {
        self.v
    }
}

const fn compute_mask(n: u32) -> Wrapping<u64> {
    if n == u64::BITS {
        return Wrapping(0u64.wrapping_sub(1u64));
    }
    let num_values = u64::checked_shl(1u64, n);
    if let Some(num_values) = num_values {
        Wrapping(num_values.wrapping_sub(1u64))
    } else {
        panic!("Too many bits for MachineU")
    }
}

impl<const L: u32> MachineBitvector<L> {
    fn w_new(value: Wrapping<u64>) -> Self {
        let mask = compute_mask(L);
        if (value & !mask) != Wrapping(0) {
            panic!("MachineU value {} does not fit into {} bits", value, L);
        }

        //println!("New {}-bitvector (mask {}): {}", N, mask, value);

        MachineBitvector { v: value }
    }

    pub fn new(value: u64) -> Self {
        Self::w_new(Wrapping(value))
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

impl<const L: u32> PartialEq for MachineBitvector<L> {
    fn eq(&self, other: &Self) -> bool {
        self.v == other.v
    }
}

impl<const L: u32> Eq for MachineBitvector<L> {}

impl<const L: u32> TypedEq for MachineBitvector<L> {
    type Output = MachineBitvector<1>;
    fn typed_eq(self, rhs: Self) -> Self::Output {
        let result = self == rhs;
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }
}

impl<const L: u32> TypedCmp for MachineBitvector<L> {
    type Output = MachineBitvector<1>;

    fn typed_sgt(self, rhs: Self) -> Self::Output {
        let result = self.v.0 as i64 > rhs.v.0 as i64;
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_ugt(self, rhs: Self) -> Self::Output {
        let result = self.v.0 > rhs.v.0;
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_sgte(self, rhs: Self) -> Self::Output {
        let result = self.v.0 as i64 >= rhs.v.0 as i64;
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_ugte(self, rhs: Self) -> Self::Output {
        let result = self.v.0 >= rhs.v.0;
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_slt(self, rhs: Self) -> Self::Output {
        let result = (self.v.0 as i64) < (rhs.v.0 as i64);
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_ult(self, rhs: Self) -> Self::Output {
        let result = (self.v.0) < (rhs.v.0);
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_slte(self, rhs: Self) -> Self::Output {
        let result = (self.v.0) as i64 <= (rhs.v.0 as i64);
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }

    fn typed_ulte(self, rhs: Self) -> Self::Output {
        let result = (self.v.0) <= (rhs.v.0);
        MachineBitvector::<1>::w_new(Wrapping(result as u64))
    }
}

impl<const L: u32, const X: u32> Uext<X> for MachineBitvector<L> {
    type Output = MachineBitvector<X>;

    fn uext(self) -> Self::Output {
        // only shorten if needed
        MachineBitvector::<X>::w_new(self.v & compute_mask(X))
    }
}

impl<const L: u32, const X: u32> Sext<X> for MachineBitvector<L> {
    type Output = MachineBitvector<X>;

    fn sext(self) -> Self::Output {
        // shorten if needed
        let mut v = self.v & compute_mask(X);
        // copy sign bit where necessary
        if X > L {
            let num_sign_extend = X - L;
            let sign_masked = self.v & (Wrapping(1u64) << (L - 1) as usize);
            for i in 1..num_sign_extend + 1 {
                v |= sign_masked << i as usize;
            }
        }

        MachineBitvector::<X>::w_new(v)
    }
}

impl<const L: u32> Sll for MachineBitvector<L> {
    type Output = Self;

    fn sll(self, amount: Self) -> Self {
        if amount.v.0 >= L as u64 {
            // zero if the shift is too big
            MachineBitvector::w_new(Wrapping(0))
        } else {
            MachineBitvector::w_new(self.v << amount.v.0 as usize)
        }
    }
}

impl<const L: u32> Srl for MachineBitvector<L> {
    type Output = Self;

    fn srl(self, amount: Self) -> Self {
        if amount.v.0 >= L as u64 {
            // zero if the shift is too big
            MachineBitvector::w_new(Wrapping(0))
        } else {
            MachineBitvector::w_new(self.v >> amount.v.0 as usize)
        }
    }
}

impl<const L: u32> Sra for MachineBitvector<L> {
    type Output = Self;

    fn sra(self, amount: Self) -> Self {
        let sign_masked = self.v & (Wrapping(1u64) << (L - 1) as usize);
        if amount.v.0 >= L as u64 {
            // fill with sign bit if the shift is too big
            if sign_masked != Wrapping(0) {
                MachineBitvector::w_new(compute_mask(L))
            } else {
                MachineBitvector::w_new(Wrapping(0))
            }
        } else {
            // copy sign bit where necessary
            let mut v = self.v >> amount.v.0 as usize;
            for i in 0..amount.v.0 {
                v |= sign_masked >> i as usize;
            }

            MachineBitvector::w_new(v)
        }
    }
}

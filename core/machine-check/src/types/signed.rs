use std::{
    fmt::Debug,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub},
};

use mck::{
    concr::{self, IntoMck},
    forward::{Bitwise, HwArith, HwShift},
};

use crate::{traits::Ext, Bitvector, Unsigned};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Signed<const L: u32>(pub(super) concr::Bitvector<L>);

impl<const L: u32> Signed<L> {
    pub fn new(value: u64) -> Self {
        Signed(concr::Bitvector::new(value))
    }
}

impl<const L: u32> IntoMck for Signed<L> {
    type Type = mck::concr::Bitvector<L>;

    fn into_mck(self) -> Self::Type {
        self.0
    }
}

// --- BITWISE OPERATIONS ---

impl<const L: u32> Not for Signed<L> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(self.0.bit_not())
    }
}

impl<const L: u32> BitAnd<Signed<L>> for Signed<L> {
    type Output = Self;

    fn bitand(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.bit_and(rhs.0))
    }
}
impl<const L: u32> BitOr<Signed<L>> for Signed<L> {
    type Output = Self;

    fn bitor(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.bit_or(rhs.0))
    }
}
impl<const L: u32> BitXor<Signed<L>> for Signed<L> {
    type Output = Self;

    fn bitxor(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.bit_xor(rhs.0))
    }
}

// --- ARITHMETIC OPERATIONS ---

impl<const L: u32> Add<Signed<L>> for Signed<L> {
    type Output = Self;

    fn add(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<const L: u32> Sub<Signed<L>> for Signed<L> {
    type Output = Self;

    fn sub(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<const L: u32> Mul<Signed<L>> for Signed<L> {
    type Output = Self;

    fn mul(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

impl<const L: u32> Div<Signed<L>> for Signed<L> {
    type Output = Self;

    fn div(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.sdiv(rhs.0))
    }
}

impl<const L: u32> Rem<Signed<L>> for Signed<L> {
    type Output = Self;

    fn rem(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.srem(rhs.0))
    }
}

impl<const L: u32> Shl<Signed<L>> for Signed<L> {
    type Output = Self;

    fn shl(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.logic_shl(rhs.0))
    }
}

impl<const L: u32> Shr<Signed<L>> for Signed<L> {
    type Output = Self;

    fn shr(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.arith_shr(rhs.0))
    }
}

// --- EXTENSION ---
impl<const L: u32, const X: u32> Ext<X> for Signed<L> {
    type Output = Signed<X>;

    fn ext(self) -> Self::Output {
        Signed::<X>(mck::forward::Ext::sext(self.0))
    }
}

// --- ORDERING ---

impl<const L: u32> PartialOrd for Signed<L> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const L: u32> Ord for Signed<L> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.signed_cmp(&other.0)
    }
}

// --- CONVERSION ---
impl<const L: u32> From<Unsigned<L>> for Signed<L> {
    fn from(value: Unsigned<L>) -> Self {
        Self(value.0)
    }
}

impl<const L: u32> From<Bitvector<L>> for Signed<L> {
    fn from(value: Bitvector<L>) -> Self {
        Self(value.0)
    }
}

// --- MISC ---

impl<const L: u32> Debug for Signed<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

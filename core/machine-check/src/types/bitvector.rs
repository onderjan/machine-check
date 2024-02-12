use std::{
    fmt::Debug,
    ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Shl, Sub},
};

use mck::{
    concr::{self, IntoMck},
    forward::{Bitwise, HwArith, HwShift},
};

use crate::{Signed, Unsigned};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Bitvector<const L: u32>(pub(super) concr::Bitvector<L>);

impl<const L: u32> Bitvector<L> {
    pub fn new(value: u64) -> Self {
        Bitvector(concr::Bitvector::new(value))
    }
}

impl<const L: u32> IntoMck for Bitvector<L> {
    type Type = mck::concr::Bitvector<L>;

    fn into_mck(self) -> Self::Type {
        self.0
    }
}

// --- BITWISE OPERATIONS ---

impl<const L: u32> Not for Bitvector<L> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(self.0.bit_not())
    }
}

impl<const L: u32> BitAnd<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    fn bitand(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.bit_and(rhs.0))
    }
}
impl<const L: u32> BitOr<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    fn bitor(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.bit_or(rhs.0))
    }
}
impl<const L: u32> BitXor<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    fn bitxor(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.bit_xor(rhs.0))
    }
}

// --- ARITHMETIC OPERATIONS ---

impl<const L: u32> Add<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    fn add(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<const L: u32> Sub<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    fn sub(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<const L: u32> Mul<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    fn mul(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

// --- SHIFT ---
impl<const L: u32> Shl<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    fn shl(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.logic_shl(rhs.0))
    }
}

// --- CONVERSION ---
impl<const L: u32> From<Unsigned<L>> for Bitvector<L> {
    fn from(value: Unsigned<L>) -> Self {
        Self(value.0)
    }
}

impl<const L: u32> From<Signed<L>> for Bitvector<L> {
    fn from(value: Signed<L>) -> Self {
        Self(value.0)
    }
}

impl<const L: u32> Debug for Bitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

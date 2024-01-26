use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Rem, Shl, Shr, Sub},
};

use mck::{
    concr,
    forward::{HwArith, HwShift},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Unsigned<const L: u32>(pub(super) concr::Bitvector<L>);

impl<const L: u32> Unsigned<L> {
    pub fn new(value: u64) -> Self {
        Unsigned(concr::Bitvector::new(value))
    }
}

impl<const L: u32> Add<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn add(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<const L: u32> Sub<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn sub(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<const L: u32> Mul<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn mul(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

impl<const L: u32> Div<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn div(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.udiv(rhs.0))
    }
}

impl<const L: u32> Rem<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn rem(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.urem(rhs.0))
    }
}

impl<const L: u32> Shl<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn shl(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.logic_shl(rhs.0))
    }
}

impl<const L: u32> Shr<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn shr(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.logic_shr(rhs.0))
    }
}

impl<const L: u32> PartialOrd for Unsigned<L> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const L: u32> Ord for Unsigned<L> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.unsigned_cmp(&other.0)
    }
}

impl<const L: u32> Debug for Unsigned<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

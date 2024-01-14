use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Rem, Shl, Shr, Sub},
};

use crate::{
    bitvector::concr,
    forward::{HwArith, HwShift},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Unsigned<const L: u32>(concr::Bitvector<L>);

impl<const L: u32> Unsigned<L> {
    pub fn new(value: u64) -> Self {
        Unsigned(concr::Bitvector::new(value))
    }

    pub fn zero() -> Self {
        Unsigned(concr::Bitvector::new(0))
    }

    pub fn one() -> Self {
        Unsigned(concr::Bitvector::new(1))
    }

    pub fn from_bitvector(bitvector: concr::Bitvector<L>) -> Self {
        Unsigned(bitvector)
    }

    pub fn as_bitvector(&self) -> concr::Bitvector<L> {
        self.0
    }
}

impl<const L: u32> Add<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn add(self, rhs: Unsigned<L>) -> Self::Output {
        Self::from_bitvector(self.0.add(rhs.0))
    }
}

impl<const L: u32> Sub<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn sub(self, rhs: Unsigned<L>) -> Self::Output {
        Self::from_bitvector(self.0.sub(rhs.0))
    }
}

impl<const L: u32> Mul<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn mul(self, rhs: Unsigned<L>) -> Self::Output {
        Self::from_bitvector(self.0.mul(rhs.0))
    }
}

impl<const L: u32> Div<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn div(self, rhs: Unsigned<L>) -> Self::Output {
        Self::from_bitvector(self.0.udiv(rhs.0))
    }
}

impl<const L: u32> Rem<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn rem(self, rhs: Unsigned<L>) -> Self::Output {
        Self::from_bitvector(self.0.urem(rhs.0))
    }
}

impl<const L: u32> Shl<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn shl(self, rhs: Unsigned<L>) -> Self::Output {
        Self::from_bitvector(self.0.logic_shl(rhs.0))
    }
}

impl<const L: u32> Shr<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    fn shr(self, rhs: Unsigned<L>) -> Self::Output {
        Self::from_bitvector(self.0.logic_shr(rhs.0))
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
        write!(f, "{}", self.0)
    }
}

impl<const L: u32> Display for Unsigned<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

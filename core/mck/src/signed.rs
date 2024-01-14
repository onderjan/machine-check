use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Rem, Shl, Shr, Sub},
};

use crate::{
    bitvector::concr,
    forward::{HwArith, HwShift},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Signed<const L: u32>(concr::Bitvector<L>);

impl<const L: u32> Signed<L> {
    pub fn new(value: u64) -> Self {
        Signed(concr::Bitvector::new(value))
    }

    pub fn zero() -> Self {
        Signed(concr::Bitvector::new(0))
    }

    pub fn one() -> Self {
        Signed(concr::Bitvector::new(1))
    }

    pub fn from_bitvector(bitvector: concr::Bitvector<L>) -> Self {
        Signed(bitvector)
    }

    pub fn as_bitvector(&self) -> concr::Bitvector<L> {
        self.0
    }
}

impl<const L: u32> Add<Signed<L>> for Signed<L> {
    type Output = Self;

    fn add(self, rhs: Signed<L>) -> Self::Output {
        Self::from_bitvector(self.0.add(rhs.0))
    }
}

impl<const L: u32> Sub<Signed<L>> for Signed<L> {
    type Output = Self;

    fn sub(self, rhs: Signed<L>) -> Self::Output {
        Self::from_bitvector(self.0.sub(rhs.0))
    }
}

impl<const L: u32> Mul<Signed<L>> for Signed<L> {
    type Output = Self;

    fn mul(self, rhs: Signed<L>) -> Self::Output {
        Self::from_bitvector(self.0.mul(rhs.0))
    }
}

impl<const L: u32> Div<Signed<L>> for Signed<L> {
    type Output = Self;

    fn div(self, rhs: Signed<L>) -> Self::Output {
        Self::from_bitvector(self.0.sdiv(rhs.0))
    }
}

impl<const L: u32> Rem<Signed<L>> for Signed<L> {
    type Output = Self;

    fn rem(self, rhs: Signed<L>) -> Self::Output {
        Self::from_bitvector(self.0.srem(rhs.0))
    }
}

impl<const L: u32> Shl<Signed<L>> for Signed<L> {
    type Output = Self;

    fn shl(self, rhs: Signed<L>) -> Self::Output {
        Self::from_bitvector(self.0.logic_shl(rhs.0))
    }
}

impl<const L: u32> Shr<Signed<L>> for Signed<L> {
    type Output = Self;

    fn shr(self, rhs: Signed<L>) -> Self::Output {
        Self::from_bitvector(self.0.arith_shr(rhs.0))
    }
}

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

impl<const L: u32> Debug for Signed<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<const L: u32> Display for Signed<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

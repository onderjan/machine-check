use std::{
    fmt::Debug,
    ops::{Add, Mul, Shl, Sub},
};

use mck::{
    concr,
    forward::{HwArith, HwShift},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Bitvector<const L: u32>(concr::Bitvector<L>);

impl<const L: u32> Bitvector<L> {
    pub fn new(value: u64) -> Self {
        Bitvector(concr::Bitvector::new(value))
    }
}

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

impl<const L: u32> Shl<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    fn shl(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.logic_shl(rhs.0))
    }
}

impl<const L: u32> Debug for Bitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

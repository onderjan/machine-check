use std::ops::{Add, Sub};

use crate::{bitvector::concr, forward::HwArith};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Unsigned<const L: u32>(concr::Bitvector<L>);

impl<const L: u32> Unsigned<L> {
    pub fn new(value: u64) -> Self {
        Unsigned(concr::Bitvector::new(value))
    }

    pub fn from_bitvector(bitvector: concr::Bitvector<L>) -> Self {
        Unsigned(bitvector)
    }

    pub fn as_bitvector(&self) -> concr::Bitvector<L> {
        self.0
    }
}

impl<const L: u32> Add<Unsigned<L>> for Unsigned<L> {
    type Output = Unsigned<L>;

    fn add(self, rhs: Unsigned<L>) -> Self::Output {
        Self::from_bitvector(self.0.add(rhs.0))
    }
}

impl<const L: u32> Sub<Unsigned<L>> for Unsigned<L> {
    type Output = Unsigned<L>;

    fn sub(self, rhs: Unsigned<L>) -> Self::Output {
        Self::from_bitvector(self.0.sub(rhs.0))
    }
}

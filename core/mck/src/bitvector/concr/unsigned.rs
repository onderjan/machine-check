use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Rem, Shl, Shr, Sub},
};

use num::{One, Zero};

use crate::{
    concr::PanicResult,
    forward::{Ext, HwArith, HwShift},
};

use super::ConcreteBitvector;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct UnsignedBitvector<const L: u32>(ConcreteBitvector<L>);

impl<const L: u32> UnsignedBitvector<L> {
    pub fn new(value: u64) -> Self {
        UnsignedBitvector(ConcreteBitvector::new(value))
    }

    pub fn zero() -> Self {
        UnsignedBitvector(ConcreteBitvector::new(0))
    }

    pub fn one() -> Self {
        UnsignedBitvector(ConcreteBitvector::new(1))
    }

    pub(super) const fn from_bitvector(bitvector: ConcreteBitvector<L>) -> Self {
        UnsignedBitvector(bitvector)
    }

    pub fn as_bitvector(self) -> ConcreteBitvector<L> {
        self.0
    }

    pub fn to_u64(self) -> u64 {
        self.0.to_u64()
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_nonzero(&self) -> bool {
        self.0.is_nonzero()
    }

    pub fn ext<const X: u32>(self) -> UnsignedBitvector<X> {
        UnsignedBitvector(self.0.uext())
    }
}

impl<const L: u32> Add<UnsignedBitvector<L>> for UnsignedBitvector<L> {
    type Output = Self;

    fn add(self, rhs: UnsignedBitvector<L>) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<const L: u32> Sub<UnsignedBitvector<L>> for UnsignedBitvector<L> {
    type Output = Self;

    fn sub(self, rhs: UnsignedBitvector<L>) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<const L: u32> Mul<UnsignedBitvector<L>> for UnsignedBitvector<L> {
    type Output = Self;

    fn mul(self, rhs: UnsignedBitvector<L>) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

impl<const L: u32> Div<UnsignedBitvector<L>> for UnsignedBitvector<L> {
    type Output = PanicResult<Self>;

    fn div(self, rhs: UnsignedBitvector<L>) -> PanicResult<Self> {
        // unsigned division
        let panic_result = self.0.udiv(rhs.0);
        PanicResult {
            panic: panic_result.panic,
            result: Self(panic_result.result),
        }
    }
}

impl<const L: u32> Rem<UnsignedBitvector<L>> for UnsignedBitvector<L> {
    type Output = PanicResult<Self>;

    fn rem(self, rhs: UnsignedBitvector<L>) -> PanicResult<Self> {
        // unsigned remainder
        let panic_result = self.0.urem(rhs.0);
        PanicResult {
            panic: panic_result.panic,
            result: Self(panic_result.result),
        }
    }
}

impl<const L: u32> Shl<UnsignedBitvector<L>> for UnsignedBitvector<L> {
    type Output = Self;

    fn shl(self, rhs: UnsignedBitvector<L>) -> Self::Output {
        // both signed and unsigned use logic shift left
        Self(self.0.logic_shl(rhs.0))
    }
}

impl<const L: u32> Shr<UnsignedBitvector<L>> for UnsignedBitvector<L> {
    type Output = Self;

    fn shr(self, rhs: UnsignedBitvector<L>) -> Self::Output {
        // signed uses arithmetic shift right
        Self(self.0.logic_shr(rhs.0))
    }
}

impl<const L: u32> PartialOrd for UnsignedBitvector<L> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const L: u32> Ord for UnsignedBitvector<L> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // unsigned comparison
        self.0.unsigned_cmp(&other.0)
    }
}

impl<const L: u32> Zero for UnsignedBitvector<L> {
    fn zero() -> Self {
        UnsignedBitvector(ConcreteBitvector::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<const L: u32> One for UnsignedBitvector<L> {
    fn one() -> Self {
        UnsignedBitvector(ConcreteBitvector::one())
    }
}

impl<const L: u32> Debug for UnsignedBitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_u64())
    }
}

impl<const L: u32> Display for UnsignedBitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_u64())
    }
}

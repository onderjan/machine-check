use std::{
    fmt::{Debug, Display},
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub},
};

use serde::{Deserialize, Serialize};

use crate::{
    bitvector::BitvectorBound,
    concr::{OutsideBound, PanicResult},
    forward::{BExt, Bitwise, HwArith, HwShift},
};

use super::ConcreteBitvector;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsignedBitvector<B: BitvectorBound>(ConcreteBitvector<B>);

impl<B: BitvectorBound> UnsignedBitvector<B> {
    pub fn new(value: u64, bound: B) -> Self {
        UnsignedBitvector(ConcreteBitvector::new(value, bound))
    }

    pub fn try_new(value: u64, bound: B) -> Result<Self, OutsideBound<u64>> {
        ConcreteBitvector::try_new(value, bound).map(UnsignedBitvector)
    }

    pub fn bound(&self) -> B {
        self.0.bound
    }

    pub fn zero(bound: B) -> Self {
        UnsignedBitvector(ConcreteBitvector::new(0, bound))
    }

    pub fn one(bound: B) -> Self {
        UnsignedBitvector(ConcreteBitvector::new(1, bound))
    }

    pub(super) const fn from_bitvector(bitvector: ConcreteBitvector<B>) -> Self {
        UnsignedBitvector(bitvector)
    }

    pub fn cast_bitvector(self) -> ConcreteBitvector<B> {
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

    pub fn ext<X: BitvectorBound>(self, new_bound: X) -> UnsignedBitvector<X> {
        UnsignedBitvector(self.0.uext(new_bound))
    }
}

impl<B: BitvectorBound> Add<UnsignedBitvector<B>> for UnsignedBitvector<B> {
    type Output = Self;

    fn add(self, rhs: UnsignedBitvector<B>) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<B: BitvectorBound> Sub<UnsignedBitvector<B>> for UnsignedBitvector<B> {
    type Output = Self;

    fn sub(self, rhs: UnsignedBitvector<B>) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<B: BitvectorBound> Mul<UnsignedBitvector<B>> for UnsignedBitvector<B> {
    type Output = Self;

    fn mul(self, rhs: UnsignedBitvector<B>) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

impl<B: BitvectorBound> Div<UnsignedBitvector<B>> for UnsignedBitvector<B> {
    type Output = PanicResult<Self>;

    fn div(self, rhs: UnsignedBitvector<B>) -> PanicResult<Self> {
        // unsigned division
        let panic_result = self.0.udiv(rhs.0);
        PanicResult {
            panic: panic_result.panic,
            result: Self(panic_result.result),
        }
    }
}

impl<B: BitvectorBound> Rem<UnsignedBitvector<B>> for UnsignedBitvector<B> {
    type Output = PanicResult<Self>;

    fn rem(self, rhs: UnsignedBitvector<B>) -> PanicResult<Self> {
        // unsigned remainder
        let panic_result = self.0.urem(rhs.0);
        PanicResult {
            panic: panic_result.panic,
            result: Self(panic_result.result),
        }
    }
}

impl<B: BitvectorBound> Not for UnsignedBitvector<B> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(self.0.bit_not())
    }
}

impl<B: BitvectorBound> BitAnd for UnsignedBitvector<B> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0.bit_and(rhs.0))
    }
}

impl<B: BitvectorBound> BitOr for UnsignedBitvector<B> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0.bit_or(rhs.0))
    }
}

impl<B: BitvectorBound> BitXor for UnsignedBitvector<B> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0.bit_xor(rhs.0))
    }
}

impl<B: BitvectorBound> Shl<UnsignedBitvector<B>> for UnsignedBitvector<B> {
    type Output = Self;

    fn shl(self, rhs: UnsignedBitvector<B>) -> Self::Output {
        // both signed and unsigned use logic shift left
        Self(self.0.logic_shl(rhs.0))
    }
}

impl<B: BitvectorBound> Shr<UnsignedBitvector<B>> for UnsignedBitvector<B> {
    type Output = Self;

    fn shr(self, rhs: UnsignedBitvector<B>) -> Self::Output {
        // signed uses arithmetic shift right
        Self(self.0.logic_shr(rhs.0))
    }
}

impl<B: BitvectorBound> PartialOrd for UnsignedBitvector<B> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<B: BitvectorBound> Ord for UnsignedBitvector<B> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // unsigned comparison
        self.0.unsigned_cmp(&other.0)
    }
}

impl<B: BitvectorBound> Debug for UnsignedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.to_u64(), f)
    }
}

impl<B: BitvectorBound> Display for UnsignedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.to_u64(), f)
    }
}

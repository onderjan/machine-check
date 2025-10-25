use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Rem, Shl, Shr, Sub},
};

use crate::{
    bitvector::BitvectorBound,
    concr::PanicResult,
    forward::{BExt, HwArith, HwShift},
};

use super::ConcreteBitvector;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SignedBitvector<B: BitvectorBound>(ConcreteBitvector<B>);

impl<B: BitvectorBound> SignedBitvector<B> {
    pub fn new(value: u64, bound: B) -> Self {
        SignedBitvector(ConcreteBitvector::new(value, bound))
    }

    fn zero(bound: B) -> Self {
        SignedBitvector(ConcreteBitvector::zero(bound))
    }

    fn one(bound: B) -> Self {
        SignedBitvector(ConcreteBitvector::one(bound))
    }

    pub(super) const fn from_bitvector(bitvector: ConcreteBitvector<B>) -> Self {
        SignedBitvector(bitvector)
    }

    pub fn cast_bitvector(&self) -> ConcreteBitvector<B> {
        self.0
    }

    pub fn bound(&self) -> B {
        self.0.bound
    }

    pub fn to_i64(self) -> i64 {
        self.0.to_i64()
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_nonzero(&self) -> bool {
        self.0.is_nonzero()
    }

    pub fn ext<X: BitvectorBound>(self, new_bound: X) -> SignedBitvector<X> {
        SignedBitvector(self.0.sext(new_bound))
    }
}

impl<B: BitvectorBound> Neg for SignedBitvector<B> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0.arith_neg())
    }
}

impl<B: BitvectorBound> Add<SignedBitvector<B>> for SignedBitvector<B> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<B: BitvectorBound> Sub<SignedBitvector<B>> for SignedBitvector<B> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<B: BitvectorBound> Mul<SignedBitvector<B>> for SignedBitvector<B> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

impl<B: BitvectorBound> Div<SignedBitvector<B>> for SignedBitvector<B> {
    type Output = PanicResult<Self>;

    fn div(self, rhs: Self) -> PanicResult<Self> {
        // signed division
        let panic_result = self.0.sdiv(rhs.0);
        PanicResult {
            panic: panic_result.panic,
            result: Self(panic_result.result),
        }
    }
}

impl<B: BitvectorBound> Rem<SignedBitvector<B>> for SignedBitvector<B> {
    type Output = PanicResult<Self>;

    fn rem(self, rhs: Self) -> PanicResult<Self> {
        // signed remainder
        let panic_result = self.0.srem(rhs.0);
        PanicResult {
            panic: panic_result.panic,
            result: Self(panic_result.result),
        }
    }
}

impl<B: BitvectorBound> Shl<SignedBitvector<B>> for SignedBitvector<B> {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        // both signed and unsigned use logic shift left
        Self(self.0.logic_shl(rhs.0))
    }
}

impl<B: BitvectorBound> Shr<SignedBitvector<B>> for SignedBitvector<B> {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        // signed uses arithmetic shift right
        Self(self.0.arith_shr(rhs.0))
    }
}

impl<B: BitvectorBound> PartialOrd for SignedBitvector<B> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<B: BitvectorBound> Ord for SignedBitvector<B> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // signed comparison
        self.0.signed_cmp(&other.0)
    }
}

impl<B: BitvectorBound> Debug for SignedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_i64())
    }
}

impl<B: BitvectorBound> Display for SignedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_i64())
    }
}

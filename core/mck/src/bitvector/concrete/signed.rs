use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Rem, Shl, Shr, Sub},
};

use num::{One, Zero};

use crate::{
    bitvector::concr,
    concr::PanicResult,
    forward::{HwArith, HwShift},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SignedBitvector<const L: u32>(concr::Bitvector<L>);

impl<const L: u32> SignedBitvector<L> {
    pub fn new(value: u64) -> Self {
        SignedBitvector(concr::Bitvector::new(value))
    }

    pub fn zero() -> Self {
        SignedBitvector(concr::Bitvector::new(0))
    }

    pub fn one() -> Self {
        SignedBitvector(concr::Bitvector::new(1))
    }

    pub(super) fn from_bitvector(bitvector: concr::Bitvector<L>) -> Self {
        SignedBitvector(bitvector)
    }

    pub fn as_bitvector(&self) -> concr::Bitvector<L> {
        self.0
    }

    pub fn to_i64(self) -> i64 {
        self.0.as_signed()
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_nonzero(&self) -> bool {
        self.0.is_nonzero()
    }
}

impl<const L: u32> Neg for SignedBitvector<L> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0.arith_neg())
    }
}

impl<const L: u32> Add<SignedBitvector<L>> for SignedBitvector<L> {
    type Output = Self;

    fn add(self, rhs: SignedBitvector<L>) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<const L: u32> Sub<SignedBitvector<L>> for SignedBitvector<L> {
    type Output = Self;

    fn sub(self, rhs: SignedBitvector<L>) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<const L: u32> Mul<SignedBitvector<L>> for SignedBitvector<L> {
    type Output = Self;

    fn mul(self, rhs: SignedBitvector<L>) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

impl<const L: u32> Div<SignedBitvector<L>> for SignedBitvector<L> {
    type Output = PanicResult<Self>;

    fn div(self, rhs: SignedBitvector<L>) -> PanicResult<Self> {
        // signed division
        let panic_result = self.0.sdiv(rhs.0);
        PanicResult {
            panic: panic_result.panic,
            result: Self(panic_result.result),
        }
    }
}

impl<const L: u32> Rem<SignedBitvector<L>> for SignedBitvector<L> {
    type Output = PanicResult<Self>;

    fn rem(self, rhs: SignedBitvector<L>) -> PanicResult<Self> {
        // signed remainder
        let panic_result = self.0.srem(rhs.0);
        PanicResult {
            panic: panic_result.panic,
            result: Self(panic_result.result),
        }
    }
}

impl<const L: u32> Shl<SignedBitvector<L>> for SignedBitvector<L> {
    type Output = Self;

    fn shl(self, rhs: SignedBitvector<L>) -> Self::Output {
        // both signed and unsigned use logic shift left
        Self(self.0.logic_shl(rhs.0))
    }
}

impl<const L: u32> Shr<SignedBitvector<L>> for SignedBitvector<L> {
    type Output = Self;

    fn shr(self, rhs: SignedBitvector<L>) -> Self::Output {
        // signed uses arithmetic shift right
        Self(self.0.arith_shr(rhs.0))
    }
}

impl<const L: u32> PartialOrd for SignedBitvector<L> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const L: u32> Ord for SignedBitvector<L> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // signed comparison
        self.0.signed_cmp(&other.0)
    }
}

impl<const L: u32> Zero for SignedBitvector<L> {
    fn zero() -> Self {
        SignedBitvector(concr::Bitvector::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<const L: u32> One for SignedBitvector<L> {
    fn one() -> Self {
        SignedBitvector(concr::Bitvector::one())
    }
}

impl<const L: u32> Debug for SignedBitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<const L: u32> Display for SignedBitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

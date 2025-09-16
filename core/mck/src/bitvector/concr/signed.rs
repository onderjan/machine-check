use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Rem, Shl, Shr, Sub},
};

use num::{One, Zero};

use crate::{
    concr::{PanicResult, RConcreteBitvector},
    forward::{Ext, HwArith, HwShift},
};

use super::ConcreteBitvector;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct RSignedBitvector(RConcreteBitvector);

impl RSignedBitvector {
    pub(crate) fn new(value: u64, width: u32) -> Self {
        Self::from_bitvector(RConcreteBitvector::new(value, width))
    }

    pub(crate) const fn from_bitvector(bitvector: RConcreteBitvector) -> Self {
        RSignedBitvector(bitvector)
    }

    pub fn as_bitvector(self) -> RConcreteBitvector {
        self.0
    }

    pub fn to_i64(self) -> i64 {
        self.0.to_i64()
    }
}

impl PartialOrd for RSignedBitvector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RSignedBitvector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // unsigned comparison
        self.0.signed_cmp(&other.0)
    }
}

impl Neg for RSignedBitvector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0.arith_neg())
    }
}

impl Add<RSignedBitvector> for RSignedBitvector {
    type Output = Self;

    fn add(self, rhs: RSignedBitvector) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl Sub<RSignedBitvector> for RSignedBitvector {
    type Output = Self;

    fn sub(self, rhs: RSignedBitvector) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl Mul<RSignedBitvector> for RSignedBitvector {
    type Output = Self;

    fn mul(self, rhs: RSignedBitvector) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

impl Div<RSignedBitvector> for RSignedBitvector {
    type Output = PanicResult<Self>;

    fn div(self, rhs: RSignedBitvector) -> PanicResult<Self> {
        // signed division
        let panic_result = self.0.sdiv(rhs.0);
        PanicResult {
            panic: panic_result.panic,
            result: Self(panic_result.result),
        }
    }
}

impl Rem<RSignedBitvector> for RSignedBitvector {
    type Output = PanicResult<Self>;

    fn rem(self, rhs: RSignedBitvector) -> PanicResult<Self> {
        // signed remainder
        let panic_result = self.0.srem(rhs.0);
        PanicResult {
            panic: panic_result.panic,
            result: Self(panic_result.result),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SignedBitvector<const W: u32>(ConcreteBitvector<W>);

impl<const W: u32> SignedBitvector<W> {
    pub fn new(value: u64) -> Self {
        SignedBitvector(ConcreteBitvector::new(value))
    }

    pub fn zero() -> Self {
        SignedBitvector(ConcreteBitvector::new(0))
    }

    pub fn one() -> Self {
        SignedBitvector(ConcreteBitvector::new(1))
    }

    pub(super) const fn from_bitvector(bitvector: ConcreteBitvector<W>) -> Self {
        SignedBitvector(bitvector)
    }

    pub fn as_bitvector(&self) -> ConcreteBitvector<W> {
        self.0
    }

    pub fn to_i64(self) -> i64 {
        self.0.to_i64()
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_nonzero(&self) -> bool {
        self.0.is_nonzero()
    }

    pub fn ext<const X: u32>(self) -> SignedBitvector<X> {
        SignedBitvector(self.0.sext())
    }
}

impl<const W: u32> Neg for SignedBitvector<W> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0.arith_neg())
    }
}

impl<const W: u32> Add<SignedBitvector<W>> for SignedBitvector<W> {
    type Output = Self;

    fn add(self, rhs: SignedBitvector<W>) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<const W: u32> Sub<SignedBitvector<W>> for SignedBitvector<W> {
    type Output = Self;

    fn sub(self, rhs: SignedBitvector<W>) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<const W: u32> Mul<SignedBitvector<W>> for SignedBitvector<W> {
    type Output = Self;

    fn mul(self, rhs: SignedBitvector<W>) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

impl<const W: u32> Div<SignedBitvector<W>> for SignedBitvector<W> {
    type Output = PanicResult<Self>;

    fn div(self, rhs: SignedBitvector<W>) -> PanicResult<Self> {
        // signed division
        let panic_result = self.0.sdiv(rhs.0);
        PanicResult {
            panic: panic_result.panic,
            result: Self(panic_result.result),
        }
    }
}

impl<const W: u32> Rem<SignedBitvector<W>> for SignedBitvector<W> {
    type Output = PanicResult<Self>;

    fn rem(self, rhs: SignedBitvector<W>) -> PanicResult<Self> {
        // signed remainder
        let panic_result = self.0.srem(rhs.0);
        PanicResult {
            panic: panic_result.panic,
            result: Self(panic_result.result),
        }
    }
}

impl<const W: u32> Shl<SignedBitvector<W>> for SignedBitvector<W> {
    type Output = Self;

    fn shl(self, rhs: SignedBitvector<W>) -> Self::Output {
        // both signed and unsigned use logic shift left
        Self(self.0.logic_shl(rhs.0))
    }
}

impl<const W: u32> Shr<SignedBitvector<W>> for SignedBitvector<W> {
    type Output = Self;

    fn shr(self, rhs: SignedBitvector<W>) -> Self::Output {
        // signed uses arithmetic shift right
        Self(self.0.arith_shr(rhs.0))
    }
}

impl<const W: u32> PartialOrd for SignedBitvector<W> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const W: u32> Ord for SignedBitvector<W> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // signed comparison
        self.0.signed_cmp(&other.0)
    }
}

impl<const W: u32> Zero for SignedBitvector<W> {
    fn zero() -> Self {
        SignedBitvector(ConcreteBitvector::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<const W: u32> One for SignedBitvector<W> {
    fn one() -> Self {
        SignedBitvector(ConcreteBitvector::one())
    }
}

impl<const W: u32> Debug for SignedBitvector<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_i64())
    }
}

impl<const W: u32> Display for SignedBitvector<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_i64())
    }
}

use std::{
    fmt::{Debug, Display},
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub},
};

use crate::{
    bitvector::BitvectorBound,
    concr::{OutsideBound, PanicResult},
    forward::{BExt, Bitwise, HwArith, HwShift},
};

use super::ConcreteBitvector;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SignedBitvector<B: BitvectorBound>(ConcreteBitvector<B>);

impl<B: BitvectorBound> SignedBitvector<B> {
    pub fn new(value: i64, bound: B) -> Self {
        match Self::try_new(value, bound) {
            Ok(ok) => ok,
            Err(err) => panic!("{}", err),
        }
    }

    pub fn try_new(value: i64, bound: B) -> Result<Self, OutsideBound<i64>> {
        // test that the value is within bounds
        let max_value = (bound.mask() ^ bound.sign_bit_mask()) as i64;
        let min_value = (!bound.mask() ^ bound.sign_bit_mask()) as i64;

        if value < min_value || value > max_value {
            return Err(OutsideBound {
                width: bound.width(),
                value,
                min_value,
                max_value,
            });
        }

        let bounded_value = (value as u64) & bound.mask();
        Ok(SignedBitvector(ConcreteBitvector::new(
            bounded_value,
            bound,
        )))
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

impl<B: BitvectorBound> Not for SignedBitvector<B> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(self.0.bit_not())
    }
}

impl<B: BitvectorBound> BitAnd for SignedBitvector<B> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0.bit_and(rhs.0))
    }
}

impl<B: BitvectorBound> BitOr for SignedBitvector<B> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0.bit_or(rhs.0))
    }
}

impl<B: BitvectorBound> BitXor for SignedBitvector<B> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0.bit_xor(rhs.0))
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

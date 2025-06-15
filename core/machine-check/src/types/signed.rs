use std::{
    fmt::Debug,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub},
};

use machine_check_common::{PANIC_MSG_DIV_BY_ZERO, PANIC_MSG_REM_BY_ZERO};
use mck::{
    concr::{self, IntoMck},
    forward::{Bitwise, HwArith, HwShift},
};

use crate::{traits::Ext, Bitvector, Unsigned};

/// Signed bitvector.
///
/// The number of bits is specified in the generic parameter L.
/// Signed bitvectors support bitwise operations and wrapping-arithmetic operations.
/// Arithmetic bit extension is also possible (the sign bit is copied into any bits above it).
/// Signed bitvectors be converted into [`Unsigned`] or [`Bitvector`].
///
/// Currently, it is not possible to create signed bitvectors directly, only convert into them.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Signed<const L: u32>(pub(super) concr::Bitvector<L>);

// --- BITWISE OPERATIONS ---

impl<const L: u32> Not for Signed<L> {
    type Output = Self;

    /// Performs bitwise NOT.
    fn not(self) -> Self::Output {
        Self(self.0.bit_not())
    }
}

impl<const L: u32> BitAnd<Signed<L>> for Signed<L> {
    type Output = Self;

    /// Performs bitwise AND.
    fn bitand(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.bit_and(rhs.0))
    }
}
impl<const L: u32> BitOr<Signed<L>> for Signed<L> {
    type Output = Self;

    /// Performs bitwise OR.
    fn bitor(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.bit_or(rhs.0))
    }
}
impl<const L: u32> BitXor<Signed<L>> for Signed<L> {
    type Output = Self;

    /// Performs bitwise XOR.
    fn bitxor(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.bit_xor(rhs.0))
    }
}

// --- ARITHMETIC OPERATIONS ---

impl<const L: u32> Add<Signed<L>> for Signed<L> {
    type Output = Self;

    /// Performs wrapping addition.
    fn add(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<const L: u32> Sub<Signed<L>> for Signed<L> {
    type Output = Self;

    /// Performs wrapping subtraction.
    fn sub(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<const L: u32> Mul<Signed<L>> for Signed<L> {
    type Output = Self;

    /// Performs wrapping multiplication.
    fn mul(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

impl<const L: u32> Div<Signed<L>> for Signed<L> {
    type Output = Self;

    /// Performs wrapping signed division.
    ///
    /// This behaves as Rust `wrapping_div`, where the result of `MIN / -1`,
    /// which would be unrepresentable `-MIN`, is wrapped to `MIN`.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero.
    fn div(self, rhs: Signed<L>) -> Self::Output {
        let panic_result = self.0.sdiv(rhs.0);
        if panic_result.panic.is_nonzero() {
            panic!("{}", PANIC_MSG_DIV_BY_ZERO)
        }
        Self(panic_result.result)
    }
}

impl<const L: u32> Rem<Signed<L>> for Signed<L> {
    type Output = Self;

    /// Performs wrapping signed division.
    ///
    /// This behaves as Rust `wrapping_rem`, where the result of `MIN % -1`,
    /// is defined as 0.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero.
    fn rem(self, rhs: Signed<L>) -> Self::Output {
        let panic_result = self.0.srem(rhs.0);
        if panic_result.panic.is_nonzero() {
            panic!("{}", PANIC_MSG_REM_BY_ZERO)
        }
        Self(panic_result.result)
    }
}

impl<const L: u32> Shl<Signed<L>> for Signed<L> {
    type Output = Self;

    /// Performs a left shift.
    ///
    /// Unlike a right shift, where the behaviour is dependent on signedness,
    /// the left shift has the same behaviour: shifted-out bits on the left
    /// are discarded and zeros are shifted in on the right.
    ///
    /// The right-hand side operand is interpreted as unsigned and if it
    /// is equal or greater to the bit-width, the result is all-zeros,
    /// as in Rust `unbounded_shl`. It is planned to restrict the bit-width
    /// in the future so that this edge case can never occur.
    ///
    /// Note that this means that shifting left with a negative right operand
    /// produces an all-zeros value.
    fn shl(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.logic_shl(rhs.0))
    }
}

impl<const L: u32> Shr<Signed<L>> for Signed<L> {
    type Output = Self;

    /// Performs an arithmetic right shift.
    ///
    /// The right-hand side operand is interpreted as unsigned and if it
    /// is equal or greater to the bit-width, the result is all-zeros or
    /// all-ones depending on the original sign bit, as in Rust `unbounded_shr`
    /// on signed primitives.
    /// It is planned to restrict the bit-width in the future so that this edge
    /// case can never occur.
    ///
    /// Note that this means that shifting right with a negative right operand
    /// produces an all-zeros or all-ones value, depending on the original sign bit.
    fn shr(self, rhs: Signed<L>) -> Self::Output {
        Self(self.0.arith_shr(rhs.0))
    }
}

// --- EXTENSION ---
impl<const L: u32, const X: u32> Ext<X> for Signed<L> {
    type Output = Signed<X>;

    /// Extends or narrows the bit-vector.
    ///
    /// If an extension is performed, the upper bits
    /// are the copy of the original sign bit.
    fn ext(self) -> Self::Output {
        Signed::<X>(mck::forward::Ext::sext(self.0))
    }
}

// --- ORDERING ---

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

// --- CONVERSION ---
impl<const L: u32> From<Unsigned<L>> for Signed<L> {
    /// Converts the signedness information from `Unsigned` to `Signed`.
    fn from(value: Unsigned<L>) -> Self {
        Self(value.0)
    }
}

impl<const L: u32> From<Bitvector<L>> for Signed<L> {
    /// Adds signedness information to `Bitvector`.
    fn from(value: Bitvector<L>) -> Self {
        Self(value.0)
    }
}

// --- MISC ---

impl<const L: u32> Debug for Signed<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

// --- INTERNAL IMPLEMENTATIONS ---

#[doc(hidden)]
impl<const L: u32> IntoMck for Signed<L> {
    type Type = mck::concr::Bitvector<L>;

    fn into_mck(self) -> Self::Type {
        self.0
    }
}

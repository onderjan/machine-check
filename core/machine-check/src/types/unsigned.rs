use std::{
    fmt::Debug,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub},
};

use mck::{
    concr::{self, IntoMck},
    forward::{Bitwise, HwArith, HwShift},
};

use crate::{traits::Ext, Bitvector, Signed};

///
/// Unsigned bitvector.
///
/// The number of bits is specified in the generic parameter L.
/// Unsigned bitvectors support bitwise operations and wrapping-arithmetic operations.
/// Logical bit extension is also possible (any new bits are zero).
/// Signed bitvectors be converted into [`Unsigned`] or [`Bitvector`].
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Unsigned<const L: u32>(pub(super) concr::Bitvector<L>);

impl<const L: u32> Unsigned<L> {
    ///
    /// Creates a new bitvector with the given value.
    /// Panics if the value does not fit into the type.
    ///
    pub fn new(value: u64) -> Self {
        Unsigned(concr::Bitvector::new(value))
    }
}
// --- BITWISE OPERATIONS ---

impl<const L: u32> Not for Unsigned<L> {
    type Output = Self;

    /// Performs bitwise NOT.
    fn not(self) -> Self::Output {
        Self(self.0.bit_not())
    }
}

impl<const L: u32> BitAnd<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    /// Performs bitwise AND.
    fn bitand(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.bit_and(rhs.0))
    }
}
impl<const L: u32> BitOr<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    /// Performs bitwise OR.
    fn bitor(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.bit_or(rhs.0))
    }
}
impl<const L: u32> BitXor<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    /// Performs bitwise XOR.
    fn bitxor(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.bit_xor(rhs.0))
    }
}

// --- ARITHMETIC OPERATIONS ---

impl<const L: u32> Add<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    /// Performs wrapping addition.
    fn add(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<const L: u32> Sub<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    /// Performs wrapping subtraction.
    fn sub(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<const L: u32> Mul<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    /// Performs wrapping multiplication.
    fn mul(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

impl<const L: u32> Div<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    /// Performs wrapping unsigned division.
    ///
    /// While the division is defined to be wrapping,
    /// no wrapping actually can happen in unsigned division.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero.
    fn div(self, rhs: Unsigned<L>) -> Self::Output {
        let panic_result = self.0.udiv(rhs.0);
        if panic_result.panic.is_nonzero() {
            panic!("attempt to divide by zero")
        }
        Self(panic_result.result)
    }
}

impl<const L: u32> Rem<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    /// Performs wrapping unsigned remainer.
    ///
    /// While the remainder is defined to be wrapping,
    /// no wrapping actually can happen in unsigned remainder.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero.
    fn rem(self, rhs: Unsigned<L>) -> Self::Output {
        let panic_result = self.0.urem(rhs.0);
        if panic_result.panic.is_nonzero() {
            panic!("attempt to calculate the remainder with a divisor of zero")
        }
        Self(panic_result.result)
    }
}

impl<const L: u32> Shl<Unsigned<L>> for Unsigned<L> {
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
    fn shl(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.logic_shl(rhs.0))
    }
}

impl<const L: u32> Shr<Unsigned<L>> for Unsigned<L> {
    type Output = Self;

    /// Performs a logic right shift.
    ///
    /// The right-hand side operand is interpreted as unsigned and if it
    /// is equal or greater to the bit-width, the result is all-zeros,
    /// as in Rust `unbounded_shr` on unsigned primitives.
    /// It is planned to restrict the bit-width in the future so that this edge
    /// case can never occur.
    fn shr(self, rhs: Unsigned<L>) -> Self::Output {
        Self(self.0.logic_shr(rhs.0))
    }
}

// --- EXTENSION ---
impl<const L: u32, const X: u32> Ext<X> for Unsigned<L> {
    type Output = Unsigned<X>;

    /// Extends or narrows the bit-vector.
    ///
    /// If an extension is performed, the upper bits will be zero.
    fn ext(self) -> Self::Output {
        Unsigned::<X>(mck::forward::Ext::uext(self.0))
    }
}

// --- ORDERING ---

impl<const L: u32> PartialOrd for Unsigned<L> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const L: u32> Ord for Unsigned<L> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.unsigned_cmp(&other.0)
    }
}

// --- CONVERSION ---

impl<const L: u32> From<Bitvector<L>> for Unsigned<L> {
    /// Adds signedness information to `Bitvector`.
    fn from(value: Bitvector<L>) -> Self {
        Self(value.0)
    }
}

impl<const L: u32> From<Signed<L>> for Unsigned<L> {
    /// Converts the signedness information from `Signed` to `Unsigned`.
    fn from(value: Signed<L>) -> Self {
        Self(value.0)
    }
}

// --- MISC ---

impl<const L: u32> Debug for Unsigned<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

// --- INTERNAL IMPLEMENTATIONS ---

#[doc(hidden)]
impl<const L: u32> IntoMck for Unsigned<L> {
    type Type = mck::concr::Bitvector<L>;

    fn into_mck(self) -> Self::Type {
        self.0
    }
}

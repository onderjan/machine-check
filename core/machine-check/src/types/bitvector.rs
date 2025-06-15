use std::{
    fmt::Debug,
    ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Shl, Sub},
};

use mck::{
    concr::{self, IntoMck},
    forward::{Bitwise, HwArith, HwShift},
};

use crate::{Signed, Unsigned};

/// Bitvector without signedness information.
///
/// The number of bits is specified in the generic parameter L.
/// Bitvectors support bitwise operations and wrapping-arithmetic operations.
/// Only operations where the behaviour of signed and unsigned numbers match are implemented.
/// For others, conversion into [`Unsigned`] or [`Signed`] is necessary.
/// Bit-extension is not possible directly, as signed and unsigned bitvectors are extended differently.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Bitvector<const L: u32>(pub(super) concr::Bitvector<L>);

impl<const L: u32> Bitvector<L> {
    /// Creates a new bitvector with the given value.
    ///
    /// Panics if the value does not fit into the type.
    pub fn new(value: u64) -> Self {
        Bitvector(concr::Bitvector::new(value))
    }
}
// --- BITWISE OPERATIONS ---

impl<const L: u32> Not for Bitvector<L> {
    type Output = Self;

    /// Performs bitwise NOT.
    fn not(self) -> Self::Output {
        Self(self.0.bit_not())
    }
}

impl<const L: u32> BitAnd<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    /// Performs bitwise AND.
    fn bitand(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.bit_and(rhs.0))
    }
}
impl<const L: u32> BitOr<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    /// Performs bitwise OR.
    fn bitor(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.bit_or(rhs.0))
    }
}
impl<const L: u32> BitXor<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    /// Performs bitwise XOR.
    fn bitxor(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.bit_xor(rhs.0))
    }
}

// --- ARITHMETIC OPERATIONS ---

impl<const L: u32> Add<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    /// Performs wrapping addition.
    fn add(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<const L: u32> Sub<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    /// Performs wrapping subtraction.
    fn sub(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<const L: u32> Mul<Bitvector<L>> for Bitvector<L> {
    type Output = Self;

    /// Performs wrapping multiplication.
    fn mul(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

// --- SHIFT ---
impl<const L: u32> Shl<Bitvector<L>> for Bitvector<L> {
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
    fn shl(self, rhs: Bitvector<L>) -> Self::Output {
        Self(self.0.logic_shl(rhs.0))
    }
}

// --- CONVERSION ---
impl<const L: u32> From<Unsigned<L>> for Bitvector<L> {
    /// Drops the signedness information from `Unsigned`.
    fn from(value: Unsigned<L>) -> Self {
        Self(value.0)
    }
}

impl<const L: u32> From<Signed<L>> for Bitvector<L> {
    /// Drops the signedness information from `Signed`.
    fn from(value: Signed<L>) -> Self {
        Self(value.0)
    }
}

impl<const L: u32> Debug for Bitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

// --- INTERNAL IMPLEMENTATIONS ---

#[doc(hidden)]
impl<const L: u32> IntoMck for Bitvector<L> {
    type Type = mck::concr::Bitvector<L>;

    fn into_mck(self) -> Self::Type {
        self.0
    }
}

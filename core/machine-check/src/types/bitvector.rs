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
/// The width (number of bits) is specified by the generic parameter W.
/// Bitvectors support bitwise operations and wrapping-arithmetic operations.
/// Only operations where the behaviour of signed and unsigned numbers match are implemented.
/// For others, conversion into [`Unsigned`] or [`Signed`] is necessary.
/// Bit-extension is not possible directly, as signed and unsigned bitvectors are extended differently.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Bitvector<const W: u32>(pub(super) concr::Bitvector<W>);

impl<const W: u32> Bitvector<W> {
    /// Creates a new bitvector with the given value.
    ///
    /// Panics if the value does not fit into the type.
    pub fn new(value: u64) -> Self {
        Bitvector(concr::Bitvector::new(value))
    }
}
// --- BITWISE OPERATIONS ---

impl<const W: u32> Not for Bitvector<W> {
    type Output = Self;

    /// Performs bitwise NOT.
    fn not(self) -> Self::Output {
        Self(self.0.bit_not())
    }
}

impl<const W: u32> BitAnd<Bitvector<W>> for Bitvector<W> {
    type Output = Self;

    /// Performs bitwise AND.
    fn bitand(self, rhs: Bitvector<W>) -> Self::Output {
        Self(self.0.bit_and(rhs.0))
    }
}
impl<const W: u32> BitOr<Bitvector<W>> for Bitvector<W> {
    type Output = Self;

    /// Performs bitwise OR.
    fn bitor(self, rhs: Bitvector<W>) -> Self::Output {
        Self(self.0.bit_or(rhs.0))
    }
}
impl<const W: u32> BitXor<Bitvector<W>> for Bitvector<W> {
    type Output = Self;

    /// Performs bitwise XOR.
    fn bitxor(self, rhs: Bitvector<W>) -> Self::Output {
        Self(self.0.bit_xor(rhs.0))
    }
}

// --- ARITHMETIC OPERATIONS ---

impl<const W: u32> Add<Bitvector<W>> for Bitvector<W> {
    type Output = Self;

    /// Performs wrapping addition.
    fn add(self, rhs: Bitvector<W>) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<const W: u32> Sub<Bitvector<W>> for Bitvector<W> {
    type Output = Self;

    /// Performs wrapping subtraction.
    fn sub(self, rhs: Bitvector<W>) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<const W: u32> Mul<Bitvector<W>> for Bitvector<W> {
    type Output = Self;

    /// Performs wrapping multiplication.
    fn mul(self, rhs: Bitvector<W>) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

// --- SHIFT ---
impl<const W: u32> Shl<Bitvector<W>> for Bitvector<W> {
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
    fn shl(self, rhs: Bitvector<W>) -> Self::Output {
        Self(self.0.logic_shl(rhs.0))
    }
}

// --- CONVERSION ---
impl<const W: u32> From<Unsigned<W>> for Bitvector<W> {
    /// Drops the signedness information from `Unsigned`.
    fn from(value: Unsigned<W>) -> Self {
        Self(value.0)
    }
}

impl<const W: u32> From<Signed<W>> for Bitvector<W> {
    /// Drops the signedness information from `Signed`.
    fn from(value: Signed<W>) -> Self {
        Self(value.0)
    }
}

impl<const W: u32> Debug for Bitvector<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

// --- INTERNAL IMPLEMENTATIONS ---

#[doc(hidden)]
impl<const W: u32> IntoMck for Bitvector<W> {
    type Type = mck::concr::Bitvector<W>;

    fn into_mck(self) -> Self::Type {
        self.0
    }
}

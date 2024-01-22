use std::fmt::Debug;
use std::fmt::Display;

use crate::bitvector::util;
use crate::concr::Test;
use crate::forward::Bitwise;
use crate::forward::TypedCmp;

use super::ConcreteBitvector;

impl<const L: u32> ConcreteBitvector<L> {
    #[allow(dead_code)]
    pub fn new(value: u64) -> Self {
        let mask: u64 = Self::bit_mask().0;
        if (value & !mask) != 0 {
            panic!(
                "Machine bitvector value {} does not fit into {} bits",
                value, L
            );
        }

        Self(value)
    }

    // not for use where it may be replaced by abstraction
    pub fn as_unsigned(&self) -> u64 {
        self.0
    }

    pub fn as_signed(&self) -> i64 {
        let mut result = self.0;
        if self.bit_and(Self::sign_bit_mask()).is_nonzero() {
            // add signed extension
            result |= !Self::bit_mask().0;
        }
        result as i64
    }

    pub fn as_offset_signed(&self) -> u64 {
        if L == 0 {
            return self.0;
        }
        // offset by flipping the most significant bit
        self.0 ^ (1 << (L - 1))
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn is_nonzero(&self) -> bool {
        self.0 != 0
    }

    pub fn is_full_mask(&self) -> bool {
        self == &Self::bit_mask()
    }

    pub fn is_sign_bit_set(&self) -> bool {
        util::is_u64_highest_bit_set(self.0, L)
    }

    pub fn sign_bit_mask() -> ConcreteBitvector<L> {
        ConcreteBitvector(util::compute_u64_sign_bit_mask(L))
    }

    pub fn bit_mask() -> ConcreteBitvector<L> {
        ConcreteBitvector(util::compute_u64_mask(L))
    }

    pub fn all_with_length_iter() -> impl Iterator<Item = Self> {
        (0..=Self::bit_mask().as_unsigned()).map(Self)
    }

    pub fn umin(self, other: ConcreteBitvector<L>) -> ConcreteBitvector<L> {
        if self.typed_ulte(other).is_nonzero() {
            self
        } else {
            other
        }
    }

    pub fn umax(self, other: ConcreteBitvector<L>) -> ConcreteBitvector<L> {
        if other.typed_ulte(self).is_nonzero() {
            self
        } else {
            other
        }
    }
}

impl Test for ConcreteBitvector<1> {
    fn is_true(self) -> bool {
        self.is_nonzero()
    }
}

impl<const L: u32> Debug for ConcreteBitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:X}", self.0)
        /*write!(f, "'")?;
        for little_k in 0..L {
            let big_k = L - little_k - 1;
            let bit = (self.0 >> (big_k)) & 1;
            write!(f, "{}", bit)?;
        }
        write!(f, "'")*/
    }
}

impl<const L: u32> Display for ConcreteBitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

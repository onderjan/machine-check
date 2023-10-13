use std::fmt::{Debug, Display};

use crate::{
    bitvector::{concr, util},
    forward::Bitwise,
};

use super::ThreeValuedBitvector;

impl<const L: u32> ThreeValuedBitvector<L> {
    pub fn new(value: u64) -> Self {
        Self::from_concrete(concr::Bitvector::new(value))
    }

    pub fn from_zeros_ones(zeros: concr::Bitvector<L>, ones: concr::Bitvector<L>) -> Self {
        let mask = Self::get_mask();
        // the used bits must be set in zeros, ones, or both
        assert_eq!(Bitwise::bitor(zeros, ones), mask);
        Self { zeros, ones }
    }
    fn from_concrete(value: concr::Bitvector<L>) -> Self {
        // bit-negate for zeros
        let zeros = Bitwise::not(value);
        // leave as-is for ones
        let ones = value;

        Self::from_zeros_ones(zeros, ones)
    }

    pub fn new_unknown() -> Self {
        // all zeros and ones set within mask
        let zeros = Self::get_mask();
        let ones = Self::get_mask();
        Self::from_zeros_ones(zeros, ones)
    }

    pub fn new_value_known(value: concr::Bitvector<L>, known: concr::Bitvector<L>) -> Self {
        let unknown = Bitwise::not(known);
        Self::new_value_unknown(value, unknown)
    }

    pub fn new_value_unknown(value: concr::Bitvector<L>, unknown: concr::Bitvector<L>) -> Self {
        let zeros = Bitwise::bitor(Bitwise::not(value), unknown);
        let ones = Bitwise::bitor(value, unknown);
        Self::from_zeros_ones(zeros, ones)
    }

    pub fn get_unknown_bits(&self) -> concr::Bitvector<L> {
        Bitwise::bitand(self.zeros, self.ones)
    }

    pub fn get_possibly_one_flags(&self) -> concr::Bitvector<L> {
        self.ones
    }

    pub fn get_possibly_zero_flags(&self) -> concr::Bitvector<L> {
        self.zeros
    }

    pub fn concrete_value(&self) -> Option<concr::Bitvector<L>> {
        // all bits must be equal
        let nxor = Bitwise::not(Bitwise::bitxor(self.ones, self.zeros));
        if !nxor.is_zero() {
            return None;
        }
        // ones then contain the value
        Some(self.ones)
    }

    pub fn get_mask() -> concr::Bitvector<L> {
        concr::Bitvector::new(util::compute_u64_mask(L))
    }

    pub fn is_zeros_sign_bit_set(&self) -> bool {
        self.zeros.is_sign_bit_set()
    }

    pub fn is_ones_sign_bit_set(&self) -> bool {
        self.ones.is_sign_bit_set()
    }

    pub fn umin(&self) -> concr::Bitvector<L> {
        // unsigned min value is value of bit-negated zeros (one only where it must be)
        Bitwise::not(self.zeros)
    }

    pub fn umax(&self) -> concr::Bitvector<L> {
        // unsigned max value is value of ones (one everywhere it can be)
        self.ones
    }

    pub fn smin(&self) -> concr::Bitvector<L> {
        let sign_bit_mask = concr::Bitvector::<L>::sign_bit_mask();
        // take the unsigned minimum
        let mut result = self.umin();
        // but the signed value is smaller when the sign bit is one
        // if it is possible to set it to one, set it
        if self.is_ones_sign_bit_set() {
            result = result.bitor(sign_bit_mask)
        }
        result
    }

    pub fn smax(&self) -> concr::Bitvector<L> {
        let sign_bit_mask = concr::Bitvector::<L>::sign_bit_mask();
        // take the unsigned maximum
        let mut result = self.umax();
        // but the signed value is bigger when the sign bit is zero
        // if it is possible to set it to zero, set it
        if self.is_zeros_sign_bit_set() {
            result = result.bitand(sign_bit_mask.not());
        }
        result
    }

    pub fn contains(&self, rhs: &Self) -> bool {
        // rhs zeros must be within our zeros and rhs ones must be within our ones
        let excessive_rhs_zeros = rhs.zeros.bitand(self.zeros.not());
        let excessive_rhs_ones = rhs.ones.bitand(self.ones.not());
        excessive_rhs_zeros.is_zero() && excessive_rhs_ones.is_zero()
    }

    pub fn can_contain(&self, a: concr::Bitvector<L>) -> bool {
        // value zeros must be within our zeros and value ones must be within our ones
        let excessive_rhs_zeros = a.not().bitand(self.zeros.not());
        let excessive_rhs_ones = a.bitand(self.ones.not());
        excessive_rhs_zeros.is_zero() && excessive_rhs_ones.is_zero()
    }
}

impl<const L: u32> Default for ThreeValuedBitvector<L> {
    fn default() -> Self {
        // default to fully unknown
        Self::new_unknown()
    }
}

impl<const L: u32> Debug for ThreeValuedBitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;
        for little_k in 0..L {
            let big_k = L - little_k - 1;
            let zero = (self.zeros.as_unsigned() >> (big_k as usize)) & 1 != 0;
            let one = (self.ones.as_unsigned() >> (big_k as usize)) & 1 != 0;
            let c = match (zero, one) {
                (true, true) => 'X',
                (true, false) => '0',
                (false, true) => '1',
                (false, false) => 'V',
            };
            write!(f, "{}", c)?;
        }
        write!(f, "\"")
    }
}

impl<const L: u32> Display for ThreeValuedBitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

use std::fmt::{Debug, Display};

use crate::{
    abstr::{Boolean, Phi, Test},
    bitvector::{concrete::ConcreteBitvector, util},
    forward::Bitwise,
    traits::misc::MetaEq,
};

use super::ThreeValuedBitvector;

impl<const L: u32> ThreeValuedBitvector<L> {
    #[must_use]
    pub fn new(value: u64) -> Self {
        Self::from_concrete(ConcreteBitvector::new(value))
    }

    #[must_use]
    pub fn from_zeros_ones(zeros: ConcreteBitvector<L>, ones: ConcreteBitvector<L>) -> Self {
        match Self::try_from_zeros_ones(zeros, ones) {
            Ok(ok) => ok,
            Err(_) => panic!(
                "Invalid zeros-ones with some unset bits (length {}, zeros {}, ones {})",
                L, zeros, ones
            ),
        }
    }

    pub fn try_from_zeros_ones(
        zeros: ConcreteBitvector<L>,
        ones: ConcreteBitvector<L>,
    ) -> Result<Self, ()> {
        let mask = Self::get_mask();
        // the used bits must be set in zeros, ones, or both
        if Bitwise::bit_or(zeros, ones) != mask {
            return Err(());
        }
        Ok(Self { zeros, ones })
    }

    #[must_use]
    pub fn from_concrete(value: ConcreteBitvector<L>) -> Self {
        // bit-negate for zeros
        let zeros = Bitwise::bit_not(value);
        // leave as-is for ones
        let ones = value;

        Self::from_zeros_ones(zeros, ones)
    }

    #[must_use]
    pub fn from_interval(min: ConcreteBitvector<L>, max: ConcreteBitvector<L>) -> Self {
        assert!(min.as_unsigned() <= max.as_unsigned());
        // make positions where min and max agree known
        let xor = min.bit_xor(max);
        let Some(unknown_positions) = xor.as_unsigned().checked_ilog2() else {
            // min is equal to max
            return Self::from_concrete(min);
        };
        if unknown_positions >= L {
            // all positions are unknown
            return Self::new_unknown();
        }

        let unknown_mask = ConcreteBitvector::new(1u64 << unknown_positions);
        Self::new_value_unknown(min, unknown_mask)
    }

    pub fn intersection(&self, other: &Self) -> Self {
        let zeros = self.zeros.bit_and(other.zeros);
        let ones = self.ones.bit_and(other.ones);

        Self::from_zeros_ones(zeros, ones)
    }

    #[must_use]
    pub fn new_unknown() -> Self {
        // all zeros and ones set within mask
        let zeros = Self::get_mask();
        let ones = Self::get_mask();
        Self::from_zeros_ones(zeros, ones)
    }

    #[must_use]
    pub fn new_value_known(value: ConcreteBitvector<L>, known: ConcreteBitvector<L>) -> Self {
        let unknown = Bitwise::bit_not(known);
        Self::new_value_unknown(value, unknown)
    }

    #[must_use]
    pub fn new_value_unknown(value: ConcreteBitvector<L>, unknown: ConcreteBitvector<L>) -> Self {
        let zeros = Bitwise::bit_or(Bitwise::bit_not(value), unknown);
        let ones = Bitwise::bit_or(value, unknown);
        Self::from_zeros_ones(zeros, ones)
    }

    #[must_use]
    pub fn get_unknown_bits(&self) -> ConcreteBitvector<L> {
        Bitwise::bit_and(self.zeros, self.ones)
    }

    #[must_use]
    pub fn get_possibly_one_flags(&self) -> ConcreteBitvector<L> {
        self.ones
    }

    #[must_use]
    pub fn get_possibly_zero_flags(&self) -> ConcreteBitvector<L> {
        self.zeros
    }

    #[must_use]
    pub fn concrete_value(&self) -> Option<ConcreteBitvector<L>> {
        // all bits must be equal
        let nxor = Bitwise::bit_not(Bitwise::bit_xor(self.ones, self.zeros));
        if !nxor.is_zero() {
            return None;
        }
        // ones then contain the value
        Some(self.ones)
    }

    #[must_use]
    pub fn get_mask() -> ConcreteBitvector<L> {
        ConcreteBitvector::new(util::compute_u64_mask(L))
    }

    #[must_use]
    pub fn is_zeros_sign_bit_set(&self) -> bool {
        self.zeros.is_sign_bit_set()
    }

    #[must_use]
    pub fn is_ones_sign_bit_set(&self) -> bool {
        self.ones.is_sign_bit_set()
    }

    #[must_use]
    pub fn umin(&self) -> ConcreteBitvector<L> {
        // unsigned min value is value of bit-negated zeros (one only where it must be)
        Bitwise::bit_not(self.zeros)
    }

    #[must_use]
    pub fn umax(&self) -> ConcreteBitvector<L> {
        // unsigned max value is value of ones (one everywhere it can be)
        self.ones
    }

    #[must_use]
    pub fn smin(&self) -> ConcreteBitvector<L> {
        let sign_bit_mask = ConcreteBitvector::<L>::sign_bit_mask();
        // take the unsigned minimum
        let mut result = self.umin();
        // but the signed value is smaller when the sign bit is one
        // if it is possible to set it to one, set it
        if self.is_ones_sign_bit_set() {
            result = result.bit_or(sign_bit_mask)
        }
        result
    }

    #[must_use]
    pub fn smax(&self) -> ConcreteBitvector<L> {
        let sign_bit_mask = ConcreteBitvector::<L>::sign_bit_mask();
        // take the unsigned maximum
        let mut result = self.umax();
        // but the signed value is bigger when the sign bit is zero
        // if it is possible to set it to zero, set it
        if self.is_zeros_sign_bit_set() {
            result = result.bit_and(sign_bit_mask.bit_not());
        }
        result
    }

    #[must_use]
    pub fn contains(&self, rhs: &Self) -> bool {
        // rhs zeros must be within our zeros and rhs ones must be within our ones
        let excessive_rhs_zeros = rhs.zeros.bit_and(self.zeros.bit_not());
        let excessive_rhs_ones = rhs.ones.bit_and(self.ones.bit_not());
        excessive_rhs_zeros.is_zero() && excessive_rhs_ones.is_zero()
    }

    #[must_use]
    pub fn contains_concr(&self, a: &ConcreteBitvector<L>) -> bool {
        // value zeros must be within our zeros and value ones must be within our ones
        let excessive_rhs_zeros = a.bit_not().bit_and(self.zeros.bit_not());
        let excessive_rhs_ones = a.bit_and(self.ones.bit_not());
        excessive_rhs_zeros.is_zero() && excessive_rhs_ones.is_zero()
    }

    #[must_use]
    pub fn concrete_join(&self, concrete: ConcreteBitvector<L>) -> Self {
        let zeros = self.zeros.bit_or(concrete.bit_not());
        let ones = self.ones.bit_or(concrete);
        Self::from_zeros_ones(zeros, ones)
    }

    pub fn all_with_length_iter() -> impl Iterator<Item = Self> {
        let zeros_iter = ConcreteBitvector::<L>::all_with_length_iter();
        zeros_iter.flat_map(|zeros| {
            let ones_iter = ConcreteBitvector::<L>::all_with_length_iter();
            ones_iter.filter_map(move |ones| Self::try_from_zeros_ones(zeros, ones).ok())
        })
    }
}

impl<const L: u32> MetaEq for ThreeValuedBitvector<L> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.ones == other.ones && self.zeros == other.zeros
    }
}

impl ThreeValuedBitvector<1> {
    fn from_bools(can_be_zero: bool, can_be_one: bool) -> Self {
        Self::from_zeros_ones(
            ConcreteBitvector::new(can_be_zero as u64),
            ConcreteBitvector::new(can_be_one as u64),
        )
    }
}

impl From<Boolean> for ThreeValuedBitvector<1> {
    fn from(value: Boolean) -> Self {
        Self::from_bools(value.can_be_false(), value.can_be_true())
    }
}

impl ThreeValuedBitvector<1> {
    pub fn can_be_true(self) -> bool {
        self.ones.is_nonzero()
    }

    pub fn can_be_false(self) -> bool {
        self.zeros.is_nonzero()
    }
}

impl<const L: u32> Default for ThreeValuedBitvector<L> {
    fn default() -> Self {
        // default to fully unknown
        Self::new_unknown()
    }
}

impl<const L: u32> Phi for ThreeValuedBitvector<L> {
    type Condition = Boolean;
    fn phi(self, other: Self) -> Self {
        let zeros = self.zeros.bit_or(other.zeros);
        let ones = self.ones.bit_or(other.ones);

        Self::from_zeros_ones(zeros, ones)
    }

    fn uninit_read() -> Self {
        // present unknown so there is no loss of soundness in case of bug
        Self::new_unknown()
    }

    fn uninit_write() -> Self {
        // present unknown so there is no loss of soundness in case of bug
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

use std::fmt::{Debug, Display};

use crate::{
    abstr::{
        three_valued::{CThreeValuedBitvector, InvalidZerosOnes, RThreeValuedBitvector},
        BitvectorDomain, CBitvectorDomain,
    },
    bitvector::{compute_u64_mask, BitvectorBound},
    concr::{CConcreteBitvector, ConcreteBitvector, SignedBitvector, UnsignedBitvector},
    forward::Bitwise,
    misc::{Join, RBound},
    traits::misc::MetaEq,
};

use super::ThreeValuedBitvector;

impl<B: BitvectorBound> Join for ThreeValuedBitvector<B> {
    fn join(self, other: &Self) -> Self {
        assert_eq!(self.bound(), other.bound());

        let zeros = self.zeros.bit_or(other.zeros);
        let ones = self.ones.bit_or(other.ones);

        Self::from_zeros_ones(zeros, ones)
    }
}

impl<B: BitvectorBound> ThreeValuedBitvector<B> {
    #[must_use]
    pub fn new(value: u64, bound: B) -> Self {
        Self::from_concrete_value(ConcreteBitvector::new(value, bound))
    }

    #[must_use]
    pub fn from_zeros_ones(zeros: ConcreteBitvector<B>, ones: ConcreteBitvector<B>) -> Self {
        match Self::try_from_zeros_ones(zeros, ones) {
            Ok(ok) => ok,
            Err(_) => panic!(
                "Invalid zeros-ones with some unset bits (zeros {}, ones {})",
                zeros, ones
            ),
        }
    }

    pub fn try_from_zeros_ones(
        zeros: ConcreteBitvector<B>,
        ones: ConcreteBitvector<B>,
    ) -> Result<Self, InvalidZerosOnes> {
        assert_eq!(zeros.bound(), ones.bound());
        let bound = zeros.bound();

        // the used bits must be set in zeros, ones, or both
        if Bitwise::bit_or(zeros, ones) != ConcreteBitvector::bit_mask(bound) {
            return Err(InvalidZerosOnes);
        }
        Ok(Self { zeros, ones })
    }

    pub fn from_concrete_value(value: ConcreteBitvector<B>) -> Self {
        // bit-negate for zeros
        let zeros = Bitwise::bit_not(value);
        // leave as-is for ones
        let ones = value;

        Self::from_zeros_ones(zeros, ones)
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
    pub fn contains_concrete(&self, a: &ConcreteBitvector<B>) -> bool {
        // value zeros must be within our zeros and value ones must be within our ones
        let excessive_rhs_zeros = a.bit_not().bit_and(self.zeros.bit_not());
        let excessive_rhs_ones = a.bit_and(self.ones.bit_not());
        excessive_rhs_zeros.is_zero() && excessive_rhs_ones.is_zero()
    }

    #[must_use]
    pub fn get_possibly_one_flags(&self) -> ConcreteBitvector<B> {
        self.ones
    }

    #[must_use]
    pub fn get_possibly_zero_flags(&self) -> ConcreteBitvector<B> {
        self.zeros
    }

    #[must_use]
    pub(super) fn new_unknown(bound: B) -> Self {
        // all zeros and ones set within mask
        let zeros = ConcreteBitvector::bit_mask(bound);
        let ones = ConcreteBitvector::bit_mask(bound);
        Self::from_zeros_ones(zeros, ones)
    }

    #[must_use]
    pub fn new_value_known(value: ConcreteBitvector<B>, known: ConcreteBitvector<B>) -> Self {
        let unknown = Bitwise::bit_not(known);
        Self::new_value_unknown(value, unknown)
    }

    #[must_use]
    pub fn new_value_unknown(value: ConcreteBitvector<B>, unknown: ConcreteBitvector<B>) -> Self {
        let zeros = Bitwise::bit_or(Bitwise::bit_not(value), unknown);
        let ones = Bitwise::bit_or(value, unknown);
        Self::from_zeros_ones(zeros, ones)
    }

    #[must_use]
    pub fn get_unknown_bits(&self) -> ConcreteBitvector<B> {
        Bitwise::bit_and(self.zeros, self.ones)
    }

    #[must_use]
    pub fn contains(&self, rhs: &Self) -> bool {
        // rhs zeros must be within our zeros and rhs ones must be within our ones
        let excessive_rhs_zeros = rhs.zeros.bit_and(self.zeros.bit_not());
        let excessive_rhs_ones = rhs.ones.bit_and(self.ones.bit_not());
        excessive_rhs_zeros.is_zero() && excessive_rhs_ones.is_zero()
    }

    #[allow(dead_code)]
    pub(crate) fn all_with_bound_iter(bound: B) -> impl Iterator<Item = Self> {
        let zeros_iter = ConcreteBitvector::<B>::all_with_bound_iter(bound);
        zeros_iter.flat_map(move |zeros| {
            let ones_iter = ConcreteBitvector::<B>::all_with_bound_iter(bound);
            ones_iter.filter_map(move |ones| Self::try_from_zeros_ones(zeros, ones).ok())
        })
    }

    #[must_use]
    pub fn from_unsigned_interval(min: UnsignedBitvector<B>, max: UnsignedBitvector<B>) -> Self {
        assert_eq!(min.bound(), max.bound());
        let bound = min.bound();

        assert!(min <= max);
        let min = min.cast_bitvector();
        let max = max.cast_bitvector();
        // make positions where min and max agree known
        let xor = min.bit_xor(max);
        let Some(unknown_positions) = xor.to_u64().checked_ilog2() else {
            // min is equal to max
            return Self::from_concrete_value(min);
        };

        let unknown_mask = ConcreteBitvector::new(compute_u64_mask(unknown_positions + 1), bound);
        Self::new_value_unknown(min, unknown_mask)
    }
}

impl<B: BitvectorBound> BitvectorDomain for ThreeValuedBitvector<B> {
    type Bound = B;
    type General<X: BitvectorBound> = ThreeValuedBitvector<X>;

    fn single_value(value: u64, bound: Self::Bound) -> Self {
        Self::new(value, bound)
    }

    fn top(bound: B) -> Self {
        Self::new_unknown(bound)
    }

    fn meet(self, rhs: &Self) -> Option<Self> {
        let zeros = self.zeros.bit_and(rhs.zeros);
        let ones = self.ones.bit_and(rhs.ones);

        Self::try_from_zeros_ones(zeros, ones).ok()
    }

    fn bound(&self) -> Self::Bound {
        // zeros and ones must have the same bound
        self.zeros.bound()
    }

    fn umin(&self) -> UnsignedBitvector<Self::Bound> {
        // unsigned min value is value of bit-negated zeros (one only where it must be)
        Bitwise::bit_not(self.zeros).as_unsigned()
    }

    fn umax(&self) -> UnsignedBitvector<Self::Bound> {
        // unsigned max value is value of ones (one everywhere it can be)
        self.ones.as_unsigned()
    }

    fn smin(&self) -> SignedBitvector<B> {
        let bound = self.bound();
        let sign_bit_mask = ConcreteBitvector::<B>::sign_bit_mask(bound);
        // take the unsigned minimum
        let mut result = self.umin().cast_bitvector();
        // but the signed value is smaller when the sign bit is one
        // if it is possible to set it to one, set it
        if self.is_ones_sign_bit_set() {
            result = result.bit_or(sign_bit_mask)
        }
        result.as_signed()
    }

    fn smax(&self) -> SignedBitvector<B> {
        let bound = self.bound();
        let sign_bit_mask = ConcreteBitvector::<B>::sign_bit_mask(bound);
        // take the unsigned maximum
        let mut result = self.umax().cast_bitvector();
        // but the signed value is bigger when the sign bit is zero
        // if it is possible to set it to zero, set it
        if self.is_zeros_sign_bit_set() {
            result = result.bit_and(sign_bit_mask.bit_not());
        }
        result.as_signed()
    }

    fn concrete_value(&self) -> Option<ConcreteBitvector<B>> {
        // all bits must be equal
        let nxor = Bitwise::bit_not(Bitwise::bit_xor(self.ones, self.zeros));
        if !nxor.is_zero() {
            return None;
        }
        // ones then contain the value
        Some(self.ones)
    }
}

impl<B: BitvectorBound> MetaEq for ThreeValuedBitvector<B> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.ones == other.ones && self.zeros == other.zeros
    }
}

impl<const W: u32> CBitvectorDomain for CThreeValuedBitvector<W> {
    type Concrete = CConcreteBitvector<W>;

    fn from_concrete_bitvector(value: Self::Concrete) -> Self {
        // bit-negate for zeros
        let zeros = Bitwise::bit_not(value);
        // leave as-is for ones
        let ones = value;

        Self::from_zeros_ones(zeros, ones)
    }

    fn from_runtime_bitvector(value: Self::General<RBound>) -> Self {
        assert_eq!(value.bound().width(), W);

        let zeros = ConcreteBitvector::from_runtime_bitvector(value.zeros);
        let ones = ConcreteBitvector::from_runtime_bitvector(value.ones);

        Self::from_zeros_ones(zeros, ones)
    }

    fn as_runtime_bitvector(&self) -> Self::General<RBound> {
        RThreeValuedBitvector {
            zeros: self.zeros.as_runtime_bitvector(),
            ones: self.ones.as_runtime_bitvector(),
        }
    }
}

impl<B: BitvectorBound> Debug for ThreeValuedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let zeros = self.zeros.to_u64();
        let ones = self.ones.to_u64();

        format_zeros_ones(f, self.bound().width(), zeros, ones)
    }
}

impl<B: BitvectorBound> Display for ThreeValuedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

pub fn format_zeros_ones(
    f: &mut std::fmt::Formatter<'_>,
    bit_width: u32,
    zeros: u64,
    ones: u64,
) -> std::fmt::Result {
    let nxor = !(ones ^ zeros);
    if nxor == 0 {
        // concrete value
        return write!(f, "{:?}", ones);
    }

    write!(f, "\"")?;
    for little_k in 0..bit_width {
        let big_k = bit_width - little_k - 1;
        let zero = (zeros >> (big_k as usize)) & 1 != 0;
        let one = (ones >> (big_k as usize)) & 1 != 0;
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

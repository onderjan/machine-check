use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use crate::{
    abstr::{
        Abstr, BitvectorDomain, BitvectorElement, BitvectorField, Boolean, Field, ManipField, Phi,
        Test,
    },
    bitvector::{
        abstr::three_valued::RThreeValuedBitvector,
        interval::UnsignedInterval,
        util::{self, compute_u64_mask},
    },
    concr::{
        self, ConcreteBitvector, RConcreteBitvector, RSignedBitvector, RUnsignedBitvector,
        SignedBitvector, UnsignedBitvector,
    },
    forward::Bitwise,
    traits::misc::MetaEq,
};

use super::ThreeValuedBitvector;

impl RThreeValuedBitvector {
    #[must_use]
    pub fn new(value: u64, width: u32) -> Self {
        Self::from_concrete(RConcreteBitvector::new(value, width))
    }

    #[must_use]
    pub fn new_unknown(width: u32) -> Self {
        // all zeros and ones set within mask
        let zeros = RConcreteBitvector::from_masked_u64(!0u64, width);
        let ones = zeros;
        Self::from_zeros_ones(zeros, ones)
    }

    fn from_concrete(value: RConcreteBitvector) -> Self {
        // bit-negate for zeros
        let zeros = Bitwise::bit_not(value);
        // leave as-is for ones
        let ones = value;

        Self::from_zeros_ones(zeros, ones)
    }

    pub fn width(&self) -> u32 {
        self.zeros.width()
    }

    #[must_use]
    pub fn from_zeros_ones(zeros: RConcreteBitvector, ones: RConcreteBitvector) -> Self {
        assert_eq!(zeros.width(), ones.width());
        RThreeValuedBitvector { zeros, ones }
    }

    pub(crate) fn unwrap_typed<const W: u32>(self) -> ThreeValuedBitvector<W> {
        assert_eq!(self.zeros.width(), W);
        ThreeValuedBitvector {
            zeros: self.zeros.unwrap_typed(),
            ones: self.ones.unwrap_typed(),
        }
    }

    #[must_use]
    pub fn umin(&self) -> RUnsignedBitvector {
        // unsigned min value is value of bit-negated zeros (one only where it must be)
        self.zeros.bit_not().cast_unsigned()
    }

    #[must_use]
    pub fn umax(&self) -> RUnsignedBitvector {
        // unsigned max value is value of ones (one everywhere it can be)
        self.ones.cast_unsigned()
    }

    #[must_use]
    pub fn smin(&self) -> RSignedBitvector {
        let sign_bit_mask = self.zeros.sign_bit_mask_bitvector();
        // take the unsigned minimum
        let mut result = self.umin().as_bitvector();
        // but the signed value is smaller when the sign bit is one
        // if it is possible to set it to one, set it
        if self.is_ones_sign_bit_set() {
            result = result.bit_or(sign_bit_mask)
        }
        result.cast_signed()
    }

    #[must_use]
    pub fn smax(&self) -> RSignedBitvector {
        let sign_bit_mask = self.zeros.sign_bit_mask_bitvector();
        // take the unsigned maximum
        let mut result = self.umax().as_bitvector();
        // but the signed value is bigger when the sign bit is zero
        // if it is possible to set it to zero, set it
        if self.is_zeros_sign_bit_set() {
            result = result.bit_and(sign_bit_mask.bit_not());
        }
        result.cast_signed()
    }

    #[must_use]
    pub fn is_zeros_sign_bit_set(&self) -> bool {
        self.zeros.is_sign_bit_set()
    }

    #[must_use]
    pub fn is_ones_sign_bit_set(&self) -> bool {
        self.ones.is_sign_bit_set()
    }

    pub const fn bit_mask_u64(self) -> u64 {
        self.zeros.bit_mask_u64()
    }

    pub const fn bit_mask_bitvector(self) -> RConcreteBitvector {
        self.zeros.bit_mask_bitvector()
    }

    #[must_use]
    pub fn contains_concr(&self, a: &RConcreteBitvector) -> bool {
        assert_eq!(self.width(), a.width());
        // value zeros must be within our zeros and value ones must be within our ones
        let excessive_rhs_zeros = a.bit_not().bit_and(self.zeros.bit_not());
        let excessive_rhs_ones = a.bit_and(self.ones.bit_not());
        excessive_rhs_zeros.is_zero() && excessive_rhs_ones.is_zero()
    }

    #[must_use]
    pub fn concrete_value(&self) -> Option<RConcreteBitvector> {
        // all bits must be equal
        let nxor = Bitwise::bit_not(Bitwise::bit_xor(self.ones, self.zeros));
        if !nxor.is_zero() {
            return None;
        }
        // ones then contain the value
        Some(self.ones)
    }

    #[must_use]
    pub fn new_value_unknown(value: RConcreteBitvector, unknown: RConcreteBitvector) -> Self {
        assert_eq!(value.width(), unknown.width());
        let zeros = Bitwise::bit_or(Bitwise::bit_not(value), unknown);
        let ones = Bitwise::bit_or(value, unknown);
        Self::from_zeros_ones(zeros, ones)
    }

    #[must_use]
    pub fn get_unknown_bits(&self) -> RConcreteBitvector {
        Bitwise::bit_and(self.zeros, self.ones)
    }

    #[must_use]
    pub fn get_possibly_one_flags(&self) -> RConcreteBitvector {
        self.ones
    }

    #[must_use]
    pub fn get_possibly_zero_flags(&self) -> RConcreteBitvector {
        self.zeros
    }
}

impl<const W: u32> Abstr<concr::Bitvector<W>> for ThreeValuedBitvector<W> {
    fn from_concrete(value: concr::Bitvector<W>) -> Self {
        // bit-negate for zeros
        let zeros = Bitwise::bit_not(value);
        // leave as-is for ones
        let ones = value;

        Self::from_zeros_ones(zeros, ones)
    }
}

impl<const W: u32> ThreeValuedBitvector<W> {
    #[must_use]
    pub fn new(value: u64) -> Self {
        Self::from_concrete(ConcreteBitvector::new(value))
    }

    pub(crate) fn to_runtime(self) -> RThreeValuedBitvector {
        RThreeValuedBitvector {
            zeros: self.zeros.to_runtime(),
            ones: self.ones.to_runtime(),
        }
    }

    #[must_use]
    pub fn from_zeros_ones(zeros: ConcreteBitvector<W>, ones: ConcreteBitvector<W>) -> Self {
        match Self::try_from_zeros_ones(zeros, ones) {
            Ok(ok) => ok,
            Err(_) => panic!(
                "Invalid zeros-ones with some unset bits (length {}, zeros {}, ones {})",
                W, zeros, ones
            ),
        }
    }

    pub fn try_from_zeros_ones(
        zeros: ConcreteBitvector<W>,
        ones: ConcreteBitvector<W>,
    ) -> Result<Self, ()> {
        let mask = Self::get_mask();
        // the used bits must be set in zeros, ones, or both
        if Bitwise::bit_or(zeros, ones) != mask {
            return Err(());
        }
        Ok(Self { zeros, ones })
    }

    #[must_use]
    pub fn from_interval(min: ConcreteBitvector<W>, max: ConcreteBitvector<W>) -> Self {
        assert!(min.to_u64() <= max.to_u64());
        // make positions where min and max agree known
        let xor = min.bit_xor(max);
        let Some(unknown_positions) = xor.to_u64().checked_ilog2() else {
            // min is equal to max
            return Self::from_concrete(min);
        };

        let unknown_mask = ConcreteBitvector::new(compute_u64_mask(unknown_positions + 1));
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
    pub fn new_value_known(value: ConcreteBitvector<W>, known: ConcreteBitvector<W>) -> Self {
        let unknown = Bitwise::bit_not(known);
        Self::new_value_unknown(value, unknown)
    }

    #[must_use]
    pub fn new_value_unknown(value: ConcreteBitvector<W>, unknown: ConcreteBitvector<W>) -> Self {
        let zeros = Bitwise::bit_or(Bitwise::bit_not(value), unknown);
        let ones = Bitwise::bit_or(value, unknown);
        Self::from_zeros_ones(zeros, ones)
    }

    #[must_use]
    pub fn get_unknown_bits(&self) -> ConcreteBitvector<W> {
        Bitwise::bit_and(self.zeros, self.ones)
    }

    #[must_use]
    pub fn get_possibly_one_flags(&self) -> ConcreteBitvector<W> {
        self.ones
    }

    #[must_use]
    pub fn get_possibly_zero_flags(&self) -> ConcreteBitvector<W> {
        self.zeros
    }

    #[must_use]
    pub fn concrete_value(&self) -> Option<ConcreteBitvector<W>> {
        // all bits must be equal
        let nxor = Bitwise::bit_not(Bitwise::bit_xor(self.ones, self.zeros));
        if !nxor.is_zero() {
            return None;
        }
        // ones then contain the value
        Some(self.ones)
    }

    #[must_use]
    pub fn get_mask() -> ConcreteBitvector<W> {
        ConcreteBitvector::new(util::compute_u64_mask(W))
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
    pub fn umin(&self) -> UnsignedBitvector<W> {
        // unsigned min value is value of bit-negated zeros (one only where it must be)
        Bitwise::bit_not(self.zeros).cast_unsigned()
    }

    #[must_use]
    pub fn umax(&self) -> UnsignedBitvector<W> {
        // unsigned max value is value of ones (one everywhere it can be)
        self.ones.cast_unsigned()
    }

    #[must_use]
    pub fn smin(&self) -> SignedBitvector<W> {
        let sign_bit_mask = ConcreteBitvector::<W>::sign_bit_mask();
        // take the unsigned minimum
        let mut result = self.umin().as_bitvector();
        // but the signed value is smaller when the sign bit is one
        // if it is possible to set it to one, set it
        if self.is_ones_sign_bit_set() {
            result = result.bit_or(sign_bit_mask)
        }
        result.cast_signed()
    }

    #[must_use]
    pub fn smax(&self) -> SignedBitvector<W> {
        let sign_bit_mask = ConcreteBitvector::<W>::sign_bit_mask();
        // take the unsigned maximum
        let mut result = self.umax().as_bitvector();
        // but the signed value is bigger when the sign bit is zero
        // if it is possible to set it to zero, set it
        if self.is_zeros_sign_bit_set() {
            result = result.bit_and(sign_bit_mask.bit_not());
        }
        result.cast_signed()
    }

    #[must_use]
    pub fn contains(&self, rhs: &Self) -> bool {
        // rhs zeros must be within our zeros and rhs ones must be within our ones
        let excessive_rhs_zeros = rhs.zeros.bit_and(self.zeros.bit_not());
        let excessive_rhs_ones = rhs.ones.bit_and(self.ones.bit_not());
        excessive_rhs_zeros.is_zero() && excessive_rhs_ones.is_zero()
    }

    #[must_use]
    pub fn contains_concr(&self, a: &ConcreteBitvector<W>) -> bool {
        // value zeros must be within our zeros and value ones must be within our ones
        let excessive_rhs_zeros = a.bit_not().bit_and(self.zeros.bit_not());
        let excessive_rhs_ones = a.bit_and(self.ones.bit_not());
        excessive_rhs_zeros.is_zero() && excessive_rhs_ones.is_zero()
    }

    #[must_use]
    pub fn concrete_join(&self, concrete: ConcreteBitvector<W>) -> Self {
        let zeros = self.zeros.bit_or(concrete.bit_not());
        let ones = self.ones.bit_or(concrete);
        Self::from_zeros_ones(zeros, ones)
    }

    pub fn all_with_length_iter() -> impl Iterator<Item = Self> {
        let zeros_iter = ConcreteBitvector::<W>::all_with_width_iter();
        zeros_iter.flat_map(|zeros| {
            let ones_iter = ConcreteBitvector::<W>::all_with_width_iter();
            ones_iter.filter_map(move |ones| Self::try_from_zeros_ones(zeros, ones).ok())
        })
    }

    fn field_value(&self) -> ThreeValuedFieldValue {
        ThreeValuedFieldValue {
            zeros: self.zeros.to_u64(),
            ones: self.ones.to_u64(),
        }
    }
}

impl<const W: u32> MetaEq for ThreeValuedBitvector<W> {
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

impl<const W: u32> Default for ThreeValuedBitvector<W> {
    fn default() -> Self {
        // default to fully unknown
        Self::new_unknown()
    }
}

impl<const W: u32> Phi for ThreeValuedBitvector<W> {
    fn phi(self, other: Self) -> Self {
        let zeros = self.zeros.bit_or(other.zeros);
        let ones = self.ones.bit_or(other.ones);

        Self::from_zeros_ones(zeros, ones)
    }

    fn uninit() -> Self {
        // present unknown so there is no loss of soundness in case of bug
        Self::new_unknown()
    }
}

impl<const W: u32> BitvectorDomain<W> for ThreeValuedBitvector<W> {
    fn unsigned_interval(&self) -> UnsignedInterval<W> {
        UnsignedInterval::new(self.umin(), self.umax())
    }

    fn element_description(&self) -> BitvectorElement {
        BitvectorElement {
            three_valued: Some(self.field_value()),
            dual_interval: None,
        }
    }

    fn join(self, other: Self) -> Self {
        self.phi(other)
    }

    fn meet(self, other: Self) -> Option<Self> {
        let zeros = self.zeros.bit_and(other.zeros);
        let ones = self.ones.bit_and(other.ones);

        Self::try_from_zeros_ones(zeros, ones).ok()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ThreeValuedFieldValue {
    zeros: u64,
    ones: u64,
}

impl ThreeValuedFieldValue {
    pub fn write(&self, f: &mut std::fmt::Formatter<'_>, bit_width: u32) -> std::fmt::Result {
        format_zeros_ones(f, bit_width, self.zeros, self.ones)
    }
}

impl<const W: u32> ManipField for ThreeValuedBitvector<W> {
    fn num_bits(&self) -> Option<u32> {
        Some(W)
    }

    fn runtime_bitvector(&self) -> Option<crate::abstr::RBitvector> {
        Some(self.to_runtime())
    }

    fn min_unsigned(&self) -> Option<u64> {
        Some(self.umin().to_u64())
    }

    fn max_unsigned(&self) -> Option<u64> {
        Some(self.umax().to_u64())
    }

    fn min_signed(&self) -> Option<i64> {
        Some(self.smin().to_i64())
    }

    fn max_signed(&self) -> Option<i64> {
        Some(self.smax().to_i64())
    }

    fn index(&self, _index: u64) -> Option<&dyn ManipField> {
        None
    }

    fn description(&self) -> Field {
        Field::Bitvector(BitvectorField {
            bit_width: W,
            element: self.element_description(),
        })
    }
}

impl<const W: u32> Debug for ThreeValuedBitvector<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.field_value().write(f, W)
    }
}

impl<const W: u32> Display for ThreeValuedBitvector<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl Debug for RThreeValuedBitvector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_zeros_ones(f, self.width(), self.zeros.to_u64(), self.ones.to_u64())
    }
}

impl Display for RThreeValuedBitvector {
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

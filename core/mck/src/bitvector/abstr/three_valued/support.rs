use std::fmt::{Debug, Display};

use crate::{
    abstr::{
        three_valued::{InvalidZerosOnes, RThreeValuedBitvector},
        BitvectorDomain, Boolean, Phi, Test,
    },
    bitvector::{
        interval::UnsignedInterval,
        util::{self, compute_u64_mask},
        BitvectorBound,
    },
    concr::{self, ConcreteBitvector, SignedBitvector, UnsignedBitvector},
    forward::Bitwise,
    misc::{CBound, Join},
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

    pub fn bound(&self) -> B {
        // zeros and ones must have the same bound
        self.zeros.bound()
    }

    pub fn width(&self) -> u32 {
        self.bound().width()
    }

    #[must_use]
    pub fn umin(&self) -> UnsignedBitvector<B> {
        // unsigned min value is value of bit-negated zeros (one only where it must be)
        Bitwise::bit_not(self.zeros).as_unsigned()
    }

    #[must_use]
    pub fn umax(&self) -> UnsignedBitvector<B> {
        // unsigned max value is value of ones (one everywhere it can be)
        self.ones.as_unsigned()
    }

    #[must_use]
    pub fn smin(&self) -> SignedBitvector<B> {
        let bound = self.bound();
        let sign_bit_mask = ConcreteBitvector::<B>::sign_bit_mask(bound);
        // take the unsigned minimum
        let mut result = self.umin().as_bitvector();
        // but the signed value is smaller when the sign bit is one
        // if it is possible to set it to one, set it
        if self.is_ones_sign_bit_set() {
            result = result.bit_or(sign_bit_mask)
        }
        result.as_signed()
    }

    #[must_use]
    pub fn smax(&self) -> SignedBitvector<B> {
        let bound = self.bound();
        let sign_bit_mask = ConcreteBitvector::<B>::sign_bit_mask(bound);
        // take the unsigned maximum
        let mut result = self.umax().as_bitvector();
        // but the signed value is bigger when the sign bit is zero
        // if it is possible to set it to zero, set it
        if self.is_zeros_sign_bit_set() {
            result = result.bit_and(sign_bit_mask.bit_not());
        }
        result.as_signed()
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
    pub fn concrete_value(&self) -> Option<ConcreteBitvector<B>> {
        // all bits must be equal
        let nxor = Bitwise::bit_not(Bitwise::bit_xor(self.ones, self.zeros));
        if !nxor.is_zero() {
            return None;
        }
        // ones then contain the value
        Some(self.ones)
    }

    pub fn as_runtime_bitvector(&self) -> RThreeValuedBitvector {
        RThreeValuedBitvector {
            zeros: self.zeros.as_runtime_bitvector(),
            ones: self.ones.as_runtime_bitvector(),
        }
    }

    /*



    #[must_use]
    pub fn from_interval(min: ConcreteBitvector<B>, max: ConcreteBitvector<B>) -> Self {
        assert!(min.to_u64() <= max.to_u64());
        // make positions where min and max agree known
        let xor = min.bit_xor(max);
        let Some(unknown_positions) = xor.to_u64().checked_ilog2() else {
            // min is equal to max
            return Self::from_concrete_value(min);
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
    pub fn get_mask() -> ConcreteBitvector<B> {
        ConcreteBitvector::new(util::compute_u64_mask(W))
    }



    #[must_use]
    pub fn contains(&self, rhs: &Self) -> bool {
        // rhs zeros must be within our zeros and rhs ones must be within our ones
        let excessive_rhs_zeros = rhs.zeros.bit_and(self.zeros.bit_not());
        let excessive_rhs_ones = rhs.ones.bit_and(self.ones.bit_not());
        excessive_rhs_zeros.is_zero() && excessive_rhs_ones.is_zero()
    }


    #[must_use]
    pub fn concrete_join(&self, concrete: ConcreteBitvector<B>) -> Self {
        let zeros = self.zeros.bit_or(concrete.bit_not());
        let ones = self.ones.bit_or(concrete);
        Self::from_zeros_ones(zeros, ones)
    }

    pub fn all_with_width_iter() -> impl Iterator<Item = Self> {
        let zeros_iter = ConcreteBitvector::<B>::all_with_width_iter();
        zeros_iter.flat_map(|zeros| {
            let ones_iter = ConcreteBitvector::<B>::all_with_width_iter();
            ones_iter.filter_map(move |ones| Self::try_from_zeros_ones(zeros, ones).ok())
        })
    }

    */
}

impl<const W: u32> ThreeValuedBitvector<CBound<W>> {
    pub fn from_runtime_bitvector(bitvector: RThreeValuedBitvector) -> Self {
        assert_eq!(bitvector.width(), W);

        let zeros = ConcreteBitvector::from_runtime_bitvector(bitvector.zeros);
        let ones = ConcreteBitvector::from_runtime_bitvector(bitvector.ones);

        Self::from_zeros_ones(zeros, ones)
    }
}

impl<B: BitvectorBound> MetaEq for ThreeValuedBitvector<B> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.ones == other.ones && self.zeros == other.zeros
    }
}

/*impl MetaEq for RThreeValuedBitvector {
    fn meta_eq(&self, other: &Self) -> bool {
        assert_eq!(self.zeros.width(), other.zeros.width());
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

impl<B: BitvectorBound> Default for ThreeValuedBitvector<B> {
    fn default() -> Self {
        // default to fully unknown
        Self::new_unknown()
    }
}


impl<B: BitvectorBound> BitvectorDomain<B> for ThreeValuedBitvector<B> {
    fn unsigned_interval(&self) -> UnsignedInterval<B> {
        UnsignedInterval::new(self.umin(), self.umax())
    }

    fn join(self, other: Self) -> Self {
        self.phi(other)
    }

    fn meet(self, other: Self) -> Option<Self> {
        let zeros = self.zeros.bit_and(other.zeros);
        let ones = self.ones.bit_and(other.ones);

        Self::try_from_zeros_ones(zeros, ones).ok()
    }
}*/

impl<B: BitvectorBound> Phi for ThreeValuedBitvector<B> {
    fn phi(self, other: Self) -> Self {
        let zeros = self.zeros.bit_or(other.zeros);
        let ones = self.ones.bit_or(other.ones);

        Self::from_zeros_ones(zeros, ones)
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

use std::fmt::Debug;
use std::fmt::Display;

use crate::bitvector::bound::BitvectorBound;
use crate::bitvector::RBound;
use crate::concr::RConcreteBitvector;
use crate::misc::CBound;

use super::ConcreteBitvector;
use super::SignedBitvector;
use super::UnsignedBitvector;

impl<B: BitvectorBound> ConcreteBitvector<B> {
    pub fn new(value: u64, bound: B) -> Self {
        let mask: u64 = bound.mask();
        if (value & !mask) != 0 {
            panic!(
                "Machine bitvector value {} does not fit into bound {:?}",
                value, bound
            );
        }
        Self { value, bound }
    }

    pub fn bound(self) -> B {
        self.bound
    }

    pub fn zero(bound: B) -> Self {
        Self { value: 0, bound }
    }

    pub fn one(bound: B) -> Self {
        // mask by bound to support zero-sized bitvectors
        let one = 1 & bound.mask();
        Self { value: one, bound }
    }

    pub fn bit_mask(bound: B) -> Self {
        Self {
            value: bound.mask(),
            bound,
        }
    }

    pub fn sign_bit_mask(bound: B) -> Self {
        Self {
            value: bound.sign_bit_mask(),
            bound,
        }
    }

    pub fn from_masked_u64(value: u64, bound: B) -> Self {
        let value = value & bound.mask();
        Self { value, bound }
    }

    pub fn to_u64(self) -> u64 {
        self.value
    }

    pub fn to_i64(self) -> i64 {
        let mut result = self.value;
        let sign_bit_mask = self.bound.sign_bit_mask();
        if self.value & sign_bit_mask != 0 {
            // add signed extension
            result |= !self.bound.mask();
        }
        result as i64
    }

    pub fn is_sign_bit_set(self) -> bool {
        self.value & self.bound.sign_bit_mask() != 0
    }

    /*pub fn unwrap_typed<const W: u32>(self) -> ConcreteBitvector<W> {
        assert_eq!(self.bound, W);
        ConcreteBitvector(self.value)
    }*/

    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    pub fn is_nonzero(&self) -> bool {
        self.value != 0
    }
    pub fn is_full_mask(&self) -> bool {
        self.value == self.bound.mask()
    }

    pub fn all_with_bound_iter(bound: B) -> impl Iterator<Item = Self> {
        (0..=bound.mask()).map(move |value| Self { bound, value })
    }

    pub const fn as_unsigned(self) -> UnsignedBitvector<B> {
        UnsignedBitvector::from_bitvector(self)
    }

    pub const fn as_signed(self) -> SignedBitvector<B> {
        SignedBitvector::from_bitvector(self)
    }

    pub fn new_umin(bound: B) -> Self {
        // this is just zero
        Self::zero(bound)
    }

    pub fn new_underhalf(bound: B) -> Self {
        let value = bound.mask() ^ bound.sign_bit_mask();
        Self::from_masked_u64(value, bound)
    }

    pub fn new_overhalf(bound: B) -> Self {
        let value = bound.sign_bit_mask();
        Self::from_masked_u64(value, bound)
    }

    pub fn new_umax(bound: B) -> Self {
        let value = bound.mask();
        Self::from_masked_u64(value, bound)
    }

    pub fn as_runtime_bitvector(self) -> RConcreteBitvector {
        RConcreteBitvector {
            bound: RBound::new(self.bound.width()),
            value: self.value,
        }
    }
}

impl<const W: u32> ConcreteBitvector<CBound<W>> {
    pub fn from_runtime_bitvector(bitvector: RConcreteBitvector) -> Self {
        assert_eq!(bitvector.bound.width(), W);

        Self {
            bound: CBound,
            value: bitvector.value,
        }
    }
}

/*
impl<const W: u32> ConcreteBitvector<W> {
    pub fn new(value: u64) -> Self {
        let mask: u64 = Self::bit_mask().0;
        if (value & !mask) != 0 {
            panic!(
                "Machine bitvector value {} does not fit into {} bits",
                value, W
            );
        }

        Self(value)
    }

    pub fn try_new(value: u64) -> Option<Self> {
        let mask: u64 = Self::bit_mask().0;
        if (value & !mask) != 0 {
            return None;
        }

        Some(Self(value))
    }

    pub const fn cast_unsigned(self) -> UnsignedBitvector<W> {
        UnsignedBitvector::from_bitvector(self)
    }

    pub const fn cast_signed(self) -> SignedBitvector<W> {
        SignedBitvector::from_bitvector(self)
    }

    // not for use where it may be replaced by abstraction
    // TODO: remove and replace by casts
    pub fn to_u64(&self) -> u64 {
        self.0
    }

    pub fn to_i64(&self) -> i64 {
        let mut result = self.0;
        if self.bit_and(Self::sign_bit_mask()).is_nonzero() {
            // add signed extension
            result |= !Self::bit_mask().0;
        }
        result as i64
    }

    pub fn as_offset_signed(&self) -> u64 {
        if W == 0 {
            return self.0;
        }
        // offset by flipping the most significant bit
        self.0 ^ (1 << (W - 1))
    }

    pub const fn zero() -> Self {
        Self(0)
    }


    pub const fn const_umax() -> Self {
        Self::bit_mask()
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn is_nonzero(&self) -> bool {
        self.0 != 0
    }

    pub fn one() -> Self {
        if W > 0 {
            Self(1)
        } else {
            // 1 is not a valid value for zero-bit bitvector
            Self(0)
        }
    }

    pub fn is_full_mask(&self) -> bool {
        self == &Self::bit_mask()
    }

    pub fn is_sign_bit_set(&self) -> bool {
        util::is_u64_highest_bit_set(self.0, W)
    }

    pub const fn sign_bit_mask() -> ConcreteBitvector<W> {
        ConcreteBitvector(util::compute_u64_sign_bit_mask(W))
    }

    const fn without_sign_bit_mask() -> ConcreteBitvector<W> {
        if W == 0 {
            return ConcreteBitvector(0);
        }

        ConcreteBitvector(util::compute_u64_mask(W) ^ util::compute_u64_sign_bit_mask(W))
    }

    pub const fn bit_mask() -> ConcreteBitvector<W> {
        ConcreteBitvector(util::compute_u64_mask(W))
    }

    pub fn all_with_bound_iter() -> impl Iterator<Item = Self> {
        (0..=Self::bit_mask().to_u64()).map(Self)
    }

    pub fn umin(self, other: ConcreteBitvector<W>) -> ConcreteBitvector<W> {
        if self.ule(other).into_bool() {
            self
        } else {
            other
        }
    }

    pub fn umax(self, other: ConcreteBitvector<W>) -> ConcreteBitvector<W> {
        if other.ule(self).into_bool() {
            self
        } else {
            other
        }
    }

    pub fn to_runtime(self) -> RConcreteBitvector {
        RConcreteBitvector {
            value: self.0,
            bound: W,
        }
    }

    pub fn from_runtime(runtime: RConcreteBitvector) -> Self {
        assert_eq!(runtime.bound, W);
        Self(runtime.value)
    }
}*/

impl<B: BitvectorBound> Debug for ConcreteBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ignore bound
        std::fmt::Debug::fmt(&self.value, f)
    }
}

impl<B: BitvectorBound> Display for ConcreteBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

/*impl Debug for RConcreteBitvector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>({})", self.bound, self.value)
    }
}

impl Display for RConcreteBitvector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}*/

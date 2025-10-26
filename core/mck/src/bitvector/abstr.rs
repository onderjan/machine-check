#[cfg(feature = "Zdual_interval")]
use crate::abstr::dual_interval::DualInterval;
#[cfg(feature = "Zdual_interval")]
use crate::abstr::three_valued::ThreeValuedBitvector;
use crate::abstr::{Abstr, AbstractValue};
use crate::bitvector::bound::{CBound, RBound};
use crate::concr::{ConcreteBitvector, SignedBitvector, UnsignedBitvector};
use crate::misc::{BitvectorBound, Join, MetaEq};
use std::hash::Hash;

pub mod combined;
pub mod dual_interval;
pub mod three_valued;

pub trait BitvectorDomain: Clone + Copy + Hash + Join + MetaEq {
    type Bound: BitvectorBound;
    type General<X: BitvectorBound>: BitvectorDomain<Bound = X>;

    fn bound(&self) -> Self::Bound;

    fn single_value(value: u64, bound: Self::Bound) -> Self;
    fn top(bound: Self::Bound) -> Self;
    fn meet(self, other: &Self) -> Option<Self>;

    fn umin(&self) -> UnsignedBitvector<Self::Bound>;
    fn umax(&self) -> UnsignedBitvector<Self::Bound>;
    fn smin(&self) -> SignedBitvector<Self::Bound>;
    fn smax(&self) -> SignedBitvector<Self::Bound>;

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>>;
}

pub trait CBitvectorDomain: BitvectorDomain {
    type Concrete;

    fn from_concrete_bitvector(value: Self::Concrete) -> Self;
    fn from_runtime_bitvector(value: Self::General<RBound>) -> Self;
    fn as_runtime_bitvector(&self) -> Self::General<RBound>;
}

#[cfg(not(feature = "Zdual_interval"))]
pub type Bitvector<W> = three_valued::ThreeValuedBitvector<W>;

#[cfg(feature = "Zdual_interval")]
pub type Bitvector<W> = combined::CombinedBitvector<W, ThreeValuedBitvector<W>, DualInterval<W>>;

pub type RBitvector = Bitvector<RBound>;
pub type CBitvector<const W: u32> = Bitvector<CBound<W>>;

pub type PanicBitvector = Bitvector<CBound<32>>;

impl<const W: u32> Abstr<super::concr::Bitvector<CBound<W>>> for Bitvector<CBound<W>> {
    fn from_concrete(value: super::concr::Bitvector<CBound<W>>) -> Self {
        Self::from_concrete_bitvector(value)
    }

    fn from_runtime(value: &AbstractValue) -> Self {
        Self::from_runtime_bitvector(*value.expect_bitvector())
    }

    fn to_runtime(&self) -> AbstractValue {
        AbstractValue::Bitvector(self.as_runtime_bitvector())
    }
}

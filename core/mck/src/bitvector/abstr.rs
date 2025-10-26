use crate::abstr::{Abstr, AbstractValue};
use crate::bitvector::bound::{CBound, RBound};
use crate::concr::{ConcreteBitvector, SignedBitvector, UnsignedBitvector};
use crate::misc::{BitvectorBound, Join, MetaEq};
use std::hash::Hash;

pub mod combined;
mod dual_interval;
pub mod three_valued;

pub trait BitvectorDomain: Clone + Copy + Hash + Join + MetaEq {
    type Bound: BitvectorBound;

    fn bound(&self) -> Self::Bound;

    fn meet(self, rhs: &Self) -> Option<Self>;

    fn umin(&self) -> UnsignedBitvector<Self::Bound>;
    fn umax(&self) -> UnsignedBitvector<Self::Bound>;
    fn smin(&self) -> SignedBitvector<Self::Bound>;
    fn smax(&self) -> SignedBitvector<Self::Bound>;

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>>;
}

pub trait CBitvectorDomain: BitvectorDomain {
    type Concrete;
    type Runtime;

    fn from_concrete_bitvector(value: Self::Concrete) -> Self;
    fn from_runtime_bitvector(value: Self::Runtime) -> Self;
    fn as_runtime_bitvector(&self) -> Self::Runtime;
}

#[cfg(not(feature = "Zdual_interval"))]
pub type Bitvector<W> = three_valued::ThreeValuedBitvector<W>;

#[cfg(feature = "Zdual_interval")]
pub type Bitvector<W> = combined::CombinedBitvector<W>;

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

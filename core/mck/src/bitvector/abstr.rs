use super::interval::UnsignedInterval;
use crate::abstr::{Abstr, AbstractValue, Phi};
#[cfg(not(feature = "Zdual_interval"))]
use crate::{
    bitvector::bound::{CBound, RBound},
    misc::BitvectorBound,
};
use std::hash::Hash;

pub mod combined;
mod dual_interval;
pub mod three_valued;

pub trait BitvectorDomain<B: BitvectorBound>: Clone + Copy + Hash + Phi {
    fn unsigned_interval(&self) -> UnsignedInterval<B>;

    fn join(self, other: Self) -> Self;
    fn meet(self, other: Self) -> Option<Self>;
}

#[cfg(not(feature = "Zdual_interval"))]
pub type Bitvector<W> = three_valued::ThreeValuedBitvector<W>;

#[cfg(feature = "Zdual_interval")]
pub type Bitvector<const W: u32> = combined::CombinedBitvector<W>;

pub type RBitvector = Bitvector<RBound>;
pub type CBitvector<const W: u32> = Bitvector<CBound<W>>;

pub type PanicBitvector = Bitvector<CBound<32>>;

//pub type BooleanBitvector = Bitvector<1>;
//pub type PanicBitvector = Bitvector<32>;

impl<const W: u32> Abstr<super::concr::Bitvector<CBound<W>>> for Bitvector<CBound<W>> {
    fn from_concrete(value: super::concr::Bitvector<CBound<W>>) -> Self {
        Self::from_concrete_value(value)
    }

    fn from_runtime(value: &AbstractValue) -> Self {
        Self::from_runtime_bitvector(*value.expect_bitvector())
    }

    fn to_runtime(&self) -> AbstractValue {
        AbstractValue::Bitvector(self.as_runtime_bitvector())
    }
}

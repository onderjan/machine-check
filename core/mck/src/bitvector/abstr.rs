use super::interval::UnsignedInterval;
use crate::abstr::{Abstr, AbstractValue, Phi};
use std::hash::Hash;

pub mod combined;
mod dual_interval;
pub mod three_valued;

pub trait BitvectorDomain<const W: u32>: Clone + Copy + Hash + Phi {
    fn unsigned_interval(&self) -> UnsignedInterval<W>;

    fn join(self, other: Self) -> Self;
    fn meet(self, other: Self) -> Option<Self>;
}

pub(super) use three_valued::{RThreeValuedBitvector, ThreeValuedBitvector};

#[cfg(not(feature = "Zdual_interval"))]
pub type Bitvector<const W: u32> = three_valued::ThreeValuedBitvector<W>;
#[cfg(not(feature = "Zdual_interval"))]
pub type RBitvector = three_valued::RThreeValuedBitvector;

#[cfg(feature = "Zdual_interval")]
pub type Bitvector<const W: u32> = combined::CombinedBitvector<W>;
#[cfg(feature = "Zdual_interval")]
pub type RBitvector = combined::RCombinedBitvector;

pub type BooleanBitvector = Bitvector<1>;
pub type PanicBitvector = Bitvector<32>;

impl<const W: u32> Abstr<super::concr::Bitvector<W>> for Bitvector<W> {
    fn from_concrete(value: super::concr::Bitvector<W>) -> Self {
        Self::from_concrete_value(value)
    }

    fn from_runtime(value: &AbstractValue) -> Self {
        Self::from_runtime_bitvector(*value.expect_bitvector())
    }

    fn to_runtime(&self) -> AbstractValue {
        AbstractValue::Bitvector(self.as_runtime_bitvector())
    }
}

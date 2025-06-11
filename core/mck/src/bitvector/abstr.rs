use super::concr::UnsignedInterval;
use crate::abstr::{BitvectorElement, ManipField, Phi};
use std::hash::Hash;

mod combined;
mod dual_interval;
mod three_valued;

pub trait BitvectorDomain<const W: u32>: Clone + Copy + Hash + Phi + ManipField {
    fn unsigned_interval(&self) -> UnsignedInterval<W>;
    fn element_description(&self) -> BitvectorElement;

    fn join(self, other: Self) -> Self;
    fn meet(self, other: Self) -> Option<Self>;
}

pub(super) use combined::CombinedBitvector;
pub(super) use three_valued::ThreeValuedBitvector;

pub(crate) use three_valued::format_zeros_ones;

#[cfg(not(feature = "Zdual_interval"))]
pub type Bitvector<const W: u32> = three_valued::ThreeValuedBitvector<W>;

#[cfg(feature = "Zdual_interval")]
pub type Bitvector<const W: u32> = combined::CombinedBitvector<W>;

pub type BooleanBitvector = Bitvector<1>;
pub type PanicBitvector = Bitvector<32>;

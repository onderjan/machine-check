mod combined;
mod dual_interval;
mod three_valued;

pub trait BitvectorDomain<const W: u32>: Clone + Copy + Hash + Phi + ManipField {
    fn unsigned_interval(&self) -> UnsignedInterval<W>;
    fn element_description(&self) -> ArrayFieldBitvector;
    fn three_valued(&self) -> &ThreeValuedBitvector<W>;
}

pub(super) use combined::CombinedBitvector;
pub(super) use three_valued::ThreeValuedBitvector;

use super::concr::UnsignedInterval;
use crate::abstr::{ArrayFieldBitvector, ManipField, Phi};
use std::hash::Hash;

pub(crate) use three_valued::format_zeros_ones;

// TODO: generic Bitvector
pub type Bitvector<const W: u32> = three_valued::ThreeValuedBitvector<W>;
//pub type Bitvector<const W: u32> = combined::CombinedBitvector<W>;

pub type BooleanBitvector = Bitvector<1>;
pub type PanicBitvector = Bitvector<32>;

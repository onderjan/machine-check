#[cfg(test)]
mod tests;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;

mod signed;
mod unsigned;

use serde::{Deserialize, Serialize};

use crate::bitvector::{bound::BitvectorBound, CBound, RBound};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConcreteBitvector<B: BitvectorBound> {
    bound: B,
    value: u64,
}

pub type RConcreteBitvector = ConcreteBitvector<RBound>;
pub type CConcreteBitvector<const W: u32> = ConcreteBitvector<CBound<W>>;

pub type PanicBitvector = ConcreteBitvector<CBound<32>>;

pub use signed::SignedBitvector;
pub use unsigned::UnsignedBitvector;

pub use ConcreteBitvector as Bitvector;

#[derive(Clone, Copy)]
pub struct OutsideBound<T> {
    width: u32,
    value: T,
    min_value: T,
    max_value: T,
}

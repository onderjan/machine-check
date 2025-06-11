#[cfg(test)]
mod tests;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;

use crate::concr::ConcreteBitvector;

#[derive(Clone, Copy, Hash)]
pub struct ThreeValuedBitvector<const L: u32> {
    zeros: ConcreteBitvector<L>,
    ones: ConcreteBitvector<L>,
}

pub use support::ThreeValuedFieldValue;

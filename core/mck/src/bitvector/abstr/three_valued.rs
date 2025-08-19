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
pub struct ThreeValuedBitvector<const W: u32> {
    zeros: ConcreteBitvector<W>,
    ones: ConcreteBitvector<W>,
}

pub use support::ThreeValuedFieldValue;

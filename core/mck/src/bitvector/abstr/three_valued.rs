#[cfg(test)]
mod tests;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;

use crate::{
    bitvector::{BitvectorBound, RBound},
    concr::ConcreteBitvector,
    misc::CBound,
};

#[derive(Clone, Copy, Hash, Serialize, Deserialize)]
pub struct ThreeValuedBitvector<B: BitvectorBound> {
    zeros: ConcreteBitvector<B>,
    ones: ConcreteBitvector<B>,
}

pub type RThreeValuedBitvector = ThreeValuedBitvector<RBound>;
pub type CThreeValuedBitvector<const W: u32> = ThreeValuedBitvector<CBound<W>>;

use serde::{Deserialize, Serialize};

pub struct InvalidZerosOnes;

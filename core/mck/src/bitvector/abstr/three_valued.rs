#[cfg(test)]
mod tests;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;

use crate::concr::{ConcreteBitvector, RConcreteBitvector};

#[derive(Clone, Copy, Hash, Serialize, Deserialize)]
pub struct RThreeValuedBitvector {
    zeros: RConcreteBitvector,
    ones: RConcreteBitvector,
}

#[derive(Clone, Copy, Hash, Serialize, Deserialize)]
pub struct ThreeValuedBitvector<const W: u32> {
    zeros: ConcreteBitvector<W>,
    ones: ConcreteBitvector<W>,
}

use serde::{Deserialize, Serialize};

pub struct InvalidZerosOnes;

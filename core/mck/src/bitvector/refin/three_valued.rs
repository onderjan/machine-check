use std::num::NonZeroU8;

use serde::{Deserialize, Serialize};

use crate::concr::RConcreteBitvector;

#[cfg(test)]
mod tests;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod meta;
mod shift;
mod support;

// TODO: remove equality in favour of meta-equality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RMarkBitvector {
    inner: Option<RBitvectorMark>,
    width: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RBitvectorMark {
    pub importance: NonZeroU8,
    pub mark: RConcreteBitvector,
}

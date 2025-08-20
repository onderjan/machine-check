use std::num::NonZeroU8;

use crate::concr::{ConcreteBitvector, RConcreteBitvector};

#[cfg(test)]
mod tests;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod meta;
mod refine;
mod shift;
mod support;

// TODO: remove equality in favour of meta-equality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RMarkBitvector {
    inner: Option<RBitvectorMark>,
    width: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RBitvectorMark {
    pub importance: NonZeroU8,
    pub mark: RConcreteBitvector,
}

// TODO: remove equality in favour of meta-equality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarkBitvector<const W: u32>(Option<BitvectorMark<W>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitvectorMark<const W: u32> {
    pub importance: NonZeroU8,
    pub mark: ConcreteBitvector<W>,
}

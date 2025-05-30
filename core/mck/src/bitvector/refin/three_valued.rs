use std::num::NonZeroU8;

use crate::concr::ConcreteBitvector;

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
pub struct MarkBitvector<const L: u32>(Option<BitvectorMark<L>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitvectorMark<const L: u32> {
    pub importance: NonZeroU8,
    pub mark: ConcreteBitvector<L>,
}

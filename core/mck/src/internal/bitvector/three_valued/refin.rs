use crate::bitvector::concrete::ConcreteBitvector;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarkBitvector<const L: u32>(ConcreteBitvector<L>);

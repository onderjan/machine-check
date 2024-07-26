use crate::bitvector::concrete::ConcreteBitvector;

#[cfg(test)]
mod tests;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;

#[derive(Clone, Copy, Hash)]
pub struct ThreeValuedBitvector<const L: u32> {
    zeros: ConcreteBitvector<L>,
    ones: ConcreteBitvector<L>,
}

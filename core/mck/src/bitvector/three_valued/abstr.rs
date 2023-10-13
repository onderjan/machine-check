use crate::concr;

#[cfg(test)]
mod test;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;

// the normal equality compares abstract bitvectors
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreeValuedBitvector<const L: u32> {
    zeros: concr::Bitvector<L>,
    ones: concr::Bitvector<L>,
}

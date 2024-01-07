mod arith;
mod support;
#[cfg(test)]
mod tests;

use crate::bitvector::concrete::ConcreteBitvector;

#[derive(Clone, Copy)]
pub struct Bitvector<const L: u32> {
    start: ConcreteBitvector<L>,
    end: ConcreteBitvector<L>,
}

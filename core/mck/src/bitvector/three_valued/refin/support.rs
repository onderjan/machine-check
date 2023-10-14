use crate::{
    bitvector::{concrete::ConcreteBitvector, three_valued::abstr::ThreeValuedBitvector},
    forward,
};

use super::MarkBitvector;

impl<const L: u32> MarkBitvector<L> {
    pub fn new_unmarked() -> Self {
        MarkBitvector(ConcreteBitvector::new(0))
    }
    pub fn new_marked() -> Self {
        if L == 0 {
            return Self(ConcreteBitvector::new(0));
        }
        let zero = ConcreteBitvector::new(0);
        let one = ConcreteBitvector::new(1);
        MarkBitvector(forward::HwArith::sub(zero, one))
    }
    pub fn new_from_flag(marked_flag: ConcreteBitvector<L>) -> Self {
        MarkBitvector(marked_flag)
    }
    pub(super) fn limit(&self, abstract_bitvec: ThreeValuedBitvector<L>) -> MarkBitvector<L> {
        MarkBitvector(forward::Bitwise::bit_and(
            self.0,
            abstract_bitvec.get_unknown_bits(),
        ))
    }
}

impl<const L: u32> Default for MarkBitvector<L> {
    fn default() -> Self {
        Self::new_unmarked()
    }
}

pub(super) fn default_uni_mark<const L: u32, const X: u32>(
    normal_input: (ThreeValuedBitvector<L>,),
    mark_later: MarkBitvector<X>,
) -> (MarkBitvector<L>,) {
    if mark_later == MarkBitvector::new_unmarked() {
        return (MarkBitvector::new_unmarked(),);
    }
    (MarkBitvector::new_marked().limit(normal_input.0),)
}

pub(super) fn default_bi_mark<const L: u32, const X: u32>(
    normal_input: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
    mark_later: MarkBitvector<X>,
) -> (MarkBitvector<L>, MarkBitvector<L>) {
    if mark_later == MarkBitvector::new_unmarked() {
        return (MarkBitvector::new_unmarked(), MarkBitvector::new_unmarked());
    }
    (
        MarkBitvector::new_marked().limit(normal_input.0),
        MarkBitvector::new_marked().limit(normal_input.1),
    )
}
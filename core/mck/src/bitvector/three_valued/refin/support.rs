use crate::{
    bitvector::{concrete::ConcreteBitvector, three_valued::abstr::ThreeValuedBitvector},
    forward,
    refin::{Boolean, ManipField, Refine},
    traits::misc::MetaEq,
};

use super::MarkBitvector;

impl<const L: u32> MarkBitvector<L> {
    pub fn new_unmarked() -> Self {
        Self {
            mark: ConcreteBitvector::new(0),
            importance: 0,
        }
    }
    pub fn new_marked(importance: u8) -> Self {
        if L == 0 {
            return Self {
                mark: ConcreteBitvector::new(0),
                importance: 0,
            };
        }
        let zero = ConcreteBitvector::new(0);
        let one = ConcreteBitvector::new(1);
        MarkBitvector {
            mark: forward::HwArith::sub(zero, one),
            importance,
        }
    }

    pub fn is_marked(&self) -> bool {
        self.mark.is_nonzero()
    }

    pub fn new_from_flag(marked_flag: ConcreteBitvector<L>) -> Self {
        MarkBitvector {
            mark: marked_flag,
            importance: 0,
        }
    }
    pub fn limit(&self, abstract_bitvec: ThreeValuedBitvector<L>) -> MarkBitvector<L> {
        MarkBitvector {
            mark: forward::Bitwise::bit_and(self.mark, abstract_bitvec.get_unknown_bits()),
            importance: self.importance,
        }
    }

    pub fn set_importance(&mut self, importance: u8) {
        self.importance = importance;
    }
}

pub(super) fn default_uni_mark<const L: u32, const X: u32>(
    normal_input: (ThreeValuedBitvector<L>,),
    mark_later: MarkBitvector<X>,
) -> (MarkBitvector<L>,) {
    if mark_later == MarkBitvector::new_unmarked() {
        return (MarkBitvector::new_unmarked(),);
    }
    (MarkBitvector::new_marked(mark_later.importance).limit(normal_input.0),)
}

pub(super) fn default_bi_mark<const L: u32, const X: u32>(
    normal_input: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
    mark_later: MarkBitvector<X>,
) -> (MarkBitvector<L>, MarkBitvector<L>) {
    if mark_later == MarkBitvector::new_unmarked() {
        return (MarkBitvector::new_unmarked(), MarkBitvector::new_unmarked());
    }
    (
        MarkBitvector::new_marked(mark_later.importance).limit(normal_input.0),
        MarkBitvector::new_marked(mark_later.importance).limit(normal_input.1),
    )
}

impl<const L: u32> MetaEq for MarkBitvector<L> {
    fn meta_eq(&self, other: &Self) -> bool {
        (self.mark, self.importance) == (other.mark, other.importance)
    }
}

impl<const L: u32> ManipField for MarkBitvector<L> {
    fn num_bits(&self) -> Option<u32> {
        Some(L)
    }

    fn mark(&mut self) {
        *self = Self::dirty();
    }

    fn index(&self, _index: u64) -> Option<&dyn ManipField> {
        None
    }

    fn index_mut(&mut self, _index: u64) -> Option<&mut dyn ManipField> {
        None
    }
}

impl From<Boolean> for MarkBitvector<1> {
    fn from(value: Boolean) -> Self {
        value.0
    }
}

impl From<MarkBitvector<1>> for Boolean {
    fn from(value: MarkBitvector<1>) -> Self {
        Boolean(value)
    }
}

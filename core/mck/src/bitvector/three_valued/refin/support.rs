use std::num::NonZeroU8;

use crate::{
    bitvector::{concrete::ConcreteBitvector, three_valued::abstr::ThreeValuedBitvector},
    forward::{self, HwArith},
    refin::{Boolean, ManipField, Refine},
    traits::misc::MetaEq,
};

use super::{BitvectorMark, MarkBitvector};

impl<const L: u32> MarkBitvector<L> {
    const LOWEST_IMPORTANCE: NonZeroU8 = Self::lowest_importance();

    const fn lowest_importance() -> NonZeroU8 {
        match NonZeroU8::new(1) {
            Some(result) => result,
            None => panic!("Number 1 should be non-zero"),
        }
    }

    pub fn new(mark: ConcreteBitvector<L>, importance: NonZeroU8) -> Self {
        if mark.is_nonzero() {
            Self(Some(BitvectorMark { mark, importance }))
        } else {
            Self(None)
        }
    }

    pub fn new_unmarked() -> Self {
        Self(None)
    }
    pub fn new_marked(importance: NonZeroU8) -> Self {
        if L == 0 {
            return Self::new_unmarked();
        }
        let zero = ConcreteBitvector::new(0);
        let one = ConcreteBitvector::new(1);
        // definitely nonzero
        Self(Some(BitvectorMark {
            mark: HwArith::sub(zero, one),
            importance,
        }))
    }

    pub fn new_marked_unimportant() -> Self {
        Self::new_marked(Self::LOWEST_IMPORTANCE)
    }

    pub fn is_marked(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_unmarked(&self) -> bool {
        !self.is_marked()
    }

    pub fn new_from_flag(mark: ConcreteBitvector<L>) -> Self {
        Self::new(mark, Self::LOWEST_IMPORTANCE)
    }
    pub fn limit(&self, abstract_bitvec: ThreeValuedBitvector<L>) -> MarkBitvector<L> {
        if let Some(own_mark) = self.0 {
            let result_mark =
                forward::Bitwise::bit_and(own_mark.mark, abstract_bitvec.get_unknown_bits());
            Self::new(result_mark, own_mark.importance)
        } else {
            Self::new_unmarked()
        }
    }

    pub fn marked_bits(&self) -> ConcreteBitvector<L> {
        if let Some(mark) = self.0 {
            mark.mark
        } else {
            ConcreteBitvector::zero()
        }
    }

    pub fn get(&self) -> &Option<BitvectorMark<L>> {
        &self.0
    }
}

pub(super) fn default_uni_mark<const L: u32, const X: u32>(
    normal_input: (ThreeValuedBitvector<L>,),
    mark_later: MarkBitvector<X>,
) -> (MarkBitvector<L>,) {
    let Some(mark_later) = mark_later.0 else {
        return (MarkBitvector::new_unmarked(),);
    };
    (MarkBitvector::new_marked(mark_later.importance).limit(normal_input.0),)
}

pub(super) fn default_bi_mark<const L: u32, const X: u32>(
    normal_input: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
    mark_later: MarkBitvector<X>,
) -> (MarkBitvector<L>, MarkBitvector<L>) {
    let Some(mark_later) = mark_later.0 else {
        return (MarkBitvector::new_unmarked(), MarkBitvector::new_unmarked());
    };
    (
        MarkBitvector::new_marked(mark_later.importance).limit(normal_input.0),
        MarkBitvector::new_marked(mark_later.importance).limit(normal_input.1),
    )
}

impl<const L: u32> MetaEq for MarkBitvector<L> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.0 == other.0
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

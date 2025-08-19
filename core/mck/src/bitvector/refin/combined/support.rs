use std::num::NonZeroU8;

use crate::{
    bitvector::{
        abstr::CombinedBitvector,
        refin::{
            three_valued::{BitvectorMark, MarkBitvector},
            FromRefin,
        },
    },
    refin::{Boolean, ManipField, Refine},
    traits::misc::MetaEq,
};

use super::CombinedMark;

impl<const W: u32> CombinedMark<W> {
    pub fn new_unmarked() -> Self {
        Self(MarkBitvector::new_unmarked())
    }

    pub fn new_marked_unimportant() -> Self {
        Self(MarkBitvector::new_marked_unimportant())
    }

    pub fn new_marked(importance: NonZeroU8) -> Self {
        Self(MarkBitvector::new_marked(importance))
    }

    pub fn is_marked(&self) -> bool {
        self.0.is_marked()
    }

    pub fn limit(&self, abstract_bitvec: CombinedBitvector<W>) -> CombinedMark<W> {
        Self(self.0.limit(*abstract_bitvec.three_valued()))
    }

    pub fn get(&self) -> &Option<BitvectorMark<W>> {
        self.0.get()
    }
}

impl<const W: u32> MetaEq for CombinedMark<W> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<const W: u32> ManipField for CombinedMark<W> {
    fn num_bits(&self) -> Option<u32> {
        Some(W)
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

impl From<Boolean> for CombinedMark<1> {
    fn from(value: Boolean) -> Self {
        Self(FromRefin::from_refin(value.0))
    }
}

impl From<CombinedMark<1>> for Boolean {
    fn from(value: CombinedMark<1>) -> Self {
        Self(FromRefin::from_refin(value.0))
    }
}

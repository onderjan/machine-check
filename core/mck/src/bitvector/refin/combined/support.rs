use std::num::NonZeroU8;

use crate::{
    bitvector::{abstr::CombinedBitvector, refin::three_valued::BitvectorMark},
    concr::ConcreteBitvector,
    forward::{self, HwArith},
    refin::{Boolean, ManipField, Refine},
    traits::misc::MetaEq,
};

use super::CombinedMark;

impl<const L: u32> CombinedMark<L> {
    pub fn new(mark: ConcreteBitvector<L>, importance: NonZeroU8) -> Self {
        todo!()
    }

    pub fn new_unmarked() -> Self {
        todo!()
    }
    pub fn new_marked(importance: NonZeroU8) -> Self {
        todo!()
    }

    pub fn new_marked_unimportant() -> Self {
        todo!()
    }

    pub fn is_marked(&self) -> bool {
        todo!()
    }

    pub fn is_unmarked(&self) -> bool {
        todo!()
    }

    pub fn new_from_flag(mark: ConcreteBitvector<L>) -> Self {
        todo!()
    }
    pub fn limit(&self, abstract_bitvec: CombinedBitvector<L>) -> CombinedMark<L> {
        todo!()
    }

    pub fn marked_bits(&self) -> ConcreteBitvector<L> {
        todo!()
    }

    pub fn get(&self) -> &Option<BitvectorMark<L>> {
        todo!()
    }
}

impl<const L: u32> MetaEq for CombinedMark<L> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<const L: u32> ManipField for CombinedMark<L> {
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

impl From<Boolean> for CombinedMark<1> {
    fn from(value: Boolean) -> Self {
        todo!()
    }
}

impl From<CombinedMark<1>> for Boolean {
    fn from(value: CombinedMark<1>) -> Self {
        todo!()
    }
}

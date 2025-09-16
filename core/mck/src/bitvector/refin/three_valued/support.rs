use std::num::NonZeroU8;

use crate::{
    bitvector::{
        abstr::{RThreeValuedBitvector, ThreeValuedBitvector},
        refin::{
            three_valued::{RBitvectorMark, RMarkBitvector},
            FromRefin,
        },
    },
    concr::{ConcreteBitvector, RConcreteBitvector},
    forward::{self, HwArith},
    refin::{Boolean, ManipField, Refine},
    traits::misc::MetaEq,
};

use super::{BitvectorMark, MarkBitvector};

impl RMarkBitvector {
    pub fn new(mark: RConcreteBitvector, importance: NonZeroU8, width: u32) -> Self {
        assert_eq!(mark.width(), width);
        let inner = if mark.is_nonzero() {
            Some(RBitvectorMark { mark, importance })
        } else {
            None
        };
        Self { inner, width }
    }

    pub fn new_unmarked(width: u32) -> Self {
        Self { inner: None, width }
    }
    pub fn new_marked(importance: NonZeroU8, width: u32) -> Self {
        if width == 0 {
            return Self::new_unmarked(width);
        }
        let zero = RConcreteBitvector::new(0, width);
        let one = RConcreteBitvector::new(1, width);
        // definitely nonzero
        Self {
            inner: Some(RBitvectorMark {
                mark: HwArith::sub(zero, one),
                importance,
            }),
            width,
        }
    }

    pub fn new_marked_unimportant(width: u32) -> Self {
        Self::new_marked(MarkBitvector::<64>::LOWEST_IMPORTANCE, width)
    }

    pub fn limit(&self, abstract_bitvec: RThreeValuedBitvector) -> RMarkBitvector {
        assert_eq!(self.width, abstract_bitvec.width());
        if let Some(own_mark) = self.inner {
            let result_mark =
                forward::Bitwise::bit_and(own_mark.mark, abstract_bitvec.get_unknown_bits());
            Self::new(result_mark, own_mark.importance, self.width)
        } else {
            Self::new_unmarked(self.width)
        }
    }

    pub fn marked_bits(&self) -> RConcreteBitvector {
        if let Some(mark) = self.inner {
            mark.mark
        } else {
            RConcreteBitvector::new(0, self.width)
        }
    }
}

impl<const W: u32> MarkBitvector<W> {
    const LOWEST_IMPORTANCE: NonZeroU8 = Self::lowest_importance();

    const fn lowest_importance() -> NonZeroU8 {
        match NonZeroU8::new(1) {
            Some(result) => result,
            None => panic!("Number 1 should be non-zero"),
        }
    }

    pub fn new(mark: ConcreteBitvector<W>, importance: NonZeroU8) -> Self {
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
        if W == 0 {
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

    pub fn new_from_flag(mark: ConcreteBitvector<W>) -> Self {
        Self::new(mark, Self::LOWEST_IMPORTANCE)
    }
    pub fn limit(&self, abstract_bitvec: ThreeValuedBitvector<W>) -> MarkBitvector<W> {
        if let Some(own_mark) = self.0 {
            let result_mark =
                forward::Bitwise::bit_and(own_mark.mark, abstract_bitvec.get_unknown_bits());
            Self::new(result_mark, own_mark.importance)
        } else {
            Self::new_unmarked()
        }
    }

    pub fn marked_bits(&self) -> ConcreteBitvector<W> {
        if let Some(mark) = self.0 {
            mark.mark
        } else {
            ConcreteBitvector::zero()
        }
    }

    pub fn get(&self) -> &Option<BitvectorMark<W>> {
        &self.0
    }

    fn to_runtime(self) -> RMarkBitvector {
        RMarkBitvector {
            inner: self.0.map(|inner| RBitvectorMark {
                importance: inner.importance,
                mark: inner.mark.to_runtime(),
            }),
            width: W,
        }
    }
}

pub(super) fn runtime_default_uni_mark(
    normal_input: (RThreeValuedBitvector,),
    mark_later: RMarkBitvector,
) -> (RMarkBitvector,) {
    // normal input and earlier mark (result) have the same width
    // mark later can have another width

    let Some(mark_later) = mark_later.inner else {
        return (RMarkBitvector::new_unmarked(normal_input.0.width()),);
    };
    (
        RMarkBitvector::new_marked(mark_later.importance, normal_input.0.width())
            .limit(normal_input.0),
    )
}

pub(super) fn runtime_default_bi_mark(
    normal_input: (RThreeValuedBitvector, RThreeValuedBitvector),
    mark_later: RMarkBitvector,
) -> (RMarkBitvector, RMarkBitvector) {
    assert_eq!(normal_input.0.width(), normal_input.1.width());
    let width = normal_input.0.width();

    // normal inputs and earlier marks (result parts) have the same width
    // mark later can have another width

    let Some(mark_later) = mark_later.inner else {
        return (
            RMarkBitvector::new_unmarked(width),
            RMarkBitvector::new_unmarked(width),
        );
    };
    (
        RMarkBitvector::new_marked(mark_later.importance, width).limit(normal_input.0),
        RMarkBitvector::new_marked(mark_later.importance, width).limit(normal_input.1),
    )
}

pub(super) fn default_uni_mark<const W: u32, const X: u32>(
    normal_input: (ThreeValuedBitvector<W>,),
    mark_later: MarkBitvector<X>,
) -> (MarkBitvector<W>,) {
    let Some(mark_later) = mark_later.0 else {
        return (MarkBitvector::new_unmarked(),);
    };
    (MarkBitvector::new_marked(mark_later.importance).limit(normal_input.0),)
}

pub(super) fn default_bi_mark<const W: u32, const X: u32>(
    normal_input: (ThreeValuedBitvector<W>, ThreeValuedBitvector<W>),
    mark_later: MarkBitvector<X>,
) -> (MarkBitvector<W>, MarkBitvector<W>) {
    let Some(mark_later) = mark_later.0 else {
        return (MarkBitvector::new_unmarked(), MarkBitvector::new_unmarked());
    };
    (
        MarkBitvector::new_marked(mark_later.importance).limit(normal_input.0),
        MarkBitvector::new_marked(mark_later.importance).limit(normal_input.1),
    )
}

impl<const W: u32> MetaEq for MarkBitvector<W> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<const W: u32> ManipField for MarkBitvector<W> {
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

impl From<Boolean> for MarkBitvector<1> {
    fn from(value: Boolean) -> Self {
        FromRefin::from_refin(value.0)
    }
}

impl From<MarkBitvector<1>> for Boolean {
    fn from(value: MarkBitvector<1>) -> Self {
        Boolean(FromRefin::from_refin(value))
    }
}

impl Boolean {
    pub(crate) fn to_runtime_bitvector(self) -> RMarkBitvector {
        self.0.to_runtime()
    }
}

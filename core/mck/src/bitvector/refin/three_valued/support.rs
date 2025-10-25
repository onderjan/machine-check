use std::num::NonZeroU8;

use crate::{
    bitvector::{
        abstr::three_valued::RThreeValuedBitvector,
        refin::three_valued::{RBitvectorMark, RMarkBitvector},
        RBound,
    },
    concr::RConcreteBitvector,
    forward::{self, HwArith},
    misc::BitvectorBound,
    traits::misc::MetaEq,
};

impl RMarkBitvector {
    const LOWEST_IMPORTANCE: NonZeroU8 = NonZeroU8::new(1).unwrap();

    pub fn new(mark: RConcreteBitvector, importance: NonZeroU8, width: u32) -> Self {
        assert_eq!(mark.bound().width(), width);
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
        let bound = RBound::new(width);
        let zero = RConcreteBitvector::new(0, bound);
        let one = RConcreteBitvector::new(1, bound);
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
        Self::new_marked(Self::LOWEST_IMPORTANCE, width)
    }

    pub fn new_from_flag(mark: RConcreteBitvector) -> Self {
        Self::new(mark, Self::LOWEST_IMPORTANCE, mark.bound().width())
    }

    pub fn marked_bits(&self) -> RConcreteBitvector {
        if let Some(mark) = self.inner {
            mark.mark
        } else {
            RConcreteBitvector::new(0, RBound::new(self.width))
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }
}

impl RMarkBitvector {
    pub fn limit(self, abstract_bitvec: &RThreeValuedBitvector) -> Self {
        assert_eq!(self.width, abstract_bitvec.width());
        if let Some(own_mark) = self.inner {
            let result_mark =
                forward::Bitwise::bit_and(own_mark.mark, abstract_bitvec.get_unknown_bits());
            Self::new(result_mark, own_mark.importance, self.width)
        } else {
            Self::new_unmarked(self.width)
        }
    }
}

impl MetaEq for RMarkBitvector {
    fn meta_eq(&self, other: &Self) -> bool {
        assert_eq!(self.width, other.width);
        self.inner == other.inner
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
            .limit(&normal_input.0),
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
        RMarkBitvector::new_marked(mark_later.importance, width).limit(&normal_input.0),
        RMarkBitvector::new_marked(mark_later.importance, width).limit(&normal_input.1),
    )
}

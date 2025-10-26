use std::num::NonZeroU8;

use crate::{
    abstr::BitvectorDomain,
    bitvector::{
        abstr::three_valued::RThreeValuedBitvector,
        refin::three_valued::{RBitvectorMark, RMarkBitvector},
        util::compute_u64_sign_bit_mask,
        RBound,
    },
    concr::RConcreteBitvector,
    forward::{self, HwArith},
    misc::BitvectorBound,
    refin::{Boolean, RefinementDomain},
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
    let width = normal_input.0.bound().width();

    let Some(mark_later) = mark_later.inner else {
        return (RMarkBitvector::new_unmarked(width),);
    };
    (RMarkBitvector::new_marked(mark_later.importance, width).limit(&normal_input.0),)
}

pub(super) fn runtime_default_bi_mark(
    normal_input: (RThreeValuedBitvector, RThreeValuedBitvector),
    mark_later: RMarkBitvector,
) -> (RMarkBitvector, RMarkBitvector) {
    assert_eq!(normal_input.0.bound(), normal_input.1.bound());
    let width = normal_input.0.bound().width();

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

impl RefinementDomain for RMarkBitvector {
    type Abstr = RThreeValuedBitvector;
    type Bound = RBound;

    fn bound(&self) -> Self::Bound {
        RBound::new(self.width)
    }

    fn new_unmarked(width: u32) -> Self {
        Self { inner: None, width }
    }

    fn new_marked(importance: NonZeroU8, width: u32) -> Self {
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

    fn from_boolean(boolean: crate::refin::Boolean) -> Self {
        if let Some(importance) = NonZeroU8::new(boolean.importance()) {
            Self::new_marked(importance, 1)
        } else {
            Self::new_unmarked(1)
        }
    }

    fn limit(self, abstract_bitvec: &RThreeValuedBitvector) -> Self {
        assert_eq!(self.width, abstract_bitvec.bound().width());
        if let Some(own_mark) = self.inner {
            let result_mark =
                forward::Bitwise::bit_and(own_mark.mark, abstract_bitvec.get_unknown_bits());
            Self::new(result_mark, own_mark.importance, self.width)
        } else {
            Self::new_unmarked(self.width)
        }
    }

    fn apply_join(&mut self, other: &Self) {
        assert_eq!(self.width, other.width);
        let Some(other_mark) = other.inner else {
            return;
        };
        if let Some(our_mark) = &mut self.inner {
            our_mark.mark = forward::Bitwise::bit_or(our_mark.mark, other_mark.mark);
            our_mark.importance = our_mark.importance.max(other_mark.importance);
        } else {
            // other mark should be nonzero
            self.inner = Some(other_mark);
        }
    }

    fn to_condition(self) -> Boolean {
        if let Some(our_mark) = self.inner {
            assert!(our_mark.mark.is_nonzero());
            Boolean::new_marked(our_mark.importance)
        } else {
            Boolean::new_unmarked()
        }
    }

    fn apply_refin(&mut self, offer: &Self) -> bool {
        assert_eq!(self.width, offer.width);
        let width = self.width;

        // return if the offer is unmarked
        let Some(offer_mark) = offer.inner else {
            return false;
        };

        // find the highest bit that is marked in offer but unmarked in ours
        let applicants = if let Some(our_mark) = self.inner {
            forward::Bitwise::bit_and(offer_mark.mark, forward::Bitwise::bit_not(our_mark.mark))
        } else {
            offer_mark.mark
        };
        if applicants.is_zero() {
            // no such bit found
            return false;
        }

        let highest_applicant_pos = applicants.to_u64().ilog2();
        let highest_applicant = RConcreteBitvector::new(
            compute_u64_sign_bit_mask(highest_applicant_pos + 1),
            RBound::new(width),
        );
        assert!(highest_applicant.is_nonzero());

        // apply the mark
        if let Some(our_mark) = &mut self.inner {
            our_mark.mark = forward::Bitwise::bit_or(our_mark.mark, highest_applicant);
            our_mark.importance = our_mark.importance.max(offer_mark.importance);
        } else {
            // highest applicant is definitely nonzero
            self.inner = Some(RBitvectorMark {
                importance: offer_mark.importance,
                mark: highest_applicant,
            });
        }
        true
    }

    fn force_decay(&self, target: &mut RThreeValuedBitvector) {
        assert_eq!(self.width, target.bound().width());

        // unmarked fields become unknown
        let forced_unknown = forward::Bitwise::bit_not(self.marked_bits());
        let zeros = forward::Bitwise::bit_or(target.get_possibly_zero_flags(), forced_unknown);
        let ones = forward::Bitwise::bit_or(target.get_possibly_one_flags(), forced_unknown);
        *target = RThreeValuedBitvector::from_zeros_ones(zeros, ones);
    }

    fn importance(&self) -> u8 {
        if let Some(inner) = self.inner {
            inner.importance.get()
        } else {
            0
        }
    }
}

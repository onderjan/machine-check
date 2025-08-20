use crate::{
    bitvector::{
        abstr::{RThreeValuedBitvector, ThreeValuedBitvector},
        concr::ConcreteBitvector,
        refin::three_valued::{BitvectorMark, RBitvectorMark, RMarkBitvector},
        util::compute_u64_sign_bit_mask,
    },
    concr::RConcreteBitvector,
    forward,
    refin::{Boolean, Refine},
};

use super::MarkBitvector;

impl RMarkBitvector {
    pub fn apply_join(&mut self, other: &Self) {
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

    fn to_condition(&self) -> Boolean {
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
        let highest_applicant =
            RConcreteBitvector::new(compute_u64_sign_bit_mask(highest_applicant_pos + 1), width);
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
        assert_eq!(self.width, target.width());

        // unmarked fields become unknown
        let forced_unknown = forward::Bitwise::bit_not(self.marked_bits());
        let zeros = forward::Bitwise::bit_or(target.get_possibly_zero_flags(), forced_unknown);
        let ones = forward::Bitwise::bit_or(target.get_possibly_one_flags(), forced_unknown);
        *target = RThreeValuedBitvector::from_zeros_ones(zeros, ones);
    }

    fn clean(width: u32) -> Self {
        Self::new_unmarked(width)
    }

    fn dirty(width: u32) -> Self {
        Self::new_marked_unimportant(width)
    }

    fn importance(&self) -> u8 {
        if let Some(mark) = self.inner {
            mark.importance.into()
        } else {
            0
        }
    }
}

impl<const W: u32> Refine<ThreeValuedBitvector<W>> for MarkBitvector<W> {
    fn apply_join(&mut self, other: &Self) {
        let Some(other_mark) = other.0 else {
            return;
        };
        if let Some(our_mark) = &mut self.0 {
            our_mark.mark = forward::Bitwise::bit_or(our_mark.mark, other_mark.mark);
            our_mark.importance = our_mark.importance.max(other_mark.importance);
        } else {
            // other mark should be nonzero
            self.0 = Some(other_mark);
        }
    }

    fn to_condition(&self) -> Boolean {
        if let Some(our_mark) = self.0 {
            assert!(our_mark.mark.is_nonzero());
            Boolean::new_marked(our_mark.importance)
        } else {
            Boolean::new_unmarked()
        }
    }

    fn apply_refin(&mut self, offer: &Self) -> bool {
        // return if the offer is unmarked
        let Some(offer_mark) = offer.0 else {
            return false;
        };

        // find the highest bit that is marked in offer but unmarked in ours
        let applicants = if let Some(our_mark) = self.0 {
            forward::Bitwise::bit_and(offer_mark.mark, forward::Bitwise::bit_not(our_mark.mark))
        } else {
            offer_mark.mark
        };
        if applicants.is_zero() {
            // no such bit found
            return false;
        }

        let highest_applicant_pos = applicants.to_u64().ilog2();
        let highest_applicant =
            ConcreteBitvector::new(compute_u64_sign_bit_mask(highest_applicant_pos + 1));
        assert!(highest_applicant.is_nonzero());

        // apply the mark
        if let Some(our_mark) = &mut self.0 {
            our_mark.mark = forward::Bitwise::bit_or(our_mark.mark, highest_applicant);
            our_mark.importance = our_mark.importance.max(offer_mark.importance);
        } else {
            // highest applicant is definitely nonzero
            self.0 = Some(BitvectorMark {
                importance: offer_mark.importance,
                mark: highest_applicant,
            });
        }
        true
    }

    fn force_decay(&self, target: &mut ThreeValuedBitvector<W>) {
        // unmarked fields become unknown
        let forced_unknown = forward::Bitwise::bit_not(self.marked_bits());
        let zeros = forward::Bitwise::bit_or(target.get_possibly_zero_flags(), forced_unknown);
        let ones = forward::Bitwise::bit_or(target.get_possibly_one_flags(), forced_unknown);
        *target = ThreeValuedBitvector::from_zeros_ones(zeros, ones);
    }

    fn clean() -> Self {
        Self::new_unmarked()
    }

    fn dirty() -> Self {
        Self::new_marked_unimportant()
    }

    fn importance(&self) -> u8 {
        if let Some(mark) = self.0 {
            mark.importance.into()
        } else {
            0
        }
    }
}

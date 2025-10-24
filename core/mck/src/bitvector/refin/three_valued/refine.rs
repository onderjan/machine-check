use crate::{
    bitvector::{
        abstr::RThreeValuedBitvector,
        refin::three_valued::{RBitvectorMark, RMarkBitvector},
        util::compute_u64_sign_bit_mask,
    },
    concr::RConcreteBitvector,
    forward,
    refin::Boolean,
};

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

    pub fn to_condition(self) -> Boolean {
        if let Some(our_mark) = self.inner {
            assert!(our_mark.mark.is_nonzero());
            Boolean::new_marked(our_mark.importance)
        } else {
            Boolean::new_unmarked()
        }
    }

    pub fn apply_refin(&mut self, offer: &Self) -> bool {
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

    pub fn force_decay(&self, target: &mut RThreeValuedBitvector) {
        assert_eq!(self.width, target.width());

        // unmarked fields become unknown
        let forced_unknown = forward::Bitwise::bit_not(self.marked_bits());
        let zeros = forward::Bitwise::bit_or(target.get_possibly_zero_flags(), forced_unknown);
        let ones = forward::Bitwise::bit_or(target.get_possibly_one_flags(), forced_unknown);
        *target = RThreeValuedBitvector::from_zeros_ones(zeros, ones);
    }

    pub fn importance(&self) -> u8 {
        self.inner.map(|mark| mark.importance.get()).unwrap_or(0)
    }
}

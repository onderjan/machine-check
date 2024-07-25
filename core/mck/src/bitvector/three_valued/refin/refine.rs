use crate::{
    bitvector::{
        concrete::ConcreteBitvector, three_valued::abstr::ThreeValuedBitvector,
        util::compute_u64_sign_bit_mask,
    },
    forward,
    refin::{Boolean, Refine},
};

use super::MarkBitvector;

impl<const L: u32> Refine<ThreeValuedBitvector<L>> for MarkBitvector<L> {
    fn apply_join(&mut self, other: &Self) {
        if other.mark.is_zero() {
            return;
        }
        self.mark = forward::Bitwise::bit_or(self.mark, other.mark);
        self.importance = self.importance.max(other.importance);
    }

    fn to_condition(&self) -> Boolean {
        if self.mark.is_nonzero() {
            Boolean::new_marked(self.importance)
        } else {
            Boolean::new_unmarked()
        }
    }

    fn apply_refin(&mut self, offer: &Self) -> bool {
        // find the highest bit that is marked in offer but unmarked in ours
        let applicants =
            forward::Bitwise::bit_and(offer.mark, forward::Bitwise::bit_not(self.mark));
        if applicants.is_zero() {
            // no such bit found
            return false;
        }

        let highest_applicant_pos = applicants.as_unsigned().ilog2();
        let highest_applicant =
            ConcreteBitvector::new(compute_u64_sign_bit_mask(highest_applicant_pos + 1));

        // apply the mark
        self.mark = forward::Bitwise::bit_or(self.mark, highest_applicant);
        self.importance = self.importance.max(offer.importance);
        true
    }

    fn force_decay(&self, target: &mut ThreeValuedBitvector<L>) {
        // unmarked fields become unknown
        let forced_unknown = forward::Bitwise::bit_not(self.mark);
        let zeros = forward::Bitwise::bit_or(target.get_possibly_zero_flags(), forced_unknown);
        let ones = forward::Bitwise::bit_or(target.get_possibly_one_flags(), forced_unknown);
        *target = ThreeValuedBitvector::from_zeros_ones(zeros, ones);
    }

    fn clean() -> Self {
        Self::new_unmarked()
    }

    fn dirty() -> Self {
        Self::new_marked(0)
    }

    fn importance(&self) -> u8 {
        self.importance
    }
}

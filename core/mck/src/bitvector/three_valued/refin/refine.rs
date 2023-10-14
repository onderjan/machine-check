use crate::{
    bitvector::{
        concr, three_valued::abstr::ThreeValuedBitvector, util::compute_u64_sign_bit_mask,
    },
    forward,
    refin::{Refinable, Refine},
};

use super::MarkBitvector;

impl<const L: u32> Refine<ThreeValuedBitvector<L>> for MarkBitvector<L> {
    fn apply_join(&mut self, other: &Self) {
        self.0 = forward::Bitwise::bitor(self.0, other.0);
    }

    fn apply_refin(&mut self, offer: &Self) -> bool {
        // find the highest bit that is marked in offer but unmarked in ours
        let applicants = forward::Bitwise::bitand(offer.0, forward::Bitwise::not(self.0));
        if applicants.is_zero() {
            // no such bit found
            return false;
        }

        let highest_applicant_pos = applicants.as_unsigned().ilog2();
        let highest_applicant =
            concr::Bitvector::new(compute_u64_sign_bit_mask(highest_applicant_pos + 1));

        // apply the mark
        self.0 = forward::Bitwise::bitor(self.0, highest_applicant);
        true
    }

    fn force_decay(&self, target: &mut ThreeValuedBitvector<L>) {
        // unmarked fields become unknown
        let forced_unknown = forward::Bitwise::not(self.0);
        let zeros = forward::Bitwise::bitor(target.get_possibly_zero_flags(), forced_unknown);
        let ones = forward::Bitwise::bitor(target.get_possibly_one_flags(), forced_unknown);
        *target = ThreeValuedBitvector::from_zeros_ones(zeros, ones);
    }
}

impl<const L: u32> Refinable for ThreeValuedBitvector<L> {
    type Refin = MarkBitvector<L>;

    fn clean_refin(&self) -> Self::Refin {
        MarkBitvector::new_unmarked()
    }
}

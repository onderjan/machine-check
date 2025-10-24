/*use crate::{
    abstr::combined::RCombinedBitvector, bitvector::refin::combined::RCombinedMark, refin::Boolean,
};

impl RCombinedMark {
    fn apply_join(&mut self, other: &Self) {
        self.0.apply_join(&other.0);
    }

    fn to_condition(&self) -> Boolean {
        self.0.to_condition()
    }

    fn apply_refin(&mut self, offer: &Self) -> bool {
        self.0.apply_refin(&offer.0)
    }

    fn force_decay(&self, target: &mut RCombinedBitvector) {
        // TODO: force decay on both

        let mut three_valued = *target.three_valued();
        self.0.force_decay(&mut three_valued);

        *target = RCombinedBitvector::combine(three_valued, *target.dual_interval());
    }

    fn importance(&self) -> u8 {
        self.0.importance()
    }
}
*/

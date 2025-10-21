use crate::{
    bitvector::{
        abstr::{RThreeValuedBitvector, ThreeValuedBitvector},
        concr::ConcreteBitvector,
        refin::three_valued::RMarkBitvector,
    },
    concr::RConcreteBitvector,
    traits::misc::Meta,
};

use super::MarkBitvector;

impl Meta<RThreeValuedBitvector> for RMarkBitvector {
    fn proto_first(&self) -> RThreeValuedBitvector {
        // all known bits are 0
        let known_bits = self.marked_bits().to_u64();
        RThreeValuedBitvector::new_value_known(
            RConcreteBitvector::new(0, self.width),
            RConcreteBitvector::new(known_bits, self.width),
        )
    }

    fn proto_increment(&self, proto: &mut RThreeValuedBitvector) -> bool {
        // the marked bits should be split into possibilities
        let known_bits = self.marked_bits().to_u64();

        if known_bits == 0 {
            // if full-unknown, stop immediately after first to avoid shl overflow
            return false;
        }

        // manual addition-style updates: only update marked positions
        // start with lowest marked position
        // if it is 0 within current, update it to 1 and end
        // if it is 1, update it to 0, temporarily forget mark and update next
        // end if we overflow

        // work with bitvector of only values, the unknowns do not change
        let mut current = proto.umin().to_u64();
        let mut considered_bits = known_bits;

        loop {
            let one_pos = considered_bits.trailing_zeros();
            let one_mask = 1u64 << one_pos;
            if current & one_mask == 0 {
                // if considered bit is 0 within current, update it to 1 and end
                current |= one_mask;
                let result = RThreeValuedBitvector::new_value_known(
                    RConcreteBitvector::new(current, self.width),
                    RConcreteBitvector::new(known_bits, self.width),
                );

                *proto = result;
                return true;
            }
            // if it is 1, update it to 0, temporarily do not consider it and update next
            current &= !one_mask;
            considered_bits &= !one_mask;

            // end if we overflow
            // reset possibility to allow for cycling
            if considered_bits == 0 {
                *proto = self.proto_first();
                return false;
            }
        }
    }
}

impl<const W: u32> Meta<ThreeValuedBitvector<W>> for MarkBitvector<W> {
    fn proto_first(&self) -> ThreeValuedBitvector<W> {
        // all known bits are 0
        let known_bits = self.marked_bits().to_u64();
        ThreeValuedBitvector::new_value_known(
            ConcreteBitvector::new(0),
            ConcreteBitvector::new(known_bits),
        )
    }

    fn proto_increment(&self, proto: &mut ThreeValuedBitvector<W>) -> bool {
        // the marked bits should be split into possibilities
        let known_bits = self.marked_bits().to_u64();

        if known_bits == 0 {
            // if full-unknown, stop immediately after first to avoid shl overflow
            return false;
        }

        // manual addition-style updates: only update marked positions
        // start with lowest marked position
        // if it is 0 within current, update it to 1 and end
        // if it is 1, update it to 0, temporarily forget mark and update next
        // end if we overflow

        // work with bitvector of only values, the unknowns do not change
        let mut current = proto.umin().to_u64();
        let mut considered_bits = known_bits;

        loop {
            let one_pos = considered_bits.trailing_zeros();
            let one_mask = 1u64 << one_pos;
            if current & one_mask == 0 {
                // if considered bit is 0 within current, update it to 1 and end
                current |= one_mask;
                let result = ThreeValuedBitvector::new_value_known(
                    ConcreteBitvector::new(current),
                    ConcreteBitvector::new(known_bits),
                );

                *proto = result;
                return true;
            }
            // if it is 1, update it to 0, temporarily do not consider it and update next
            current &= !one_mask;
            considered_bits &= !one_mask;

            // end if we overflow
            // reset possibility to allow for cycling
            if considered_bits == 0 {
                *proto = self.proto_first();
                return false;
            }
        }
    }
}

use crate::{
    bitvector::concr,
    forward::{Bitwise, TypedEq},
};

use super::ThreeValuedBitvector;

impl<const L: u32> TypedEq for ThreeValuedBitvector<L> {
    type Output = ThreeValuedBitvector<1>;
    fn typed_eq(self, rhs: Self) -> Self::Output {
        // result can be true if all bits can be the same
        // result can be false if at least one bit can be different

        let can_be_same_bits = (self.zeros.bitand(rhs.zeros)).bitor(self.ones.bitand(rhs.ones));
        let can_be_different_bits =
            (self.zeros.bitand(rhs.ones)).bitor(self.ones.bitand(rhs.zeros));

        let can_be_same = can_be_same_bits.is_full_mask();
        let can_be_different = can_be_different_bits.is_nonzero();

        Self::Output::from_zeros_ones(
            concr::Bitvector::new(can_be_different as u64),
            concr::Bitvector::new(can_be_same as u64),
        )
    }
}

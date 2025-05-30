use crate::{
    abstr::Boolean,
    concr::ConcreteBitvector,
    forward::{Bitwise, TypedEq},
};

use super::ThreeValuedBitvector;

impl<const L: u32> TypedEq for ThreeValuedBitvector<L> {
    type Output = Boolean;
    fn eq(self, rhs: Self) -> Self::Output {
        // result can be true if all bits can be the same
        // result can be false if at least one bit can be different

        let can_be_same_bits = (self.zeros.bit_and(rhs.zeros)).bit_or(self.ones.bit_and(rhs.ones));
        let can_be_different_bits =
            (self.zeros.bit_and(rhs.ones)).bit_or(self.ones.bit_and(rhs.zeros));

        let can_be_same = can_be_same_bits.is_full_mask();
        let can_be_different = can_be_different_bits.is_nonzero();

        Self::Output::from_zeros_ones(
            ConcreteBitvector::new(can_be_different as u64),
            ConcreteBitvector::new(can_be_same as u64),
        )
    }

    fn ne(self, rhs: Self) -> Self::Output {
        self.eq(rhs).bit_not()
    }
}

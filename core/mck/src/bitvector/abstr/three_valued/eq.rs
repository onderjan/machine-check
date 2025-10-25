use crate::{
    abstr::Boolean,
    bitvector::BitvectorBound,
    forward::{Bitwise, TypedEq},
};

use super::ThreeValuedBitvector;

impl<B: BitvectorBound> TypedEq for ThreeValuedBitvector<B> {
    type Output = Boolean;
    fn eq(self, rhs: Self) -> Boolean {
        // result can be false if at least one bit can be different
        // result can be true if all bits can be the same

        let can_be_different_bits =
            (self.zeros.bit_and(rhs.ones)).bit_or(self.ones.bit_and(rhs.zeros));
        let can_be_same_bits = (self.zeros.bit_and(rhs.zeros)).bit_or(self.ones.bit_and(rhs.ones));

        let can_be_different = can_be_different_bits.is_nonzero();
        let can_be_same = can_be_same_bits.is_full_mask();

        Boolean::from_bools(can_be_different, can_be_same)
    }

    fn ne(self, rhs: Self) -> Self::Output {
        self.eq(rhs).bit_not()
    }
}

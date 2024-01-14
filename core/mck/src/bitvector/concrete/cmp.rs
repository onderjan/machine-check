use std::cmp::Ordering;

use crate::forward::TypedCmp;

use super::ConcreteBitvector;

impl<const L: u32> TypedCmp for ConcreteBitvector<L> {
    type Output = ConcreteBitvector<1>;

    fn typed_slt(self, rhs: Self) -> Self::Output {
        let result = self.as_signed() < rhs.as_signed();
        ConcreteBitvector::<1>::new(result as u64)
    }

    fn typed_ult(self, rhs: Self) -> Self::Output {
        let result = self.as_unsigned() < rhs.as_unsigned();
        ConcreteBitvector::<1>::new(result as u64)
    }

    fn typed_slte(self, rhs: Self) -> Self::Output {
        let result = self.as_signed() <= rhs.as_signed();
        ConcreteBitvector::<1>::new(result as u64)
    }

    fn typed_ulte(self, rhs: Self) -> Self::Output {
        let result = self.as_unsigned() <= rhs.as_unsigned();
        ConcreteBitvector::<1>::new(result as u64)
    }
}

impl<const L: u32> ConcreteBitvector<L> {
    pub(crate) fn unsigned_cmp(&self, other: &Self) -> Ordering {
        self.as_unsigned().cmp(&other.as_unsigned())
    }
}

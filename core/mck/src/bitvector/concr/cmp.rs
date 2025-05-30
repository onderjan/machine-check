use std::cmp::Ordering;

use crate::{concr::Boolean, forward::TypedCmp};

use super::ConcreteBitvector;

impl<const L: u32> TypedCmp for ConcreteBitvector<L> {
    type Output = Boolean;

    fn slt(self, rhs: Self) -> Self::Output {
        let result = self.as_signed() < rhs.as_signed();
        Boolean::new(result as u64)
    }

    fn ult(self, rhs: Self) -> Self::Output {
        let result = self.as_unsigned() < rhs.as_unsigned();
        Boolean::new(result as u64)
    }

    fn sle(self, rhs: Self) -> Self::Output {
        let result = self.as_signed() <= rhs.as_signed();
        Boolean::new(result as u64)
    }

    fn ule(self, rhs: Self) -> Self::Output {
        let result = self.as_unsigned() <= rhs.as_unsigned();
        Boolean::new(result as u64)
    }
}

impl<const L: u32> ConcreteBitvector<L> {
    pub fn unsigned_cmp(&self, other: &Self) -> Ordering {
        self.as_unsigned().cmp(&other.as_unsigned())
    }
    pub fn signed_cmp(&self, other: &Self) -> Ordering {
        self.as_signed().cmp(&other.as_signed())
    }
}

use std::cmp::Ordering;

use crate::{concr::Boolean, forward::TypedCmp};

use super::ConcreteBitvector;

impl<const L: u32> TypedCmp for ConcreteBitvector<L> {
    type Output = Boolean;

    fn slt(self, rhs: Self) -> Self::Output {
        let result = self.to_i64() < rhs.to_i64();
        Boolean::new(result as u64)
    }

    fn ult(self, rhs: Self) -> Self::Output {
        let result = self.to_u64() < rhs.to_u64();
        Boolean::new(result as u64)
    }

    fn sle(self, rhs: Self) -> Self::Output {
        let result = self.to_i64() <= rhs.to_i64();
        Boolean::new(result as u64)
    }

    fn ule(self, rhs: Self) -> Self::Output {
        let result = self.to_u64() <= rhs.to_u64();
        Boolean::new(result as u64)
    }
}

impl<const L: u32> ConcreteBitvector<L> {
    pub fn unsigned_cmp(&self, other: &Self) -> Ordering {
        self.to_u64().cmp(&other.to_u64())
    }
    pub fn signed_cmp(&self, other: &Self) -> Ordering {
        self.to_i64().cmp(&other.to_i64())
    }
}

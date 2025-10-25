use std::cmp::Ordering;

use crate::{bitvector::bound::BitvectorBound, concr::Boolean, forward::TypedCmp};

use super::ConcreteBitvector;

impl<B: BitvectorBound> TypedCmp for ConcreteBitvector<B> {
    type Output = Boolean;

    fn slt(self, rhs: Self) -> Boolean {
        assert_eq!(self.bound, rhs.bound);
        let result = self.to_i64() < rhs.to_i64();
        Boolean::new(result)
    }

    fn ult(self, rhs: Self) -> Boolean {
        assert_eq!(self.bound, rhs.bound);
        let result = self.to_u64() < rhs.to_u64();
        Boolean::new(result)
    }

    fn sle(self, rhs: Self) -> Boolean {
        assert_eq!(self.bound, rhs.bound);
        let result = self.to_i64() <= rhs.to_i64();
        Boolean::new(result)
    }

    fn ule(self, rhs: Self) -> Boolean {
        assert_eq!(self.bound, rhs.bound);
        let result = self.to_u64() <= rhs.to_u64();
        Boolean::new(result)
    }
}

impl<B: BitvectorBound> ConcreteBitvector<B> {
    pub fn unsigned_cmp(&self, rhs: &Self) -> Ordering {
        assert_eq!(self.bound, rhs.bound);
        self.to_u64().cmp(&rhs.to_u64())
    }
    pub fn signed_cmp(&self, rhs: &Self) -> Ordering {
        assert_eq!(self.bound, rhs.bound);
        self.to_i64().cmp(&rhs.to_i64())
    }
}

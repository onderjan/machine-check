use std::cmp::Ordering;

use crate::{
    concr::{Boolean, RConcreteBitvector},
    forward::TypedCmp,
};

use super::ConcreteBitvector;

impl RConcreteBitvector {
    pub fn typed_slt(self, rhs: Self) -> Boolean {
        assert_eq!(self.width, rhs.width);
        let result = self.to_i64() < rhs.to_i64();
        Boolean::new(result as u64)
    }

    pub fn typed_ult(self, rhs: Self) -> Boolean {
        assert_eq!(self.width, rhs.width);
        let result = self.to_u64() < rhs.to_u64();
        Boolean::new(result as u64)
    }

    pub fn typed_sle(self, rhs: Self) -> Boolean {
        assert_eq!(self.width, rhs.width);
        let result = self.to_i64() <= rhs.to_i64();
        Boolean::new(result as u64)
    }

    pub fn typed_ule(self, rhs: Self) -> Boolean {
        assert_eq!(self.width, rhs.width);
        let result = self.to_u64() <= rhs.to_u64();
        Boolean::new(result as u64)
    }

    pub fn unsigned_cmp(&self, rhs: &Self) -> Ordering {
        assert_eq!(self.width, rhs.width);
        self.to_u64().cmp(&rhs.to_u64())
    }
    pub fn signed_cmp(&self, rhs: &Self) -> Ordering {
        assert_eq!(self.width, rhs.width);
        self.to_i64().cmp(&rhs.to_i64())
    }
}

impl<const L: u32> TypedCmp for ConcreteBitvector<L> {
    type Output = Boolean;

    fn slt(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.typed_slt(rhs)
    }

    fn ult(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.typed_ult(rhs)
    }

    fn sle(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.typed_sle(rhs)
    }

    fn ule(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.typed_ule(rhs)
    }
}

impl<const L: u32> ConcreteBitvector<L> {
    pub fn unsigned_cmp(&self, rhs: &Self) -> Ordering {
        self.to_u64().cmp(&rhs.to_u64())
    }
    pub fn signed_cmp(&self, rhs: &Self) -> Ordering {
        self.to_i64().cmp(&rhs.to_i64())
    }
}

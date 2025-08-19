use std::cmp::Ordering;

use crate::{
    concr::{Boolean, RConcreteBitvector},
    forward::TypedCmp,
};

use super::ConcreteBitvector;

impl TypedCmp for RConcreteBitvector {
    type Output = Boolean;

    fn slt(self, rhs: Self) -> Boolean {
        assert_eq!(self.width, rhs.width);
        let result = self.to_i64() < rhs.to_i64();
        Boolean::new(result as u64)
    }

    fn ult(self, rhs: Self) -> Boolean {
        assert_eq!(self.width, rhs.width);
        let result = self.to_u64() < rhs.to_u64();
        Boolean::new(result as u64)
    }

    fn sle(self, rhs: Self) -> Boolean {
        assert_eq!(self.width, rhs.width);
        let result = self.to_i64() <= rhs.to_i64();
        Boolean::new(result as u64)
    }

    fn ule(self, rhs: Self) -> Boolean {
        assert_eq!(self.width, rhs.width);
        let result = self.to_u64() <= rhs.to_u64();
        Boolean::new(result as u64)
    }
}

impl RConcreteBitvector {
    pub fn unsigned_cmp(&self, rhs: &Self) -> Ordering {
        assert_eq!(self.width, rhs.width);
        self.to_u64().cmp(&rhs.to_u64())
    }
    pub fn signed_cmp(&self, rhs: &Self) -> Ordering {
        assert_eq!(self.width, rhs.width);
        self.to_i64().cmp(&rhs.to_i64())
    }
}

impl<const W: u32> TypedCmp for ConcreteBitvector<W> {
    type Output = Boolean;

    fn slt(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.slt(rhs)
    }

    fn ult(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.ult(rhs)
    }

    fn sle(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.sle(rhs)
    }

    fn ule(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.ule(rhs)
    }
}

impl<const W: u32> ConcreteBitvector<W> {
    pub fn unsigned_cmp(&self, rhs: &Self) -> Ordering {
        self.to_u64().cmp(&rhs.to_u64())
    }
    pub fn signed_cmp(&self, rhs: &Self) -> Ordering {
        self.to_i64().cmp(&rhs.to_i64())
    }
}

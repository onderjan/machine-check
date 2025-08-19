use crate::{
    concr::{Boolean, RConcreteBitvector},
    forward::TypedEq,
};

use super::ConcreteBitvector;

impl TypedEq for RConcreteBitvector {
    type Output = Boolean;

    fn eq(self, rhs: Self) -> Boolean {
        assert_eq!(self.width, rhs.width);
        let result = self.value == rhs.value;
        Boolean::new(result as u64)
    }

    fn ne(self, rhs: Self) -> Boolean {
        assert_eq!(self.width, rhs.width);
        let result = self.value != rhs.value;
        Boolean::new(result as u64)
    }
}

impl<const W: u32> TypedEq for ConcreteBitvector<W> {
    type Output = Boolean;

    fn eq(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.eq(rhs)
    }

    fn ne(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.ne(rhs)
    }
}

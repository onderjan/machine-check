use crate::{concr::Boolean, forward::TypedEq};

use super::ConcreteBitvector;

impl<const L: u32> TypedEq for ConcreteBitvector<L> {
    type Output = Boolean;
    fn eq(self, rhs: Self) -> Self::Output {
        let result = self.0 == rhs.0;
        Boolean::new(result as u64)
    }

    fn ne(self, rhs: Self) -> Self::Output {
        let result = self.0 != rhs.0;
        Boolean::new(result as u64)
    }
}

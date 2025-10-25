use crate::{bitvector::bound::BitvectorBound, concr::Boolean, forward::TypedEq};

use super::ConcreteBitvector;

impl<B: BitvectorBound> TypedEq for ConcreteBitvector<B> {
    type Output = Boolean;

    fn eq(self, rhs: Self) -> Boolean {
        assert_eq!(self.bound, rhs.bound);
        let result = self.value == rhs.value;
        Boolean::new(result)
    }

    fn ne(self, rhs: Self) -> Boolean {
        assert_eq!(self.bound, rhs.bound);
        let result = self.value != rhs.value;
        Boolean::new(result)
    }
}

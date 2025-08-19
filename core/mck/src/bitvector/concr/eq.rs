use crate::{
    concr::{Boolean, RConcreteBitvector},
    forward::TypedEq,
};

use super::ConcreteBitvector;

impl RConcreteBitvector {
    pub fn typed_eq(self, rhs: Self) -> Boolean {
        assert_eq!(self.width, rhs.width);
        let result = self.value == rhs.value;
        Boolean::new(result as u64)
    }

    pub fn typed_ne(self, rhs: Self) -> Boolean {
        assert_eq!(self.width, rhs.width);
        let result = self.value != rhs.value;
        Boolean::new(result as u64)
    }
}

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

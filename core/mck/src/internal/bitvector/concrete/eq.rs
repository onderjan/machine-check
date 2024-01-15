use crate::forward::TypedEq;

use super::ConcreteBitvector;

impl<const L: u32> TypedEq for ConcreteBitvector<L> {
    type Output = ConcreteBitvector<1>;
    fn typed_eq(self, rhs: Self) -> Self::Output {
        let result = self.0 == rhs.0;
        ConcreteBitvector::<1>::new(result as u64)
    }
}

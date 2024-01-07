use crate::{bitvector::concrete::ConcreteBitvector, forward::TypedEq};

use super::Bitvector;

impl<const L: u32> TypedEq for Bitvector<L> {
    type Output = Bitvector<1>;
    fn typed_eq(self, rhs: Self) -> Self::Output {
        // result can be true (1) if the intervals intersect
        // result can be false (0) if the intervals do not have the same concrete value
        let can_be_true = self.intersects(&rhs);

        let can_be_false = if let (Some(lhs_value), Some(rhs_value)) =
            (self.concrete_value(), rhs.concrete_value())
        {
            lhs_value != rhs_value
        } else {
            true
        };

        let start = ConcreteBitvector::new(if can_be_false { 0 } else { 1 });
        let end = ConcreteBitvector::new(if can_be_true { 1 } else { 0 });
        assert!(start.as_unsigned() <= end.as_unsigned());
        Bitvector::<1>::from_wrap_interval(start, end)
    }
}

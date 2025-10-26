mod ops;
mod support;

use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::{
    abstr::{BitvectorDomain, Boolean, CBitvectorDomain, PanicResult, Test},
    concr::{CConcreteBitvector, ConcreteBitvector, SignedBitvector, UnsignedBitvector},
    misc::{BitvectorBound, CBound, Join, RBound},
};

#[derive(Clone, Copy, Hash, Serialize, Deserialize)]
pub struct CombinedBitvector<
    B: BitvectorBound,
    L: BitvectorDomain<Bound = B>,
    R: BitvectorDomain<Bound = B>,
> {
    left: L,
    right: R,
}

pub type CCombinedBitvector<const W: u32, L, R> = CombinedBitvector<CBound<W>, L, R>;
pub type RCombinedBitvector<L, R> = CombinedBitvector<RBound, L, R>;

#[allow(dead_code)]
impl<B: BitvectorBound, L: BitvectorDomain<Bound = B>, R: BitvectorDomain<Bound = B>>
    CombinedBitvector<B, L, R>
{
    pub fn new(value: u64, bound: B) -> Self {
        Self::single_value(value, bound)
    }

    pub(crate) fn combine(left: L, right: R) -> Self {
        todo!("Combine")
        // TODO combine
        /*
        // restrict the dual interval
        let near_min = three_valued.umin().max(dual_interval.umin());
        let near_max = three_valued.smax().min(dual_interval.smax());
        let far_min = three_valued.smin().max(dual_interval.smin());
        let far_max = three_valued.umax().min(dual_interval.umax());

        let near = WrappingInterval::new(near_min.cast_bitvector(), near_max.cast_bitvector());
        let far = WrappingInterval::new(far_min.cast_bitvector(), far_max.cast_bitvector());

        let dual_interval = DualInterval::from_wrapping_intervals(&[near, far]);

        // restrict the three-valued bit-vector
        let interval_bitvec = ThreeValuedBitvector::from_unsigned_interval(near_min, far_max);
        let Some(three_valued) = three_valued.meet(&interval_bitvec) else {
            panic!("Three-valued bit-vector combined with dual-interval should not be empty");
        };

        Self {
            three_valued,
            dual_interval,
        }
        */
    }

    fn combine_boolean(three_valued: Boolean, dual_interval: Boolean) -> Boolean {
        // meet the values
        let can_be_false = three_valued.can_be_false() && dual_interval.can_be_false();
        let can_be_true = three_valued.can_be_true() && dual_interval.can_be_true();

        Boolean::from_bools(can_be_false, can_be_true)
    }

    fn combine_panic_result(
        three_valued: PanicResult<L>,
        dual_interval: PanicResult<R>,
    ) -> PanicResult<CombinedBitvector<B, L, R>> {
        let panic = three_valued
            .panic
            .meet(&dual_interval.panic)
            .expect("Combined panic meet should not be empty");
        let result = Self::combine(three_valued.result, dual_interval.result);
        PanicResult { panic, result }
    }

    pub(crate) fn from_left(left: L) -> CombinedBitvector<B, L, R> {
        let dual_interval = R::top(left.bound());
        Self::combine(left, dual_interval)
    }

    pub(crate) fn left(&self) -> &L {
        &self.left
    }

    pub(crate) fn right(&self) -> &R {
        &self.right
    }
}

impl<B: BitvectorBound, L: BitvectorDomain<Bound = B>, R: BitvectorDomain<Bound = B>> Join
    for CombinedBitvector<B, L, R>
{
    fn join(self, other: &Self) -> Self {
        let three_valued = self.left.join(&other.left);
        let dual_interval = self.right.join(&other.right);
        Self::combine(three_valued, dual_interval)
    }
}

impl<B: BitvectorBound, L: BitvectorDomain<Bound = B>, R: BitvectorDomain<Bound = B>>
    BitvectorDomain for CombinedBitvector<B, L, R>
{
    type Bound = B;
    type General<X: BitvectorBound> = CombinedBitvector<X, L::General<X>, R::General<X>>;

    fn single_value(value: u64, bound: Self::Bound) -> Self {
        let left = L::single_value(value, bound);
        let right = R::single_value(value, bound);
        Self { left, right }
    }

    fn top(bound: Self::Bound) -> Self {
        Self {
            left: L::top(bound),
            right: R::top(bound),
        }
    }

    fn bound(&self) -> Self::Bound {
        // both bounds must be the same
        self.left.bound()
    }

    fn meet(self, other: &Self) -> Option<Self> {
        let left = self.left.meet(&other.left);
        let right = self.right.meet(&other.right);
        if let (Some(left), Some(right)) = (left, right) {
            Some(Self::combine(left, right))
        } else {
            None
        }
    }

    fn umin(&self) -> crate::concr::UnsignedBitvector<Self::Bound> {
        // take maximum of both minimums as they meet each other
        UnsignedBitvector::max(self.left.umin(), self.right.umin())
    }

    fn umax(&self) -> crate::concr::UnsignedBitvector<Self::Bound> {
        // take the minimum of both maximums as they meet each other
        UnsignedBitvector::min(self.left.umax(), self.right.umax())
    }

    fn smin(&self) -> crate::concr::SignedBitvector<Self::Bound> {
        // take maximum of both minimums as they meet each other
        SignedBitvector::max(self.left.smin(), self.right.smin())
    }

    fn smax(&self) -> crate::concr::SignedBitvector<Self::Bound> {
        // take the minimum of both maximums as they meet each other
        SignedBitvector::min(self.left.smax(), self.right.smax())
    }

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>> {
        match (self.left.concrete_value(), self.right.concrete_value()) {
            (None, None) => None,
            (None, Some(value)) => Some(value),
            (Some(value), None) => Some(value),
            (Some(left), Some(right)) => {
                assert_eq!(left, right);
                Some(left)
            }
        }
    }
}

impl<
        const W: u32,
        L: CBitvectorDomain<Bound = CBound<W>, Concrete = CConcreteBitvector<W>>,
        R: CBitvectorDomain<Bound = CBound<W>, Concrete = CConcreteBitvector<W>>,
    > CBitvectorDomain for CCombinedBitvector<W, L, R>
{
    type Concrete = CConcreteBitvector<W>;

    fn from_concrete_bitvector(value: Self::Concrete) -> Self {
        let left = L::from_concrete_bitvector(value);
        let right = R::from_concrete_bitvector(value);

        Self::combine(left, right)
    }

    fn from_runtime_bitvector(value: Self::General<RBound>) -> Self {
        Self {
            left: L::from_runtime_bitvector(value.left),
            right: R::from_runtime_bitvector(value.right),
        }
    }

    fn as_runtime_bitvector(&self) -> Self::General<RBound> {
        Self::General {
            left: self.left.as_runtime_bitvector(),
            right: self.right.as_runtime_bitvector(),
        }
    }
}

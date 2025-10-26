mod ops;
mod support;

use std::{hash::Hash, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::{
    abstr::{
        dual_interval::DualInterval, three_valued::ThreeValuedBitvector, BitvectorDomain, Boolean,
        CBitvectorDomain, PanicResult, Test,
    },
    bitvector::interval::WrappingInterval,
    concr::{CConcreteBitvector, ConcreteBitvector, SignedBitvector, UnsignedBitvector},
    misc::{BitvectorBound, CBound, Join, RBound},
};

pub trait DomainCombination<B: BitvectorBound>: Clone + Copy + Hash {
    type Left: BitvectorDomain<Bound = B>;
    type Right: BitvectorDomain<Bound = B>;
    type General<X: BitvectorBound>: DomainCombination<
        X,
        Left = <Self::Left as BitvectorDomain>::General<X>,
        Right = <Self::Right as BitvectorDomain>::General<X>,
    >;

    fn combine(left: &mut Self::Left, right: &mut Self::Right);
}

#[derive(Clone, Copy, Hash, Debug)]
pub struct TVDICombination<B: BitvectorBound>(PhantomData<B>);

impl<B: BitvectorBound> DomainCombination<B> for TVDICombination<B> {
    type Left = ThreeValuedBitvector<B>;
    type Right = DualInterval<B>;

    type General<X: BitvectorBound> = TVDICombination<X>;

    fn combine(three_valued: &mut Self::Left, dual_interval: &mut Self::Right) {
        // restrict the dual interval
        let near_min = three_valued.umin().max(dual_interval.umin());
        let near_max = three_valued.smax().min(dual_interval.smax());
        let far_min = three_valued.smin().max(dual_interval.smin());
        let far_max = three_valued.umax().min(dual_interval.umax());

        let near = WrappingInterval::new(near_min.cast_bitvector(), near_max.cast_bitvector());
        let far = WrappingInterval::new(far_min.cast_bitvector(), far_max.cast_bitvector());

        *dual_interval = DualInterval::from_wrapping_intervals(&[near, far]);

        // restrict the three-valued bit-vector
        let interval_bitvec = ThreeValuedBitvector::from_unsigned_interval(near_min, far_max);
        let Some(three_valued_result) = three_valued.meet(&interval_bitvec) else {
            panic!("Three-valued bit-vector combined with dual-interval should not be empty");
        };

        *three_valued = three_valued_result;
    }
}

#[derive(Clone, Copy, Hash, Serialize, Deserialize)]
pub struct CombinedBitvector<B: BitvectorBound, D: DomainCombination<B>> {
    left: D::Left,
    right: D::Right,
}

pub type CCombinedBitvector<const W: u32, D> = CombinedBitvector<CBound<W>, D>;
pub type RCombinedBitvector<D> = CombinedBitvector<RBound, D>;

#[allow(dead_code)]
impl<B: BitvectorBound, D: DomainCombination<B>> CombinedBitvector<B, D> {
    pub fn new(value: u64, bound: B) -> Self {
        Self::single_value(value, bound)
    }

    pub(crate) fn combine(mut left: D::Left, mut right: D::Right) -> Self {
        D::combine(&mut left, &mut right);
        Self { left, right }
    }

    fn combine_boolean(three_valued: Boolean, dual_interval: Boolean) -> Boolean {
        // meet the values
        let can_be_false = three_valued.can_be_false() && dual_interval.can_be_false();
        let can_be_true = three_valued.can_be_true() && dual_interval.can_be_true();

        Boolean::from_bools(can_be_false, can_be_true)
    }

    fn combine_panic_result(
        three_valued: PanicResult<D::Left>,
        dual_interval: PanicResult<D::Right>,
    ) -> PanicResult<CombinedBitvector<B, D>> {
        let panic = three_valued
            .panic
            .meet(&dual_interval.panic)
            .expect("Combined panic meet should not be empty");
        let result = Self::combine(three_valued.result, dual_interval.result);
        PanicResult { panic, result }
    }

    pub(crate) fn from_left(left: D::Left) -> CombinedBitvector<B, D> {
        let dual_interval = D::Right::top(left.bound());
        Self::combine(left, dual_interval)
    }

    pub(crate) fn left(&self) -> &D::Left {
        &self.left
    }

    pub(crate) fn right(&self) -> &D::Right {
        &self.right
    }
}

impl<B: BitvectorBound, D: DomainCombination<B>> Join for CombinedBitvector<B, D> {
    fn join(self, other: &Self) -> Self {
        let three_valued = self.left.join(&other.left);
        let dual_interval = self.right.join(&other.right);
        Self::combine(three_valued, dual_interval)
    }
}

impl<B: BitvectorBound, D: DomainCombination<B>> BitvectorDomain for CombinedBitvector<B, D> {
    type Bound = B;
    type General<X: BitvectorBound> = CombinedBitvector<X, D::General<X>>;

    fn single_value(value: u64, bound: Self::Bound) -> Self {
        let left = D::Left::single_value(value, bound);
        let right = D::Right::single_value(value, bound);
        Self { left, right }
    }

    fn top(bound: Self::Bound) -> Self {
        Self {
            left: D::Left::top(bound),
            right: D::Right::top(bound),
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

impl<const W: u32, D: DomainCombination<CBound<W>>> CBitvectorDomain for CCombinedBitvector<W, D>
where
    D::Left: CBitvectorDomain<Concrete = CConcreteBitvector<W>>,
    D::Right: CBitvectorDomain<Concrete = CConcreteBitvector<W>>,
{
    type Concrete = CConcreteBitvector<W>;

    fn from_concrete_bitvector(value: Self::Concrete) -> Self {
        let left = D::Left::from_concrete_bitvector(value);
        let right = D::Right::from_concrete_bitvector(value);

        Self::combine(left, right)
    }

    fn from_runtime_bitvector(value: Self::General<RBound>) -> Self {
        Self {
            left: D::Left::from_runtime_bitvector(value.left),
            right: D::Right::from_runtime_bitvector(value.right),
        }
    }

    fn as_runtime_bitvector(&self) -> Self::General<RBound> {
        Self::General {
            left: self.left.as_runtime_bitvector(),
            right: self.right.as_runtime_bitvector(),
        }
    }
}

mod ops;
mod support;

use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::{
    abstr::{
        three_valued::CThreeValuedBitvector, BitvectorDomain, Boolean, CBitvectorDomain,
        PanicResult, Test,
    },
    bitvector::{abstr::dual_interval::CDualInterval, interval::WrappingInterval},
    concr::{CConcreteBitvector, ConcreteBitvector, SignedBitvector, UnsignedBitvector},
    misc::{BitvectorBound, CBound, Join, RBound},
};

use super::dual_interval::DualInterval;
use super::three_valued::ThreeValuedBitvector;

#[derive(Clone, Copy, Hash, Serialize, Deserialize)]
pub struct CombinedBitvector<B: BitvectorBound> {
    three_valued: ThreeValuedBitvector<B>,
    dual_interval: DualInterval<B>,
}

pub type CCombinedBitvector<const W: u32> = CombinedBitvector<CBound<W>>;
pub type RCombinedBitvector = CombinedBitvector<RBound>;

#[allow(dead_code)]
impl<B: BitvectorBound> CombinedBitvector<B> {
    pub fn new(value: u64, bound: B) -> Self {
        let three_valued = ThreeValuedBitvector::new(value, bound);
        let dual_interval = DualInterval::from_value(ConcreteBitvector::new(value, bound));
        Self {
            three_valued,
            dual_interval,
        }
    }

    pub(crate) fn combine(
        three_valued: ThreeValuedBitvector<B>,
        dual_interval: DualInterval<B>,
    ) -> Self {
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
    }

    fn combine_boolean(three_valued: Boolean, dual_interval: Boolean) -> Boolean {
        // meet the values
        let can_be_false = three_valued.can_be_false() && dual_interval.can_be_false();
        let can_be_true = three_valued.can_be_true() && dual_interval.can_be_true();

        Boolean::from_bools(can_be_false, can_be_true)
    }

    fn combine_panic_result(
        three_valued: PanicResult<ThreeValuedBitvector<B>>,
        dual_interval: PanicResult<DualInterval<B>>,
    ) -> PanicResult<CombinedBitvector<B>> {
        let panic = three_valued
            .panic
            .meet(&dual_interval.panic)
            .expect("Combined panic meet should not be empty");
        let result = Self::combine(three_valued.result, dual_interval.result);
        PanicResult { panic, result }
    }

    #[must_use]
    pub fn from_zeros_ones(zeros: ConcreteBitvector<B>, ones: ConcreteBitvector<B>) -> Self {
        Self::from_three_valued(ThreeValuedBitvector::from_zeros_ones(zeros, ones))
    }

    pub fn from_three_valued(three_valued: ThreeValuedBitvector<B>) -> CombinedBitvector<B> {
        let dual_interval = DualInterval::new_full(three_valued.bound());
        Self::combine(three_valued, dual_interval)
    }

    pub(crate) fn three_valued(&self) -> &ThreeValuedBitvector<B> {
        &self.three_valued
    }

    pub(crate) fn dual_interval(&self) -> &DualInterval<B> {
        &self.dual_interval
    }
}

impl<B: BitvectorBound> Join for CombinedBitvector<B> {
    fn join(self, other: &Self) -> Self {
        let three_valued = self.three_valued.join(&other.three_valued);
        let dual_interval = self.dual_interval.join(&other.dual_interval);
        Self::combine(three_valued, dual_interval)
    }
}

impl<B: BitvectorBound> BitvectorDomain for CombinedBitvector<B> {
    type Bound = B;

    fn bound(&self) -> Self::Bound {
        // both bounds must be the same
        self.three_valued.bound()
    }

    fn meet(self, rhs: &Self) -> Option<Self> {
        let three_valued = self.three_valued.meet(&rhs.three_valued);
        let dual_interval = self.dual_interval.meet(&rhs.dual_interval);
        if let (Some(three_valued), Some(dual_interval)) = (three_valued, dual_interval) {
            Some(Self::combine(three_valued, dual_interval))
        } else {
            None
        }
    }

    fn umin(&self) -> crate::concr::UnsignedBitvector<Self::Bound> {
        // take maximum of both minimums as they meet each other
        UnsignedBitvector::max(self.three_valued.umin(), self.dual_interval.umin())
    }

    fn umax(&self) -> crate::concr::UnsignedBitvector<Self::Bound> {
        // take the minimum of both maximums as they meet each other
        UnsignedBitvector::min(self.three_valued.umax(), self.dual_interval.umax())
    }

    fn smin(&self) -> crate::concr::SignedBitvector<Self::Bound> {
        // take maximum of both minimums as they meet each other
        SignedBitvector::max(self.three_valued.smin(), self.dual_interval.smin())
    }

    fn smax(&self) -> crate::concr::SignedBitvector<Self::Bound> {
        // take the minimum of both maximums as they meet each other
        SignedBitvector::min(self.three_valued.smax(), self.dual_interval.smax())
    }

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>> {
        match (
            self.three_valued.concrete_value(),
            self.dual_interval.concrete_value(),
        ) {
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

impl<const W: u32> CBitvectorDomain for CCombinedBitvector<W> {
    type Concrete = CConcreteBitvector<W>;
    type Runtime = RCombinedBitvector;

    fn from_concrete_bitvector(value: Self::Concrete) -> Self {
        let three_valued = CThreeValuedBitvector::from_concrete_bitvector(value);
        let dual_interval = CDualInterval::from_concrete_bitvector(value);

        Self::combine(three_valued, dual_interval)
    }

    fn from_runtime_bitvector(value: Self::Runtime) -> Self {
        Self {
            three_valued: CThreeValuedBitvector::from_runtime_bitvector(value.three_valued),
            dual_interval: CDualInterval::from_runtime_bitvector(value.dual_interval),
        }
    }

    fn as_runtime_bitvector(&self) -> Self::Runtime {
        Self::Runtime {
            three_valued: self.three_valued.as_runtime_bitvector(),
            dual_interval: self.dual_interval.as_runtime_bitvector(),
        }
    }
}

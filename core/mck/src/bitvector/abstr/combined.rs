mod ops;
mod support;

use std::hash::Hash;

use crate::{
    abstr::{ArrayFieldBitvector, BitvectorDomain, Boolean, ManipField, PanicResult, Phi, Test},
    concr::{ConcreteBitvector, UnsignedInterval, WrappingInterval},
};

use super::{
    dual_interval::{self, DualInterval},
    three_valued::ThreeValuedBitvector,
};

#[derive(Clone, Copy, Hash, Default)]
pub struct CombinedBitvector<const W: u32> {
    three_valued: ThreeValuedBitvector<W>,
    dual_interval: DualInterval<W>,
}

impl<const W: u32> CombinedBitvector<W> {
    pub fn new(value: u64) -> Self {
        let three_valued = ThreeValuedBitvector::new(value);
        let dual_interval = DualInterval::from_value(ConcreteBitvector::new(value));
        Self {
            three_valued,
            dual_interval,
        }
    }

    pub fn combine(three_valued: ThreeValuedBitvector<W>, dual_interval: DualInterval<W>) -> Self {
        // restrict the dual interval
        let near_min = three_valued.umin().max(dual_interval.unsigned_min());
        let near_max = three_valued.smax().min(dual_interval.signed_max());
        let far_min = three_valued.smin().max(dual_interval.signed_min());
        let far_max = three_valued.umax().min(dual_interval.unsigned_max());

        let near = WrappingInterval::new(near_min.as_bitvector(), near_max.as_bitvector());
        let far = WrappingInterval::new(far_min.as_bitvector(), far_max.as_bitvector());

        let dual_interval = DualInterval::from_wrapping_intervals(&[near, far]);

        // restrict the three-valued bit-vector
        let interval_bitvec =
            ThreeValuedBitvector::from_interval(near_min.as_bitvector(), far_max.as_bitvector());
        let three_valued = three_valued.intersection(&interval_bitvec);

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
        three_valued: PanicResult<ThreeValuedBitvector<W>>,
        dual_interval: PanicResult<DualInterval<W>>,
    ) -> PanicResult<CombinedBitvector<W>> {
        let panic = three_valued
            .panic
            .meet(dual_interval.panic)
            .expect("Combined panic meet should not be empty");
        let result = Self::combine(three_valued.result, dual_interval.result);
        PanicResult { panic, result }
    }

    #[must_use]
    pub fn from_zeros_ones(zeros: ConcreteBitvector<W>, ones: ConcreteBitvector<W>) -> Self {
        Self::from_three_valued(ThreeValuedBitvector::from_zeros_ones(zeros, ones))
    }

    pub fn concrete_value(&self) -> Option<ConcreteBitvector<W>> {
        self.three_valued.concrete_value()
    }

    pub fn from_three_valued(three_valued: ThreeValuedBitvector<W>) -> CombinedBitvector<W> {
        let dual_interval = DualInterval::FULL;
        Self::combine(three_valued, dual_interval)
    }

    pub(crate) fn three_valued(&self) -> &ThreeValuedBitvector<W> {
        &self.three_valued
    }

    pub(crate) fn dual_interval(&self) -> &DualInterval<W> {
        &self.dual_interval
    }
}

impl<const W: u32> Phi for CombinedBitvector<W> {
    fn phi(self, other: Self) -> Self {
        let three_valued = self.three_valued.phi(other.three_valued);
        let dual_interval = self.dual_interval.phi(other.dual_interval);
        Self::combine(three_valued, dual_interval)
    }

    fn uninit() -> Self {
        Self {
            three_valued: ThreeValuedBitvector::uninit(),
            dual_interval: DualInterval::uninit(),
        }
    }
}

impl<const W: u32> BitvectorDomain<W> for CombinedBitvector<W> {
    fn unsigned_interval(&self) -> UnsignedInterval<W> {
        self.dual_interval.to_unsigned_interval()
    }

    fn element_description(&self) -> ArrayFieldBitvector {
        // TODO show dual-interval values
        self.three_valued.element_description()
    }

    fn join(self, other: Self) -> Self {
        self.phi(other)
    }

    fn meet(self, other: Self) -> Option<Self> {
        let Some(three_valued) = self.three_valued.meet(other.three_valued) else {
            return None;
        };
        let Some(dual_interval) = self.dual_interval.meet(other.dual_interval) else {
            return None;
        };
        Some(Self::combine(three_valued, dual_interval))
    }
}

impl<const W: u32> ManipField for CombinedBitvector<W> {
    fn index(&self, _index: u64) -> Option<&dyn ManipField> {
        None
    }

    fn num_bits(&self) -> Option<u32> {
        Some(W)
    }

    fn min_unsigned(&self) -> Option<u64> {
        Some(self.dual_interval.unsigned_min().to_u64())
    }

    fn max_unsigned(&self) -> Option<u64> {
        Some(self.dual_interval.unsigned_max().to_u64())
    }

    fn min_signed(&self) -> Option<i64> {
        Some(self.dual_interval.signed_min().to_i64())
    }

    fn max_signed(&self) -> Option<i64> {
        Some(self.dual_interval.signed_max().to_i64())
    }

    fn description(&self) -> crate::abstr::Field {
        // TODO show dual-interval values
        self.three_valued.description()
    }
}

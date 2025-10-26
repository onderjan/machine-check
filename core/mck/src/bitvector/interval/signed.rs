use std::fmt::Debug;

use crate::{
    bitvector::{interval::SignlessInterval, BitvectorBound},
    concr::{ConcreteBitvector, SignedBitvector},
};

/// A signed interval with a minimum and a maximum value.
///
/// It is required that min <= max, which means the interval
/// does not support wrapping nor representing an empty set.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SignedInterval<B: BitvectorBound> {
    min: SignedBitvector<B>,
    max: SignedBitvector<B>,
}

impl<B: BitvectorBound> SignedInterval<B> {
    pub fn new(min: SignedBitvector<B>, max: SignedBitvector<B>) -> Self {
        // comparison will panic on different bound values
        assert!(min <= max);
        Self { min, max }
    }

    pub fn min(&self) -> SignedBitvector<B> {
        self.min
    }

    pub fn max(&self) -> SignedBitvector<B> {
        self.max
    }

    pub fn bound(&self) -> B {
        // the bound must be the same for min and max
        self.min.bound()
    }

    pub fn from_value(value: SignedBitvector<B>) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    pub fn try_into_signless(self) -> Option<SignlessInterval<B>> {
        if self.min.cast_bitvector().is_sign_bit_set()
            == self.max.cast_bitvector().is_sign_bit_set()
        {
            Some(SignlessInterval::new(
                self.min.cast_bitvector(),
                self.max.cast_bitvector(),
            ))
        } else {
            None
        }
    }

    pub fn ext<X: BitvectorBound>(self, new_bound: X) -> SignedInterval<X> {
        if self.min == self.max {
            // clearly, we can extend
            let ext_value = self.min.ext(new_bound);
            return SignedInterval {
                min: ext_value,
                max: ext_value,
            };
        }

        // if we narrow the interval and disregarded a bound, saturate
        let mut ext_min: SignedBitvector<X> = self.min.ext(new_bound);
        let mut ext_max: SignedBitvector<X> = self.max.ext(new_bound);

        let min_diff = self.min - ext_min.ext(self.min.bound());
        let max_diff = self.max - ext_max.ext(self.max.bound());

        if min_diff != max_diff {
            // we disregarded a bound, saturate
            ext_min = ConcreteBitvector::new_overhalf(new_bound).as_signed();
            ext_max = ConcreteBitvector::new_underhalf(new_bound).as_signed();
        }
        SignedInterval {
            min: ext_min,
            max: ext_max,
        }
    }

    #[allow(dead_code)]
    pub fn contains_value(&self, value: SignedBitvector<B>) -> bool {
        self.min <= value && value <= self.max
    }
}

impl<B: BitvectorBound> Debug for SignedInterval<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.min, self.max)
    }
}

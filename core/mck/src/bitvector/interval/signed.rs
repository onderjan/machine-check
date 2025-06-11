use std::fmt::Debug;

use crate::{
    bitvector::interval::SignlessInterval,
    concr::{ConcreteBitvector, SignedBitvector},
};

/// A signed interval with a minimum and a maximum value.
///
/// It is required that min <= max, which means the interval
/// does not support wrapping nor representing an empty set.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SignedInterval<const W: u32> {
    pub(super) min: SignedBitvector<W>,
    pub(super) max: SignedBitvector<W>,
}

impl<const W: u32> SignedInterval<W> {
    pub fn new(min: SignedBitvector<W>, max: SignedBitvector<W>) -> Self {
        assert!(min <= max);
        Self { min, max }
    }

    pub fn from_value(value: SignedBitvector<W>) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    pub fn contains_value(&self, value: SignedBitvector<W>) -> bool {
        self.min <= value && value <= self.max
    }

    pub fn min(&self) -> SignedBitvector<W> {
        self.min
    }
    pub fn max(&self) -> SignedBitvector<W> {
        self.max
    }

    pub fn try_into_signless(self) -> Option<SignlessInterval<W>> {
        if self.min.as_bitvector().is_sign_bit_set() == self.max.as_bitvector().is_sign_bit_set() {
            Some(SignlessInterval {
                min: self.min.as_bitvector(),
                max: self.max.as_bitvector(),
            })
        } else {
            None
        }
    }

    pub fn ext<const X: u32>(self) -> SignedInterval<X> {
        if self.min == self.max {
            // clearly, we can extend
            let ext_value = self.min.ext();
            return SignedInterval {
                min: ext_value,
                max: ext_value,
            };
        }

        // if we narrow the interval and disregarded a bound, saturate
        let mut ext_min: SignedBitvector<X> = self.min.ext();
        let mut ext_max: SignedBitvector<X> = self.max.ext();

        let min_diff = self.min - ext_min.ext();
        let max_diff = self.max - ext_max.ext();

        if min_diff != max_diff {
            // we disregarded a bound, saturate
            ext_min = ConcreteBitvector::const_overhalf().cast_signed();
            ext_max = ConcreteBitvector::const_underhalf().cast_signed();
        }
        SignedInterval {
            min: ext_min,
            max: ext_max,
        }
    }
}

impl<const W: u32> Debug for SignedInterval<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.min, self.max)
    }
}

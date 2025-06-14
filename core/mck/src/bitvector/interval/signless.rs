use std::fmt::{Debug, Display};

use crate::{
    bitvector::interval::{SignedInterval, UnsignedInterval, WrappingInterval},
    concr::ConcreteBitvector,
};

/// A signless interval with a minimum and a maximum value.
///
/// It is required that the signless interval has the minimum
/// and maximum value in the same half-plane.
/// It is required that min <= max, which means the interval
/// does not support wrapping nor representing an empty set.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SignlessInterval<const W: u32> {
    pub(super) min: ConcreteBitvector<W>,
    pub(super) max: ConcreteBitvector<W>,
}

impl<const W: u32> SignlessInterval<W> {
    pub fn new(min: ConcreteBitvector<W>, max: ConcreteBitvector<W>) -> Self {
        assert_eq!(min.is_sign_bit_set(), max.is_sign_bit_set());
        assert!(min.to_u64() <= max.to_u64());
        Self { min, max }
    }

    pub fn from_value(value: ConcreteBitvector<W>) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    pub fn is_sign_bit_set(&self) -> bool {
        // both min and max must have the same value of sign bit
        self.min.is_sign_bit_set()
    }

    pub const FULL_NEAR_HALFPLANE: Self = SignlessInterval {
        min: ConcreteBitvector::<W>::zero(),
        max: ConcreteBitvector::<W>::const_underhalf(),
    };

    pub const FULL_FAR_HALFPLANE: Self = SignlessInterval {
        min: ConcreteBitvector::<W>::const_overhalf(),
        max: ConcreteBitvector::<W>::const_umax(),
    };

    pub fn contains_value(&self, value: &ConcreteBitvector<W>) -> bool {
        // we can use either interpretation
        let value = value.cast_unsigned();
        self.min.cast_unsigned() <= value && value <= self.max.cast_unsigned()
    }

    pub fn contains(&self, other: &Self) -> bool {
        if self.min.is_sign_bit_set() != other.min.is_sign_bit_set() {
            return false;
        }
        self.min.cast_unsigned() <= other.min.cast_unsigned()
            && other.max.cast_unsigned() <= self.max.cast_unsigned()
    }

    pub fn concrete_value(&self) -> Option<ConcreteBitvector<W>> {
        if self.min == self.max {
            return Some(self.min);
        }
        None
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        assert_eq!(self.min.is_sign_bit_set(), other.min.is_sign_bit_set());
        let min = self.min.cast_unsigned().max(other.min.cast_unsigned());
        let max = self.max.cast_unsigned().min(other.max.cast_unsigned());
        if min <= max {
            Some(Self {
                min: min.as_bitvector(),
                max: max.as_bitvector(),
            })
        } else {
            None
        }
    }

    pub fn union(self, other: Self) -> Self {
        assert_eq!(self.min.is_sign_bit_set(), other.min.is_sign_bit_set());
        Self {
            min: self
                .min
                .cast_unsigned()
                .min(other.min.cast_unsigned())
                .as_bitvector(),
            max: self
                .max
                .cast_unsigned()
                .max(other.max.cast_unsigned())
                .as_bitvector(),
        }
    }

    pub fn union_opt(a: Option<Self>, b: Option<Self>) -> Option<Self> {
        match (a, b) {
            (None, None) => None,
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (Some(a), Some(b)) => Some(a.union(b)),
        }
    }

    pub fn min(&self) -> ConcreteBitvector<W> {
        self.min
    }
    pub fn max(&self) -> ConcreteBitvector<W> {
        self.max
    }

    pub fn into_wrapping(self) -> WrappingInterval<W> {
        WrappingInterval {
            start: self.min,
            end: self.max,
        }
    }

    pub fn into_unsigned(self) -> UnsignedInterval<W> {
        UnsignedInterval {
            min: self.min.cast_unsigned(),
            max: self.max.cast_unsigned(),
        }
    }

    pub fn into_signed(self) -> SignedInterval<W> {
        SignedInterval {
            min: self.min.cast_signed(),
            max: self.max.cast_signed(),
        }
    }

    pub fn all_with_length_iter(far: bool) -> impl Iterator<Item = Self> {
        let min_iter = ConcreteBitvector::<W>::all_with_length_iter();
        min_iter
            .flat_map(move |min| {
                if min.is_sign_bit_set() != far {
                    return None;
                }

                let max_iter = ConcreteBitvector::<W>::all_with_length_iter();

                let result = max_iter.flat_map(move |max| {
                    if max.is_sign_bit_set() != far {
                        return None;
                    }
                    if min.to_u64() > max.to_u64() {
                        return None;
                    }

                    Some(SignlessInterval::new(min, max))
                });
                Some(result)
            })
            .flatten()
    }
}

impl<const W: u32> Debug for SignlessInterval<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.min, self.max)
    }
}

impl<const W: u32> Display for SignlessInterval<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

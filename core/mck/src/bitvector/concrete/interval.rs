use crate::forward::HwArith;

use super::{ConcreteBitvector, SignedBitvector, UnsignedBitvector};

/// An unsigned interval with a minimum and a maximum value.
///
/// It is required that min <= max, which means the interval
/// does not support wrapping nor representing an empty set.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct UnsignedInterval<const W: u32> {
    min: UnsignedBitvector<W>,
    max: UnsignedBitvector<W>,
}

impl<const W: u32> UnsignedInterval<W> {
    pub const FULL: Self = Self {
        min: ConcreteBitvector::<W>::ZERO.cast_unsigned(),
        max: ConcreteBitvector::<W>::UMAX.cast_unsigned(),
    };

    fn contains_value(self, value: UnsignedBitvector<W>) -> bool {
        self.min <= value && value <= self.max
    }

    fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    fn union_opt(a: Option<Self>, b: Option<Self>) -> Option<Self> {
        match (a, b) {
            (None, None) => None,
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (Some(a), Some(b)) => Some(a.union(b)),
        }
    }

    pub fn min(&self) -> UnsignedBitvector<W> {
        self.min
    }
    pub fn max(&self) -> UnsignedBitvector<W> {
        self.max
    }
}

/// A signed interval with a minimum and a maximum value.
///
/// It is required that min <= max, which means the interval
/// does not support wrapping nor representing an empty set.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SignedInterval<const W: u32> {
    min: SignedBitvector<W>,
    max: SignedBitvector<W>,
}

impl<const W: u32> SignedInterval<W> {
    fn contains_value(self, value: SignedBitvector<W>) -> bool {
        self.min <= value && value <= self.max
    }

    fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    fn union_opt(a: Option<Self>, b: Option<Self>) -> Option<Self> {
        match (a, b) {
            (None, None) => None,
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (Some(a), Some(b)) => Some(a.union(b)),
        }
    }

    pub fn min(&self) -> SignedBitvector<W> {
        self.min
    }
    pub fn max(&self) -> SignedBitvector<W> {
        self.max
    }
}

/// A signless interval with a minimum and a maximum value.
///
/// It is required that the signless interval has the minimum
/// and maximum value in the same half-plane.
/// It is required that min <= max, which means the interval
/// does not support wrapping nor representing an empty set.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SignlessInterval<const W: u32> {
    min: ConcreteBitvector<W>,
    max: ConcreteBitvector<W>,
}

impl<const W: u32> SignlessInterval<W> {
    pub fn new(min: ConcreteBitvector<W>, max: ConcreteBitvector<W>) -> Self {
        assert_eq!(min.is_sign_bit_set(), max.is_sign_bit_set());
        assert!(min.as_unsigned() <= max.as_unsigned());
        Self { min, max }
    }

    pub fn from_value(value: ConcreteBitvector<W>) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    pub const FULL_NEAR_HALFPLANE: Self = SignlessInterval {
        min: ConcreteBitvector::<W>::ZERO,
        max: ConcreteBitvector::<W>::UNDERHALF,
    };

    pub const FULL_FAR_HALFPLANE: Self = SignlessInterval {
        min: ConcreteBitvector::<W>::OVERHALF,
        max: ConcreteBitvector::<W>::UMAX,
    };

    pub fn contains_value(self, value: ConcreteBitvector<W>) -> bool {
        // we can use either interpretation
        let value = value.cast_unsigned();
        self.min.cast_unsigned() <= value && value <= self.max.cast_unsigned()
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
}

pub enum WrappingInterpretation<const W: u32> {
    Signless(SignlessInterval<W>),
    Signed(SignedInterval<W>),
    Unsigned(UnsignedInterval<W>),
}

/// A wrapping interval.
///
/// If start <= end (unsigned), the interval represents [start,end].
/// If start > end, the interval represents the union of [T_MIN, end] and [start, T_MAX].
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct WrappingInterval<const W: u32> {
    start: ConcreteBitvector<W>,
    end: ConcreteBitvector<W>,
}

impl<const W: u32> WrappingInterval<W> {
    fn from_value(value: ConcreteBitvector<W>) -> Self {
        Self {
            start: value,
            end: value,
        }
    }

    // the canonical full interval is from zero to umax
    const FULL: Self = Self {
        start: ConcreteBitvector::<W>::ZERO,
        end: ConcreteBitvector::<W>::UMAX,
    };

    fn contains_value(self, value: ConcreteBitvector<W>) -> bool {
        // interpreted as unsigned interval
        if self.start.cast_unsigned() <= self.end.cast_unsigned() {
            let interval = UnsignedInterval {
                min: self.start.cast_unsigned(),
                max: self.end.cast_unsigned(),
            };
            interval.contains_value(value.cast_unsigned())
        } else {
            let interval = SignedInterval {
                min: self.end.cast_signed(),
                max: self.start.cast_signed(),
            };
            interval.contains_value(value.cast_signed())
        }
    }

    pub fn interpret(self) -> WrappingInterpretation<W> {
        if self.start.cast_unsigned() <= self.end.cast_unsigned() {
            // does not contain the unsigned seam
            if self.start.cast_signed() <= self.start.cast_signed() {
                // does not contain the any seam
                WrappingInterpretation::Signless(SignlessInterval {
                    min: self.start,
                    max: self.end,
                })
            } else {
                // contains the signed seam, but not the unsigned seam
                // can be only interpreted as unsigned
                WrappingInterpretation::Unsigned(UnsignedInterval {
                    min: self.start.cast_unsigned(),
                    max: self.end.cast_unsigned(),
                })
            }
        } else {
            if self.start.cast_signed() <= self.start.cast_signed() {
                // contains the unsigned seam but not the signed seam
                // can only be interpreted as signed
                WrappingInterpretation::Signed(SignedInterval {
                    min: self.start.cast_signed(),
                    max: self.end.cast_signed(),
                })
            } else {
                // contains both the unsigned and signed seam
                // we must degrade this to a full interval
                WrappingInterpretation::Unsigned(UnsignedInterval::FULL)
            }
        }
    }

    pub fn start(&self) -> ConcreteBitvector<W> {
        self.start
    }

    pub fn end(&self) -> ConcreteBitvector<W> {
        self.end
    }
}

impl<const W: u32> WrappingInterval<W> {
    pub fn hw_add(self, rhs: Self) -> Self {
        // ensure the produced bounds are less than 2^L apart, produce a full interval otherwise
        if self.is_addsub_full(rhs) {
            Self::FULL
        } else {
            // wrapping and fully monotonic: add bounds
            let start = self.start.add(rhs.start);
            let end = self.end.add(rhs.end);

            Self { start, end }
        }
    }

    pub fn hw_sub(self, rhs: Self) -> Self {
        // ensure the produced bounds are less than 2^L apart, produce a full interval otherwise
        if self.is_addsub_full(rhs) {
            Self::FULL
        } else {
            // wrapping, monotonic on lhs, anti-monotonic on rhs: subtract bounds, remember to flip rhs bounds
            let start = self.start.sub(rhs.end);
            let end = self.end.sub(rhs.start);

            Self { start, end }
        }
    }

    pub fn hw_mul(self, rhs: Self) -> Self {
        let lhs_start = self.start;
        let rhs_start = rhs.start;
        let start = lhs_start.mul(rhs_start);
        let lhs_diff = self.bound_diff().as_bitvector();
        let rhs_diff = rhs.bound_diff().as_bitvector();

        let Some(diff_product) = lhs_diff.checked_mul(rhs_diff) else {
            return Self::FULL;
        };
        let Some(diff_start_product) = lhs_diff.checked_mul(rhs_start) else {
            return Self::FULL;
        };
        let Some(start_diff_product) = lhs_start.checked_mul(rhs_diff) else {
            return Self::FULL;
        };
        let Some(result_len) = diff_product
            .checked_add(diff_start_product)
            .and_then(|v| v.checked_add(start_diff_product))
        else {
            return Self::FULL;
        };

        let end = start.add(result_len);

        Self { start, end }
    }

    fn is_addsub_full(self, rhs: Self) -> bool {
        let lhs_diff = self.bound_diff();
        let rhs_diff = rhs.bound_diff();

        let wrapped_total_len = lhs_diff + rhs_diff;
        wrapped_total_len < lhs_diff || wrapped_total_len < rhs_diff
    }

    pub fn bound_diff(&self) -> UnsignedBitvector<W> {
        self.end.cast_unsigned() - self.start.cast_unsigned()
    }
}

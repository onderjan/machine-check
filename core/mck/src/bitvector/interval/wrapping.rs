use std::fmt::Debug;

use crate::{
    bitvector::interval::{SignedInterval, SignlessInterval, UnsignedInterval},
    concr::{ConcreteBitvector, UnsignedBitvector},
    forward::HwArith,
};

/// A wrapping interval.
///
/// If start <= end (unsigned), the interval represents [start,end].
/// If start > end, the interval represents the union of [T_MIN, end] and [start, T_MAX].
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct WrappingInterval<const W: u32> {
    pub(super) start: ConcreteBitvector<W>,
    pub(super) end: ConcreteBitvector<W>,
}

impl<const W: u32> WrappingInterval<W> {
    pub fn new(start: ConcreteBitvector<W>, end: ConcreteBitvector<W>) -> Self {
        Self { start, end }
    }

    // the canonical full interval is from zero to umax
    const FULL: Self = Self {
        start: ConcreteBitvector::<W>::zero(),
        end: ConcreteBitvector::<W>::const_umax(),
    };

    pub fn contains_value(&self, value: &ConcreteBitvector<W>) -> bool {
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
            if self.start.cast_signed() <= self.end.cast_signed() {
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
        } else if self.start.cast_signed() <= self.end.cast_signed() {
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

    pub fn start(&self) -> ConcreteBitvector<W> {
        self.start
    }

    pub fn end(&self) -> ConcreteBitvector<W> {
        self.end
    }
}

#[derive(Clone, Debug)]
pub enum WrappingInterpretation<const W: u32> {
    Signless(SignlessInterval<W>),
    Signed(SignedInterval<W>),
    Unsigned(UnsignedInterval<W>),
}

impl<const W: u32> Debug for WrappingInterval<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} --> {}]", self.start, self.end)
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

use std::fmt::Debug;

use crate::{
    bitvector::{
        interval::{SignedInterval, SignlessInterval, UnsignedInterval},
        BitvectorBound,
    },
    concr::{ConcreteBitvector, UnsignedBitvector},
    forward::HwArith,
};

/// A wrapping interval.
///
/// If start <= end (unsigned), the interval represents [start,end].
/// If start > end, the interval represents the union of [T_MIN, end] and [start, T_MAX].
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct WrappingInterval<B: BitvectorBound> {
    start: ConcreteBitvector<B>,
    end: ConcreteBitvector<B>,
}

impl<B: BitvectorBound> WrappingInterval<B> {
    pub fn new(start: ConcreteBitvector<B>, end: ConcreteBitvector<B>) -> Self {
        assert_eq!(start.bound(), end.bound());
        Self { start, end }
    }

    // the canonical full interval is from umin (zero) to umax (full mask)
    pub fn new_full(bound: B) -> Self {
        Self {
            start: ConcreteBitvector::new_umin(bound),
            end: ConcreteBitvector::new_umax(bound),
        }
    }

    pub fn bound(&self) -> B {
        // the bounds of start and end should be same
        self.start.bound()
    }

    #[allow(dead_code)]
    pub fn contains_value(&self, value: &ConcreteBitvector<B>) -> bool {
        // interpreted as unsigned interval
        if self.start.as_unsigned() <= self.end.as_unsigned() {
            let interval = UnsignedInterval::new(self.start.as_unsigned(), self.end.as_unsigned());
            interval.contains_value(value.as_unsigned())
        } else {
            let interval = SignedInterval::new(self.end.as_signed(), self.start.as_signed());
            interval.contains_value(value.as_signed())
        }
    }

    pub fn interpret(self) -> WrappingInterpretation<B> {
        if self.start.as_unsigned() <= self.end.as_unsigned() {
            // does not contain the unsigned seam
            if self.start.as_signed() <= self.end.as_signed() {
                // does not contain the any seam
                WrappingInterpretation::Signless(SignlessInterval::new(self.start, self.end))
            } else {
                // contains the signed seam, but not the unsigned seam
                // can be only interpreted as unsigned
                WrappingInterpretation::Unsigned(UnsignedInterval::new(
                    self.start.as_unsigned(),
                    self.end.as_unsigned(),
                ))
            }
        } else if self.start.as_signed() <= self.end.as_signed() {
            // contains the unsigned seam but not the signed seam
            // can only be interpreted as signed
            WrappingInterpretation::Signed(SignedInterval::new(
                self.start.as_signed(),
                self.end.as_signed(),
            ))
        } else {
            // contains both the unsigned and signed seam
            // we must degrade this to a full interval
            WrappingInterpretation::Unsigned(UnsignedInterval::new_full(self.bound()))
        }
    }
}

#[derive(Clone, Debug)]
pub enum WrappingInterpretation<B: BitvectorBound> {
    Signless(SignlessInterval<B>),
    Signed(SignedInterval<B>),
    Unsigned(UnsignedInterval<B>),
}

impl<B: BitvectorBound> Debug for WrappingInterval<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} --> {}]", self.start, self.end)
    }
}

impl<B: BitvectorBound> WrappingInterval<B> {
    pub fn hw_add(self, rhs: Self) -> Self {
        assert_eq!(self.bound(), rhs.bound());
        let bound = self.bound();

        // ensure the produced bounds are less than 2^L apart, produce a full interval otherwise
        if self.is_addsub_full(rhs) {
            Self::new_full(bound)
        } else {
            // wrapping and fully monotonic: add bounds
            let start = self.start.add(rhs.start);
            let end = self.end.add(rhs.end);

            Self { start, end }
        }
    }

    pub fn hw_sub(self, rhs: Self) -> Self {
        assert_eq!(self.bound(), rhs.bound());
        let bound = self.bound();

        // ensure the produced bounds are less than 2^L apart, produce a full interval otherwise
        if self.is_addsub_full(rhs) {
            Self::new_full(bound)
        } else {
            // wrapping, monotonic on lhs, anti-monotonic on rhs: subtract bounds, remember to flip rhs bounds
            let start = self.start.sub(rhs.end);
            let end = self.end.sub(rhs.start);

            Self { start, end }
        }
    }

    pub fn hw_mul(self, rhs: Self) -> Self {
        assert_eq!(self.bound(), rhs.bound());
        let bound = self.bound();

        let lhs_start = self.start;
        let rhs_start = rhs.start;
        let start = lhs_start.mul(rhs_start);
        let lhs_diff = self.bound_diff().cast_bitvector();
        let rhs_diff = rhs.bound_diff().cast_bitvector();

        let Some(diff_product) = lhs_diff.checked_mul(rhs_diff) else {
            return Self::new_full(bound);
        };
        let Some(diff_start_product) = lhs_diff.checked_mul(rhs_start) else {
            return Self::new_full(bound);
        };
        let Some(start_diff_product) = lhs_start.checked_mul(rhs_diff) else {
            return Self::new_full(bound);
        };
        let Some(result_len) = diff_product
            .checked_add(diff_start_product)
            .and_then(|v| v.checked_add(start_diff_product))
        else {
            return Self::new_full(bound);
        };

        let end = start.add(result_len);

        Self { start, end }
    }

    fn is_addsub_full(self, rhs: Self) -> bool {
        assert_eq!(self.bound(), rhs.bound());

        let lhs_diff = self.bound_diff();
        let rhs_diff = rhs.bound_diff();

        let wrapped_total_len = lhs_diff + rhs_diff;
        wrapped_total_len < lhs_diff || wrapped_total_len < rhs_diff
    }

    pub fn bound_diff(&self) -> UnsignedBitvector<B> {
        self.end.as_unsigned() - self.start.as_unsigned()
    }
}

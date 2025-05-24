use num::{
    traits::{WrappingAdd, WrappingMul, WrappingNeg, WrappingSub},
    PrimInt,
};

mod arith;

trait UnsignedPrimitive: PrimInt + WrappingAdd + WrappingSub + WrappingMul + WrappingNeg {
    type Signed: SignedPrimitive;

    const ZERO: Self;
    const UNDERHALF: Self;
    const OVERHALF: Self;
    const MAX: Self;

    fn cast_signed(self) -> Self::Signed;
}
trait SignedPrimitive: PrimInt + WrappingAdd + WrappingSub + WrappingMul + WrappingNeg {
    type Unsigned: UnsignedPrimitive;
    fn cast_unsigned(self) -> Self::Unsigned;
}

/// An interval, with a minimum and a maximum value.
///
/// It is guaranteed that min <= max, which means the interval
/// does not support wrapping nor representing an empty set.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct NonWrappingInterval<T: Ord + Clone + Copy> {
    min: T,
    max: T,
}

impl<T: Ord + Clone + Copy> NonWrappingInterval<T> {
    fn from_value(value: T) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    fn contains_value(self, other: T) -> bool {
        self.min <= other && other <= self.max
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
}

impl<U: UnsignedPrimitive> NonWrappingInterval<U> {
    fn into_wrapping(self) -> WrappingInterval<U> {
        WrappingInterval {
            start: self.min,
            end: self.max,
        }
    }
}

/// A wrapping interval.
///
/// If start <= end (unsigned), the interval represents [start,end].
/// If start > end, the interval represents the union of [T_MIN, end] and [start, T_MAX].
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct WrappingInterval<U: UnsignedPrimitive> {
    start: U,
    end: U,
}

impl<U: UnsignedPrimitive> WrappingInterval<U> {
    fn from_value(value: U) -> Self {
        Self {
            start: value,
            end: value,
        }
    }

    fn full() -> Self {
        Self {
            start: U::ZERO,
            end: U::MAX,
        }
    }

    fn contains_value(self, value: U) -> bool {
        if self.start <= self.end {
            self.start <= value && value <= self.end
        } else {
            value <= self.end || self.start <= value
        }
    }
}

/// Dual-interval domain.
///
/// The idea is that the signedness of operations on the variable only really
/// impacts the continuity of the highest bit: the wrapping point is located
/// between -1 and 0 for unsigned, and between (2^N)/2-1 and (2^N)/2 for signed.
/// As such, we will consider the halves completely separately with distinct
/// intervals for each half.
///
/// The near half is located between (including) 0 and (2^N)/2-1 when interpreted
/// both as unsigned and signed. The far half is located between (including)
/// (2^N)/2 and (2^N)-1 when interpreted as unsigned and betweeen (including)
/// -(2^N)/2 and -1 when considered as signed in two's complement.
///
/// The only exception is when one of the halves does not have any value present,
/// in which case both intervals will be set equal to each other.
///
/// Unlike wrapping intervals in general, this domain forms a lattice,
/// since each half can be thought about as selecting elements from its own half
/// but admitting every element from the other half, and this domain is
/// their product (in the meaning of abstract interpretation). We pay for the
/// increased precision compared to a standard interval domain (which forms a lattice)
/// or a wrapping-interval domain (which does not, but can be more precise than
/// a non-wrapping interval) by an increase in time and memory, which should not
/// be problematic for our use.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) struct DualInterval<U: UnsignedPrimitive> {
    // The interval usually located between (including) 0 and (2^N)/2-1.
    //
    // If it is not, it must be equal to the far half.
    near_half: NonWrappingInterval<U>,
    // The interval usually located between (including) (2^N)/2 and (2^N)-1.
    //
    // If it is not, it must be equal to the near half.
    far_half: NonWrappingInterval<U>,
}

impl<U: UnsignedPrimitive> DualInterval<U> {
    pub fn from_value(value: U) -> Self {
        Self {
            near_half: NonWrappingInterval::from_value(value),
            far_half: NonWrappingInterval::from_value(value),
        }
    }

    pub fn contains_value(self, value: U) -> bool {
        self.near_half.contains_value(value) || self.far_half.contains_value(value)
    }

    pub fn full() -> Self {
        Self {
            near_half: NonWrappingInterval {
                min: U::ZERO,
                max: U::UNDERHALF,
            },
            far_half: NonWrappingInterval {
                min: U::OVERHALF,
                max: U::MAX,
            },
        }
    }

    fn from_wrapping_interval(a: WrappingInterval<U>) -> Self {
        let (near_half, far_half) = opt_halves(a);
        Self::from_opt_halves(near_half, far_half)
    }

    fn from_wrapping_intervals(intervals: &[WrappingInterval<U>]) -> Self {
        let mut near_half = None;
        let mut far_half = None;

        for interval in intervals {
            let (interval_near_half, interval_far_half) = opt_halves(*interval);
            near_half = NonWrappingInterval::union_opt(near_half, interval_near_half);
            far_half = NonWrappingInterval::union_opt(far_half, interval_far_half);
        }

        Self::from_opt_halves(near_half, far_half)
    }

    fn from_opt_halves(
        near_half: Option<NonWrappingInterval<U>>,
        far_half: Option<NonWrappingInterval<U>>,
    ) -> Self {
        let near_half = near_half.unwrap_or(NonWrappingInterval {
            min: U::ZERO,
            max: U::UNDERHALF,
        });
        let far_half = far_half.unwrap_or(NonWrappingInterval {
            min: U::OVERHALF,
            max: U::MAX,
        });
        Self {
            near_half,
            far_half,
        }
    }
}

fn opt_halves<U: UnsignedPrimitive>(
    a: WrappingInterval<U>,
) -> (
    Option<NonWrappingInterval<U>>,
    Option<NonWrappingInterval<U>>,
) {
    let preserves_unsigned_seam = a.start <= a.end;
    let preserves_signed_seam = a.start >= U::OVERHALF || a.end < U::OVERHALF;

    // We need to split the wrapping interval to two if it crosses a signed
    // or unsigned seam.
    match (preserves_unsigned_seam, preserves_signed_seam) {
        (true, true) => {
            // Preserves both seams.
            // Just return the interval in the given half.
            let not_crossing = NonWrappingInterval {
                min: a.start,
                max: a.end,
            };
            if a.end <= U::OVERHALF {
                (Some(not_crossing), None)
            } else {
                (None, Some(not_crossing))
            }
        }
        (true, false) => {
            // Preserves the unsigned seam, but crosses the signed seam.
            (
                Some(NonWrappingInterval {
                    min: a.start,
                    max: U::UNDERHALF,
                }),
                Some(NonWrappingInterval {
                    min: U::OVERHALF,
                    max: a.end,
                }),
            )
        }
        (false, true) => {
            // Crosses the unsigned seam, but preserves the signed seam.
            (
                Some(NonWrappingInterval {
                    min: U::ZERO,
                    max: a.end,
                }),
                Some(NonWrappingInterval {
                    min: a.start,
                    max: U::MAX,
                }),
            )
        }
        (false, false) => {
            // Crosses both seams. This means that both halves must contain
            // the values at both ends. We can only represent this
            // in the dual-interval domain using the full interval.
            (
                Some(NonWrappingInterval {
                    min: U::ZERO,
                    max: U::UNDERHALF,
                }),
                Some(NonWrappingInterval {
                    min: U::OVERHALF,
                    max: U::MAX,
                }),
            )
        }
    }
}

/// An interval, with a minimum and a maximum value.
///
/// It is guaranteed that min <= max, which means the interval
/// does not support wrapping nor representing an empty set.
#[derive(Clone, Copy, Hash)]
pub(crate) struct NonWrappingInterval<T: Ord + Clone + Copy> {
    min: T,
    max: T,
}

impl<T: Ord + Clone + Copy> NonWrappingInterval<T> {
    pub fn from_value(value: T) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    pub fn contains_value(self, other: T) -> bool {
        self.min <= other && other <= self.max
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
/// a non-wrapping interval) by a 2x increase in (rough) time and memory, which
/// should not be problematic for most uses.
#[derive(Clone, Copy, Hash)]
pub(crate) struct DualInterval<T: Ord + Clone + Copy> {
    // The interval usually located between (including) 0 and (2^N)/2-1.
    //
    // If it is not, it must be equal to the far half.
    near_half: NonWrappingInterval<T>,
    // The interval usually located between (including) (2^N)/2 and (2^N)-1.
    //
    // If it is not, it must be equal to the near half.
    far_half: NonWrappingInterval<T>,
}

#[allow(dead_code)]
impl<T: Ord + Clone + Copy> DualInterval<T> {
    pub fn from_value(value: T) -> Self {
        Self {
            near_half: NonWrappingInterval::from_value(value),
            far_half: NonWrappingInterval::from_value(value),
        }
    }

    pub fn contains_value(self, value: T) -> bool {
        self.near_half.contains_value(value) || self.far_half.contains_value(value)
    }
}

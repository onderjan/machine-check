use std::fmt::{Debug, Display};

use crate::{bitvector::concrete::ConcreteBitvector, bitvector::util, forward::HwArith};

use super::Bitvector;

impl<const L: u32> Bitvector<L> {
    #[must_use]
    pub fn new(value: u64) -> Self {
        Self::from_concrete(ConcreteBitvector::new(value))
    }

    #[must_use]
    pub fn from_concrete(value: ConcreteBitvector<L>) -> Self {
        Self::from_wrap_interval(value, value)
    }

    pub fn from_wrap_interval(start: ConcreteBitvector<L>, end: ConcreteBitvector<L>) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub fn new_unknown() -> Self {
        Self {
            start: Self::representable_umin(),
            end: Self::representable_umax(),
        }
    }

    #[must_use]
    pub fn representable_umin() -> ConcreteBitvector<L> {
        ConcreteBitvector::new(0)
    }

    #[must_use]
    pub fn representable_umax() -> ConcreteBitvector<L> {
        ConcreteBitvector::new(util::compute_u64_mask(L))
    }

    #[must_use]
    pub fn concrete_value(&self) -> Option<ConcreteBitvector<L>> {
        if self.start == self.end {
            Some(self.start)
        } else {
            None
        }
    }

    #[must_use]
    pub fn contains(&self, rhs: &Self) -> bool {
        println!("Does {} contain {}?", self, rhs);
        // handle full intervals specially to avoid corner cases
        if self.end.add(ConcreteBitvector::new(1)) == self.start {
            return true;
        }

        if self.start.as_unsigned() <= self.end.as_unsigned() {
            if rhs.start.as_unsigned() <= rhs.end.as_unsigned() {
                // this interval is [self.start, self.end]
                // the other interval is [rhs.start, rhs.end]
                self.start.as_unsigned() <= rhs.start.as_unsigned()
                    && rhs.end.as_unsigned() <= self.end.as_unsigned()
            } else {
                // this interval is non-wrapping, the other is wrapping
                // this interval would have to be full to contain it, but is not
                false
            }
        } else if rhs.start.as_unsigned() <= rhs.end.as_unsigned() {
            // the other interval [rhs.start, rhs.end] must be inside of either
            // [self.start, repr_max] or [repr_min, self.end]
            // the inequalities with representable are always true
            self.start.as_unsigned() <= rhs.start.as_unsigned()
                || rhs.end.as_unsigned() <= self.end.as_unsigned()
        } else {
            // this interval is [self.start, repr_max] joined by [repr_min, self.end]
            // the other interval is [rhs.start, repr_max] joined by [repr_min, rhs.end]
            // the inequalities with representable are always true
            self.start.as_unsigned() <= rhs.start.as_unsigned()
                && rhs.end.as_unsigned() <= self.end.as_unsigned()
        }
    }

    #[must_use]
    pub fn contains_concrete(&self, a: &ConcreteBitvector<L>) -> bool {
        if self.start.as_unsigned() <= self.end.as_unsigned() {
            // the value must be inside [self.start, self.end]
            self.start.as_unsigned() <= a.as_unsigned() && a.as_unsigned() <= self.end.as_unsigned()
        } else {
            // the value must be inside of either
            // [self.start, repr_max] or [repr_min, self.end]
            // the inequalities with representable are always true
            self.start.as_unsigned() <= a.as_unsigned() || a.as_unsigned() <= self.end.as_unsigned()
        }
    }

    #[must_use]
    pub fn concrete_join(&self, concrete: ConcreteBitvector<L>) -> Self {
        // do nothing if it already is in interval
        if self.contains_concrete(&concrete) {
            return *self;
        }

        // outside the interval, that means we can replace either bound with it
        // select the bound that results in smaller interval
        // prefer increasing the maximum
        if self.end.sub(concrete).as_unsigned() <= concrete.sub(self.start).as_unsigned() {
            Self::from_wrap_interval(concrete, self.end)
        } else {
            Self::from_wrap_interval(self.start, concrete)
        }
    }

    pub fn all_with_length_iter() -> impl Iterator<Item = Self> {
        let start_iter = ConcreteBitvector::<L>::all_with_length_iter();
        start_iter.flat_map(|start| {
            let end_iter = ConcreteBitvector::<L>::all_with_length_iter();
            end_iter.map(move |end| Self::from_wrap_interval(start, end))
        })
    }
}

impl<const L: u32> Default for Bitvector<L> {
    fn default() -> Self {
        // default to fully unknown
        Self::new_unknown()
    }
}

impl<const L: u32> Debug for Bitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{}]", self.start, self.end)
    }
}

impl<const L: u32> Display for Bitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

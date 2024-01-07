use std::{
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
};

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
    pub fn full() -> Self {
        Self {
            start: Self::representable_umin(),
            end: Self::representable_umax(),
        }
    }

    pub fn len(&self) -> Option<ConcreteBitvector<L>> {
        // length of full interval does not fit into the concrete bitvector
        if !self.is_full() {
            Some(self.end.sub(self.start))
        } else {
            None
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

    pub fn is_full(&self) -> bool {
        if L == 0 {
            return true;
        }

        self.end.add(ConcreteBitvector::new(1)) == self.start
    }

    #[must_use]
    pub fn contains(&self, rhs: &Self) -> bool {
        // handle full intervals specially to avoid corner cases
        if self.is_full() {
            return true;
        }
        if rhs.is_full() {
            return false;
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
    pub fn intersects(&self, rhs: &Self) -> bool {
        println!("Does {} intersect {}?", self, rhs);
        // handle full intervals specially to avoid corner cases
        if self.is_full() {
            return true;
        }
        if rhs.is_full() {
            return true;
        }

        if self.start.as_unsigned() <= self.end.as_unsigned() {
            if rhs.start.as_unsigned() <= rhs.end.as_unsigned() {
                println!("Both intervals non-wrapping");
                // this interval is [self.start, self.end]
                // the other interval is [rhs.start, rhs.end]
                (rhs.start.as_unsigned() <= self.start.as_unsigned()
                    && self.start.as_unsigned() <= rhs.end.as_unsigned())
                    || (rhs.start.as_unsigned() <= self.end.as_unsigned()
                        && self.end.as_unsigned() <= rhs.end.as_unsigned())
                    || (self.start.as_unsigned() <= rhs.start.as_unsigned()
                        && rhs.start.as_unsigned() <= self.end.as_unsigned())
                    || (self.start.as_unsigned() <= rhs.end.as_unsigned()
                        && rhs.end.as_unsigned() <= self.end.as_unsigned())
            } else {
                // this interval is [self.start, self.end]
                // the other interval is [rhs.start, repr_max] joined by [repr_min, rhs.end]
                self.end.as_unsigned() >= rhs.start.as_unsigned()
                    || self.start.as_unsigned() <= rhs.end.as_unsigned()
            }
        } else if rhs.start.as_unsigned() <= rhs.end.as_unsigned() {
            // the other interval [rhs.start, rhs.end] must be intersect either
            // [self.start, repr_max] or [repr_min, self.end]
            // the inequalities with representable are always true
            self.start.as_unsigned() <= rhs.end.as_unsigned()
                || rhs.start.as_unsigned() <= self.end.as_unsigned()
        } else {
            // this interval is [self.start, repr_max] joined by [repr_min, self.end]
            // the other interval is [rhs.start, repr_max] joined by [repr_min, rhs.end]
            // they definitely do intersect on wrapping elements
            true
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

        println!("Joining concrete {} to {}", concrete, self);

        // outside the interval, that means we can replace either bound with it
        // select the bound that results in smaller interval
        // prefer increasing the maximum
        if self.end.sub(concrete).as_unsigned() <= concrete.sub(self.start).as_unsigned() {
            Self::from_wrap_interval(concrete, self.end)
        } else {
            Self::from_wrap_interval(self.start, concrete)
        }
    }

    pub fn concrete_iter(&self) -> impl Iterator<Item = ConcreteBitvector<L>> {
        BitvectorIterator {
            current: Some(self.start),
            end: self.end,
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

impl<const L: u32> PartialEq for Bitvector<L> {
    fn eq(&self, other: &Self) -> bool {
        // all full intervals are the same
        if self.is_full() {
            return other.is_full();
        }

        // otherwise, compare the bounds
        self.start == other.start && self.end == other.end
    }
}

impl<const L: u32> Eq for Bitvector<L> {}

impl<const L: u32> Hash for Bitvector<L> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // all full intervals are the same, do not write to hasher
        if !self.is_full() {
            self.start.hash(state);
            self.end.hash(state);
        }
    }
}

impl<const L: u32> Default for Bitvector<L> {
    fn default() -> Self {
        // default to fully unknown
        Self::full()
    }
}

impl<const L: u32> Debug for Bitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start.as_unsigned() <= self.end.as_unsigned() {
            write!(f, "[{},{}]", self.start, self.end)
        } else {
            write!(f, "[{},{}] (mod 2^{})", self.start, self.end, L)
        }
    }
}

impl<const L: u32> Display for Bitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

struct BitvectorIterator<const L: u32> {
    current: Option<ConcreteBitvector<L>>,
    end: ConcreteBitvector<L>,
}

impl<const L: u32> Iterator for BitvectorIterator<L> {
    type Item = ConcreteBitvector<L>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(result) = self.current else {
            return None;
        };
        if result != self.end {
            self.current = Some(result.add(ConcreteBitvector::new(1)));
        } else {
            self.current = None;
        }
        Some(result)
    }
}

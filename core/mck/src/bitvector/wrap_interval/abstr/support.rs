use std::{
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
};

use crate::{
    bitvector::concrete::ConcreteBitvector,
    bitvector::{util, wrap_interval::interval::Interval},
    forward::HwArith,
    unsigned::Unsigned,
};

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

    pub(crate) fn from_unsigned_interval(interval: Interval<Unsigned<L>>) -> Self {
        Self {
            start: interval.min.as_bitvector(),
            end: interval.max.as_bitvector(),
        }
    }

    pub(crate) fn from_unsigned_intervals(mut intervals: Vec<Interval<Unsigned<L>>>) -> Self {
        assert!(!intervals.is_empty());

        intervals.sort_unstable_by(|a, b| a.min.cmp(&b.min));

        // join intervals first
        let mut index = 0;
        while index + 1 < intervals.len() {
            let next = intervals[index + 1];
            let current = &mut intervals[index];
            if current.max >= next.min {
                // unionize
                current.max = current.max.max(next.max);
                intervals.remove(index + 1);
            } else {
                index += 1;
            }
        }

        let mut largest_hole_index = 0;
        let mut largest_hole = 0;
        for (index, (current, next)) in intervals
            .iter()
            .cloned()
            .zip(intervals.iter().skip(1).cloned())
            .chain(std::iter::once((
                *intervals.last().unwrap(),
                *intervals.first().unwrap(),
            )))
            .enumerate()
        {
            let current_bitvec = current.max.as_bitvector();
            let next_bitvec = next.min.as_bitvector();
            let hole = next_bitvec.sub(current_bitvec).as_unsigned();
            if hole > largest_hole {
                largest_hole = hole;
                largest_hole_index = index;
            }
        }

        let end = intervals[largest_hole_index].max;
        let start = intervals
            .get(largest_hole_index + 1)
            .unwrap_or_else(|| intervals.first().unwrap())
            .min;

        Self::from_wrap_interval(start.as_bitvector(), end.as_bitvector())
        //println!("From sorted intervals {:?}: {}", intervals, result);
    }

    #[must_use]
    pub fn full() -> Self {
        Self {
            start: Self::representable_umin(),
            end: Self::representable_umax(),
        }
    }

    pub fn bound_diff(&self) -> ConcreteBitvector<L> {
        self.end.sub(self.start)
    }

    pub fn hole_diff(&self) -> ConcreteBitvector<L> {
        self.start.sub(self.end)
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
    pub fn representable_smin() -> ConcreteBitvector<L> {
        // signed minimum has only the sign bit set
        ConcreteBitvector::new(util::compute_u64_sign_bit_mask(L))
    }

    #[must_use]
    pub fn representable_smax() -> ConcreteBitvector<L> {
        if L == 0 {
            return ConcreteBitvector::new(0);
        }
        // signed maximum is wrapping one lower than signed minimum
        Self::representable_smin().sub(ConcreteBitvector::new(1))
    }

    #[must_use]
    pub fn umin(&self) -> ConcreteBitvector<L> {
        if self.start.as_unsigned() <= self.end.as_unsigned() {
            // non-wrapping
            self.start
        } else {
            // wrapping
            Self::representable_umin()
        }
    }

    #[must_use]
    pub fn umax(&self) -> ConcreteBitvector<L> {
        if self.start.as_unsigned() <= self.end.as_unsigned() {
            // non-wrapping
            self.end
        } else {
            // wrapping
            Self::representable_umax()
        }
    }

    #[must_use]
    pub fn smin(&self) -> ConcreteBitvector<L> {
        if self.start.as_signed() <= self.end.as_signed() {
            // non-wrapping in signed representation
            self.start
        } else {
            // wrapping in signed representation
            Self::representable_smin()
        }
    }

    #[must_use]
    pub fn smax(&self) -> ConcreteBitvector<L> {
        if self.start.as_signed() <= self.end.as_signed() {
            // non-wrapping in signed representation
            self.end
        } else {
            // wrapping in signed representation
            Self::representable_smax()
        }
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
        // handle full intervals specially to avoid corner cases
        if self.is_full() {
            return true;
        }
        if rhs.is_full() {
            return true;
        }

        if self.start.as_unsigned() <= self.end.as_unsigned() {
            if rhs.start.as_unsigned() <= rhs.end.as_unsigned() {
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

    pub(crate) fn unsigned_intervals(&self) -> Vec<Interval<Unsigned<L>>> {
        if self.start.as_unsigned() <= self.end.as_unsigned() {
            // single interval
            vec![Interval::new(
                Unsigned::from_bitvector(self.start),
                Unsigned::from_bitvector(self.end),
            )]
        } else {
            // start with lowest
            // interval from representable minimum to end
            // interval from start to representable maximum
            vec![
                Interval::new(
                    Unsigned::from_bitvector(Self::representable_umin()),
                    Unsigned::from_bitvector(self.end),
                ),
                Interval::new(
                    Unsigned::from_bitvector(self.start),
                    Unsigned::from_bitvector(Self::representable_umax()),
                ),
            ]
        }
    }

    pub(crate) fn unsigned_interval(&self) -> Interval<Unsigned<L>> {
        let start = self.start;
        let end = self.end;
        if start.as_unsigned() <= end.as_unsigned() {
            // single interval
            Interval::new(
                Unsigned::from_bitvector(start),
                Unsigned::from_bitvector(end),
            )
        } else {
            // full interval
            Interval::new(
                Unsigned::from_bitvector(Self::representable_umin()),
                Unsigned::from_bitvector(Self::representable_umax()),
            )
        }
    }

    pub(crate) fn offset_signed_interval(&self) -> Interval<Unsigned<L>> {
        let start = self.start.as_offset_signed();
        let end = self.end.as_offset_signed();

        if start <= end {
            // single interval
            Interval::new(Unsigned::new(start), Unsigned::new(end))
        } else {
            // full interval
            Interval::new(
                Unsigned::from_bitvector(Self::representable_umin()),
                Unsigned::from_bitvector(Self::representable_umax()),
            )
        }
    }

    pub(crate) fn negative_intervals(&self) -> Vec<Interval<Unsigned<L>>> {
        self.unsigned_intervals()
            .iter()
            .filter_map(|v| {
                if v.max.as_bitvector().as_unsigned() >= Self::representable_smin().as_unsigned() {
                    let min = Unsigned::<L>::new(
                        v.min
                            .as_bitvector()
                            .as_unsigned()
                            .max(Self::representable_smin().as_unsigned()),
                    );
                    let max = v.max;
                    Some(Interval::new(min, max))
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn absolute_negative_intervals(&self) -> Vec<Interval<Unsigned<L>>> {
        self.unsigned_intervals()
            .iter()
            .filter_map(|v| {
                if v.max.as_bitvector().as_unsigned() >= Self::representable_smin().as_unsigned() {
                    let unsigned_min = Unsigned::<L>::new(
                        v.min
                            .as_bitvector()
                            .as_unsigned()
                            .max(Self::representable_smin().as_unsigned()),
                    );
                    let unsigned_max = v.max.as_bitvector();
                    let absolute_negative_min = unsigned_max.arith_neg();
                    let absolute_negative_max = unsigned_min.as_bitvector().arith_neg();

                    Some(Interval::new(
                        Unsigned::from_bitvector(absolute_negative_min),
                        Unsigned::from_bitvector(absolute_negative_max),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn nonnegative_intervals(&self) -> Vec<Interval<Unsigned<L>>> {
        self.unsigned_intervals()
            .iter()
            .filter_map(|v| {
                if v.min.as_bitvector().as_unsigned() <= Self::representable_smax().as_unsigned() {
                    Some(Interval::new(
                        v.min,
                        Unsigned::new(
                            v.max
                                .as_bitvector()
                                .as_unsigned()
                                .min(Self::representable_smax().as_unsigned()),
                        ),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn positive_intervals(&self) -> Vec<Interval<Unsigned<L>>> {
        self.unsigned_intervals()
            .iter()
            .filter_map(|v| {
                if v.max > Unsigned::zero()
                    && v.min <= Unsigned::new(Self::representable_smax().as_unsigned())
                {
                    let min_val = v.min.as_bitvector().as_unsigned().max(1);
                    Some(Interval::new(
                        Unsigned::new(v.min.as_bitvector().as_unsigned().max(1)),
                        Unsigned::new(
                            v.max
                                .as_bitvector()
                                .as_unsigned()
                                .min(Self::representable_smax().as_unsigned())
                                .max(min_val),
                        ),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    #[must_use]
    pub fn contains_concrete(&self, a: &ConcreteBitvector<L>) -> bool {
        for interval in self.unsigned_intervals() {
            if interval.contains_single(Unsigned::from_bitvector(*a)) {
                return true;
            }
        }
        false
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

impl Bitvector<1> {
    pub fn from_bools(can_be_false: bool, can_be_true: bool) -> Self {
        assert!(can_be_false || can_be_true);
        Self {
            start: ConcreteBitvector::new(if can_be_false { 0 } else { 1 }),
            end: ConcreteBitvector::new(if can_be_true { 1 } else { 0 }),
        }
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

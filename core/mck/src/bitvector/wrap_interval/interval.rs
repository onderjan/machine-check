mod bitwise;

use core::fmt::Debug;
use core::fmt::Display;
use std::ops::Add;

use crate::unsigned::Unsigned;

#[derive(Clone, Copy, Hash)]
pub struct Interval<const L: u32> {
    pub min: Unsigned<L>,
    pub max: Unsigned<L>,
}

impl<const L: u32> Interval<L> {
    pub fn new(min: Unsigned<L>, max: Unsigned<L>) -> Self {
        assert!(min <= max);
        Self { min, max }
    }

    pub fn contains(self, other: Self) -> bool {
        self.min <= other.min && other.max <= self.max
    }

    pub fn contains_single(self, other: Unsigned<L>) -> bool {
        self.min <= other && other <= self.max
    }

    pub fn iter(&self) -> impl Iterator<Item = Unsigned<L>> {
        IntervalIterator {
            current: Some(self.min),
            max: self.max,
        }
    }

    pub fn intersects(self, other: Self) -> bool {
        // one of the four bounds must be within the two bounds of the other interval
        self.min <= other.min && other.min <= self.max
            || self.min <= other.max && other.max <= self.max
            || other.min <= self.min && self.min <= other.max
            || other.min <= self.max && self.max <= other.max
    }

    pub fn all_pairs_gt(self, other: Self) -> bool {
        self.min > other.max
    }

    pub fn all_pairs_gte(self, other: Self) -> bool {
        self.min >= other.max
    }

    pub fn all_pairs_lt(self, other: Self) -> bool {
        self.max < other.min
    }

    pub fn all_pairs_lte(self, other: Self) -> bool {
        self.max <= other.min
    }

    pub fn some_pairs_gt(self, other: Self) -> bool {
        self.max > other.min
    }

    pub fn some_pairs_gte(self, other: Self) -> bool {
        self.max >= other.min
    }

    pub fn some_pairs_lt(self, other: Self) -> bool {
        self.min < other.max
    }

    pub fn some_pairs_lte(self, other: Self) -> bool {
        self.min <= other.max
    }

    pub fn all_pairs_eq(self, other: Self) -> bool {
        self.all_pairs_lte(other) && self.all_pairs_gte(other)
    }

    pub fn some_pairs_eq(self, other: Self) -> bool {
        // cannot combine
        // equivalent to intersection
        self.intersects(other)
    }
}

impl<const L: u32> Debug for Interval<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{}]", self.min, self.max)
    }
}

impl<const L: u32> Display for Interval<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

struct IntervalIterator<const L: u32> {
    pub current: Option<Unsigned<L>>,
    pub max: Unsigned<L>,
}

impl<const L: u32> Iterator for IntervalIterator<L> {
    type Item = Unsigned<L>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(current) = self.current else {
            return None;
        };
        if current == self.max {
            self.current = None;
            return Some(current);
        }

        self.current = Some(current.add(Unsigned::one()));
        Some(current)
    }
}

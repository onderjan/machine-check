use core::fmt::Debug;
use core::fmt::Display;

#[derive(Clone, Copy, Hash)]
pub struct Interval {
    min: u64,
    max: u64,
}

impl Interval {
    pub fn new(min: u64, max: u64) -> Self {
        assert!(min <= max);
        Self { min, max }
    }

    pub fn contains(self, other: Interval) -> bool {
        self.min <= other.min && other.max <= self.max
    }

    pub fn contains_single(self, other: u64) -> bool {
        self.min <= other && other <= self.max
    }

    pub fn intersects(self, other: Interval) -> bool {
        // one of the four bounds must be within the two bounds of the other interval
        self.min <= other.min && other.min <= self.max
            || self.min <= other.max && other.max <= self.max
            || other.min <= self.min && self.min <= other.max
            || other.min <= self.max && self.max <= other.max
    }

    pub fn all_pairs_gt(self, other: Interval) -> bool {
        self.min > other.max
    }

    pub fn all_pairs_gte(self, other: Interval) -> bool {
        self.min >= other.max
    }

    pub fn all_pairs_lt(self, other: Interval) -> bool {
        self.max < other.min
    }

    pub fn all_pairs_lte(self, other: Interval) -> bool {
        self.max <= other.min
    }

    pub fn some_pairs_gt(self, other: Interval) -> bool {
        self.max > other.min
    }

    pub fn some_pairs_gte(self, other: Interval) -> bool {
        self.max >= other.min
    }

    pub fn some_pairs_lt(self, other: Interval) -> bool {
        self.min < other.max
    }

    pub fn some_pairs_lte(self, other: Interval) -> bool {
        self.min <= other.max
    }

    pub fn all_pairs_eq(self, other: Interval) -> bool {
        self.all_pairs_lte(other) && self.all_pairs_gte(other)
    }

    pub fn some_pairs_eq(self, other: Interval) -> bool {
        // cannot combine
        // equivalent to intersection
        self.intersects(other)
    }
}

impl Debug for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{}]", self.min, self.max)
    }
}

impl Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

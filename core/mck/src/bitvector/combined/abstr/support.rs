use std::fmt::{Debug, Display};

use crate::{
    abstr::Abstr,
    bitvector::{concr, support::UnsignedBitvector, three_valued, wrap_interval},
};

use super::Bitvector;

impl<const L: u32> Abstr<concr::Bitvector<L>> for Bitvector<L> {
    fn from_concrete(value: concr::Bitvector<L>) -> Self {
        Self {
            three_valued: three_valued::AbstractBitvector::from_concrete(value),
            wrap_interval: wrap_interval::AbstractBitvector::from_concrete(value),
        }
    }
}

impl<const L: u32> Bitvector<L> {
    pub fn from_join(
        t: three_valued::AbstractBitvector<L>,
        w: wrap_interval::AbstractBitvector<L>,
    ) -> Self {
        let t_interval = wrap_interval::interval::Interval::new(
            UnsignedBitvector::from_bitvector(t.umin()),
            UnsignedBitvector::from_bitvector(t.umax()),
        );
        let new_w = wrap_interval::AbstractBitvector::from_unsigned_intervals(
            w.unsigned_intervals()
                .iter()
                .filter_map(|v| v.intersection(t_interval))
                .collect(),
        );
        let new_w_interval = w.unsigned_interval();
        let new_w_three_valued = three_valued::AbstractBitvector::from_interval(
            new_w_interval.min.as_bitvector(),
            new_w_interval.max.as_bitvector(),
        );
        let new_t = t.intersection(&new_w_three_valued);
        Self {
            three_valued: new_t,
            wrap_interval: new_w,
        }
    }
}

impl<const L: u32> Debug for Bitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} \u{2293} {})", self.three_valued, self.wrap_interval)
    }
}

impl<const L: u32> Display for Bitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

use std::fmt::{Debug, Display};

use crate::{
    abstr::Abstr,
    bitvector::{
        abstr::{dual_interval::DualInterval, ThreeValuedBitvector},
        concr,
    },
    misc::MetaEq,
};

use super::CombinedBitvector;

impl<const W: u32> Abstr<concr::Bitvector<W>> for CombinedBitvector<W> {
    fn from_concrete(value: concr::Bitvector<W>) -> Self {
        Self {
            three_valued: ThreeValuedBitvector::from_concrete(value),
            dual_interval: DualInterval::from_value(value),
        }
    }
}

impl<const W: u32> MetaEq for CombinedBitvector<W> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.three_valued.meta_eq(&other.three_valued)
            && self.dual_interval.meta_eq(&other.dual_interval)
    }
}

impl<const W: u32> Debug for CombinedBitvector<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}) \u{2293} ({})",
            self.three_valued, self.dual_interval
        )
    }
}

impl<const W: u32> Display for CombinedBitvector<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl CombinedBitvector<1> {
    pub fn can_be_true(self) -> bool {
        self.three_valued.can_be_true()
    }

    pub fn can_be_false(self) -> bool {
        self.three_valued.can_be_false()
    }
}

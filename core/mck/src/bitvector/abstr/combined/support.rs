use std::fmt::{Debug, Display};

use crate::{
    abstr::{Abstr, AbstractValue, BitvectorDomain},
    bitvector::{
        abstr::{combined::RCombinedBitvector, dual_interval::DualInterval, ThreeValuedBitvector},
        concr,
    },
    misc::MetaEq,
};

use super::CombinedBitvector;

impl<const W: u32> CombinedBitvector<W> {
    pub(crate) fn from_concrete_value(value: concr::Bitvector<W>) -> Self {
        todo!()
        /*Self {
            three_valued: ThreeValuedBitvector::from_concrete(value),
            dual_interval: DualInterval::from_value(value),
        }*/
    }

    pub(crate) fn from_runtime_bitvector(value: RCombinedBitvector) -> Self {
        todo!()
    }

    pub(crate) fn as_runtime_bitvector(&self) -> RCombinedBitvector {
        todo!()
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
        std::fmt::Debug::fmt(&self.three_valued, f)?;
        write!(f, " ⊓ ")?;
        std::fmt::Debug::fmt(&self.dual_interval, f)?;
        Ok(())
    }
}

impl<const W: u32> Display for CombinedBitvector<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

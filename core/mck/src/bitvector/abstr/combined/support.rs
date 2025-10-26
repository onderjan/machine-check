use std::fmt::{Debug, Display};

use crate::misc::{BitvectorBound, MetaEq};

use super::CombinedBitvector;

impl<B: BitvectorBound> MetaEq for CombinedBitvector<B> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.three_valued.meta_eq(&other.three_valued)
            && self.dual_interval.meta_eq(&other.dual_interval)
    }
}

impl<B: BitvectorBound> Debug for CombinedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.three_valued, f)?;
        write!(f, " ⊓ ")?;
        std::fmt::Debug::fmt(&self.dual_interval, f)?;
        Ok(())
    }
}

impl<B: BitvectorBound> Display for CombinedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

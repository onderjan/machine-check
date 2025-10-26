use std::fmt::{Debug, Display};

use crate::{
    abstr::combined::DomainCombination,
    misc::{BitvectorBound, MetaEq},
};

use super::CombinedBitvector;

impl<B: BitvectorBound, D: DomainCombination<B>> MetaEq for CombinedBitvector<B, D> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.left.meta_eq(&other.left) && self.right.meta_eq(&other.right)
    }
}

impl<B: BitvectorBound, D: DomainCombination<B>> Debug for CombinedBitvector<B, D>
where
    D::Left: Debug,
    D::Right: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.left, f)?;
        write!(f, " ⊓ ")?;
        std::fmt::Debug::fmt(&self.right, f)?;
        Ok(())
    }
}

impl<B: BitvectorBound, D: DomainCombination<B>> Display for CombinedBitvector<B, D>
where
    D::Left: Display,
    D::Right: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.left, f)?;
        write!(f, " ⊓ ")?;
        std::fmt::Display::fmt(&self.right, f)?;
        Ok(())
    }
}

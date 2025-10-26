use std::fmt::{Debug, Display};

use crate::{
    abstr::BitvectorDomain,
    misc::{BitvectorBound, MetaEq},
};

use super::CombinedBitvector;

impl<B: BitvectorBound, L: BitvectorDomain<Bound = B>, R: BitvectorDomain<Bound = B>> MetaEq
    for CombinedBitvector<B, L, R>
{
    fn meta_eq(&self, other: &Self) -> bool {
        self.left.meta_eq(&other.left) && self.right.meta_eq(&other.right)
    }
}

impl<
        B: BitvectorBound,
        L: BitvectorDomain<Bound = B> + Debug,
        R: BitvectorDomain<Bound = B> + Debug,
    > Debug for CombinedBitvector<B, L, R>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.left, f)?;
        write!(f, " ⊓ ")?;
        std::fmt::Debug::fmt(&self.right, f)?;
        Ok(())
    }
}

impl<
        B: BitvectorBound,
        L: BitvectorDomain<Bound = B> + Display,
        R: BitvectorDomain<Bound = B> + Display,
    > Display for CombinedBitvector<B, L, R>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.left, f)?;
        write!(f, " ⊓ ")?;
        std::fmt::Display::fmt(&self.right, f)?;
        Ok(())
    }
}

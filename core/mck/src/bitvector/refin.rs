use crate::{misc::BitvectorBound, refin};
use std::{hash::Hash, num::NonZeroU8};

mod combined;
mod three_valued;

pub trait RefinementDomain: Clone + Copy + Hash {
    type Bound: BitvectorBound;
    type Abstr;

    fn bound(&self) -> Self::Bound;

    fn new_unmarked(width: u32) -> Self;
    fn new_marked(importance: NonZeroU8, width: u32) -> Self;
    fn new_marked_unimportant(width: u32) -> Self {
        Self::new_marked(NonZeroU8::new(1).unwrap(), width)
    }

    fn from_boolean(boolean: refin::Boolean) -> Self;
    fn to_condition(self) -> refin::Boolean;

    fn limit(self, abstract_bitvec: &Self::Abstr) -> Self;
    fn apply_join(&mut self, other: &Self);
    fn apply_refin(&mut self, offer: &Self) -> bool;
    fn force_decay(&self, target: &mut Self::Abstr);

    fn importance(&self) -> u8;
}

#[cfg(all(not(feature = "Zdual_interval"), not(feature = "Zeq_domain")))]
pub type RBitvector = three_valued::RMarkBitvector;

#[cfg(all(feature = "Zdual_interval", not(feature = "Zeq_domain")))]
pub type RBitvector =
    combined::RCombinedMark<crate::abstr::combination::TVDICombination<crate::misc::RBound>>;

#[cfg(all(not(feature = "Zdual_interval"), feature = "Zeq_domain"))]
pub type RBitvector =
    combined::RCombinedMark<crate::abstr::combination::TVEQCombination<crate::misc::RBound>>;

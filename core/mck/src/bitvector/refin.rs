use std::{hash::Hash, num::NonZeroU8};

#[cfg(feature = "Zdual_interval")]
use crate::abstr::dual_interval::RDualInterval;
use crate::{misc::BitvectorBound, refin};

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

#[cfg(not(feature = "Zdual_interval"))]
pub type RBitvector = three_valued::RMarkBitvector;

#[cfg(feature = "Zdual_interval")]
pub type RBitvector = combined::RCombinedMark<RDualInterval>;

use super::interval::UnsignedInterval;
use crate::abstr::{ManipField, Phi};
use std::{fmt::Display, hash::Hash};

mod combined;
mod dual_interval;
mod three_valued;

pub trait BitvectorDomain<const W: u32>: Clone + Copy + Hash + Phi + ManipField {
    fn unsigned_interval(&self) -> UnsignedInterval<W>;
    fn element_description(&self) -> BitvectorElement;

    fn join(self, other: Self) -> Self;
    fn meet(self, other: Self) -> Option<Self>;
}

pub(super) use combined::CombinedBitvector;
use serde::{Deserialize, Serialize};
pub(super) use three_valued::{RThreeValuedBitvector, ThreeValuedBitvector};

pub(crate) use dual_interval::DualIntervalFieldValue;
pub(crate) use three_valued::ThreeValuedFieldValue;

#[cfg(not(feature = "Zdual_interval"))]
pub type Bitvector<const W: u32> = three_valued::ThreeValuedBitvector<W>;
#[cfg(not(feature = "Zdual_interval"))]
pub type RBitvector = three_valued::RThreeValuedBitvector;

#[cfg(feature = "Zdual_interval")]
pub type Bitvector<const W: u32> = combined::CombinedBitvector<W>;
#[cfg(feature = "Zdual_interval")]
pub type RBitvector = combined::RCombinedBitvector;

pub type BooleanBitvector = Bitvector<1>;
pub type PanicBitvector = Bitvector<32>;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BitvectorElement {
    pub three_valued: Option<ThreeValuedFieldValue>,
    pub dual_interval: Option<DualIntervalFieldValue>,
}

impl BitvectorElement {
    fn write(&self, f: &mut std::fmt::Formatter<'_>, bit_width: u32) -> std::fmt::Result {
        if let Some(three_valued) = &self.three_valued {
            three_valued.write(f, bit_width)?;
        }

        if matches!(
            (&self.three_valued, &self.dual_interval),
            (Some(_), Some(_))
        ) {
            write!(f, " ⊓ ")?;
        }

        if let Some(dual_interval) = &self.dual_interval {
            dual_interval.write(f)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BitvectorField {
    pub bit_width: u32,
    pub element: BitvectorElement,
}

impl Display for BitvectorField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.element.write(f, self.bit_width)
    }
}

use serde::{Deserialize, Serialize};

use crate::abstr::{Abstr, AbstractValue};
use crate::bitvector::bound::{CBound, RBound};
use crate::concr::{ConcreteBitvector, SignedBitvector, UnsignedBitvector};
use crate::misc::{BitvectorBound, Join, MetaEq};
use std::fmt::Display;
use std::hash::Hash;

pub mod combination;
pub mod combined;
pub mod dual_interval;
pub mod eq_domain;
pub mod three_valued;

pub trait BitvectorDomain: Clone + Copy + Hash + Join + MetaEq {
    type Bound: BitvectorBound;
    type General<X: BitvectorBound>: BitvectorDomain<Bound = X>;

    fn bound(&self) -> Self::Bound;

    fn single_value(value: ConcreteBitvector<Self::Bound>) -> Self;
    fn top(bound: Self::Bound) -> Self;
    fn meet(self, other: &Self) -> Option<Self>;

    fn umin(&self) -> UnsignedBitvector<Self::Bound>;
    fn umax(&self) -> UnsignedBitvector<Self::Bound>;
    fn smin(&self) -> SignedBitvector<Self::Bound>;
    fn smax(&self) -> SignedBitvector<Self::Bound>;

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>>;

    fn display(&self) -> BitvectorDisplay;

    fn get_tracker(&self) -> Option<u32> {
        None
    }
    fn assign_tracker(&mut self, _tracker: Option<u32>) {}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DomainDisplay {
    Value(String),
    Tracker(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitvectorDisplay {
    domains: Vec<DomainDisplay>,
}

pub trait CBitvectorDomain: BitvectorDomain {
    type Concrete;

    fn from_concrete_bitvector(value: Self::Concrete) -> Self;
    fn from_runtime_bitvector(value: Self::General<RBound>) -> Self;
    fn as_runtime_bitvector(&self) -> Self::General<RBound>;
}

#[cfg(all(not(feature = "Zdual_interval"), not(feature = "Zeq_domain")))]
pub type Bitvector<B> = three_valued::ThreeValuedBitvector<B>;

#[cfg(all(feature = "Zdual_interval", not(feature = "Zeq_domain")))]
pub type Bitvector<B> = combined::CombinedBitvector<B, combination::TVDICombination<B>>;

#[cfg(all(not(feature = "Zdual_interval"), feature = "Zeq_domain"))]
pub type Bitvector<B> = combined::CombinedBitvector<B, combination::TVEQCombination<B>>;

pub type RBitvector = Bitvector<RBound>;
pub type CBitvector<const W: u32> = Bitvector<CBound<W>>;

pub type PanicBitvector = Bitvector<CBound<32>>;

impl<const W: u32> Abstr<super::concr::Bitvector<CBound<W>>> for Bitvector<CBound<W>> {
    fn from_concrete(value: super::concr::Bitvector<CBound<W>>) -> Self {
        Self::from_concrete_bitvector(value)
    }

    fn from_runtime(value: &AbstractValue) -> Self {
        Self::from_runtime_bitvector(*value.expect_bitvector())
    }

    fn to_runtime(&self) -> AbstractValue {
        AbstractValue::Bitvector(self.as_runtime_bitvector())
    }
}

impl Display for BitvectorDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.domains.is_empty() {
            return write!(f, "⊤");
        }

        let mut first = true;

        for domain in &self.domains {
            if first {
                first = false;
            } else {
                write!(f, " ∩ ")?;
            }

            match domain {
                DomainDisplay::Value(value) => write!(f, "{}", value),
                DomainDisplay::Tracker(tracker) => write!(f, "Eq(#{})", tracker),
            }?;
        }

        Ok(())
    }
}

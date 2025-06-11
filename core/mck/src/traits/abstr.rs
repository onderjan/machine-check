use serde::{Deserialize, Serialize};

use crate::abstr::{format_zeros_ones, PanicResult};
use crate::concr::FullMachine;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use super::misc::MetaEq;

pub trait Abstr<C> {
    #[must_use]
    fn from_concrete(value: C) -> Self;
}

pub trait Input<C: FullMachine>:
    Debug + MetaEq + Hash + Clone + Manipulatable + Abstr<C::Input> + Send + Sync
{
}

pub trait State<C: FullMachine>:
    Debug + MetaEq + Hash + Clone + Manipulatable + Abstr<C::State> + Send + Sync + Phi
{
}

pub trait Machine<C: FullMachine>: Abstr<C>
where
    Self: Sized + Send + Sync,
{
    type Input: Input<C>;
    type State: State<C>;

    #[must_use]
    fn init(&self, input: &Self::Input) -> PanicResult<Self::State>;
    #[must_use]
    fn next(&self, state: &Self::State, input: &Self::Input) -> PanicResult<Self::State>;
}

pub trait Test {
    fn can_be_true(self) -> bool;
    fn can_be_false(self) -> bool;
}

pub trait Phi
where
    Self: std::marker::Sized,
{
    fn phi(self, other: Self) -> Self;
    fn uninit() -> Self;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ThreeValuedFieldValue {
    pub zeros: u64,
    pub ones: u64,
}

impl ThreeValuedFieldValue {
    fn write(&self, f: &mut std::fmt::Formatter<'_>, bit_width: u32) -> std::fmt::Result {
        format_zeros_ones(f, bit_width, self.zeros, self.ones)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DualIntervalFieldValue {
    pub near_min: u64,
    pub near_max: u64,
    pub far_min: u64,
    pub far_max: u64,
}

impl DualIntervalFieldValue {
    fn write(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_interval(f: &mut std::fmt::Formatter<'_>, min: u64, max: u64) -> std::fmt::Result {
            if min == max {
                write!(f, "{}", min)
            } else {
                write!(f, "[{}, {}]", min, max)
            }
        }

        if self.near_min == self.far_min && self.near_max == self.far_max {
            // write just one interval
            write_interval(f, self.near_min, self.near_max)?;
        } else {
            // write the union of two intervals
            write!(f, "(")?;
            write_interval(f, self.near_min, self.near_max)?;
            write!(f, " ∪ ")?;
            write_interval(f, self.far_min, self.far_max)?;
            write!(f, ")")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BitvectorElement {
    pub three_valued: Option<ThreeValuedFieldValue>,
    pub dual_interval: Option<DualIntervalFieldValue>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BitvectorField {
    pub bit_width: u32,
    pub element: BitvectorElement,
}

impl Display for BitvectorField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(three_valued) = &self.element.three_valued {
            three_valued.write(f, self.bit_width)?;
        }

        if matches!(
            (&self.element.three_valued, &self.element.dual_interval),
            (Some(_), Some(_))
        ) {
            write!(f, " ⊓ ")?;
        }

        if let Some(dual_interval) = &self.element.dual_interval {
            dual_interval.write(f)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArrayField {
    pub bit_width: u32,
    pub bit_length: u32,
    pub inner: BTreeMap<u64, BitvectorElement>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Field {
    Bitvector(BitvectorField),
    Array(ArrayField),
}

pub trait ManipField {
    fn index(&self, index: u64) -> Option<&dyn ManipField>;
    fn num_bits(&self) -> Option<u32>;
    fn min_unsigned(&self) -> Option<u64>;
    fn max_unsigned(&self) -> Option<u64>;
    fn min_signed(&self) -> Option<i64>;
    fn max_signed(&self) -> Option<i64>;
    fn description(&self) -> Field;
}
pub trait Manipulatable {
    #[must_use]
    fn get(&self, name: &str) -> Option<&dyn ManipField>;
    #[must_use]
    fn get_mut(&mut self, name: &str) -> Option<&mut dyn ManipField>;
    #[must_use]
    fn field_names() -> Vec<&'static str>;
}

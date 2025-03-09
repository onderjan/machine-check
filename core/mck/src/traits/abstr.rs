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
    Debug + MetaEq + Hash + Clone + Manipulatable + Abstr<C::Input>
{
}

pub trait State<C: FullMachine>:
    Debug + MetaEq + Hash + Clone + Manipulatable + Abstr<C::State>
{
}

pub trait Machine<C: FullMachine>: Abstr<C>
where
    Self: std::marker::Sized,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct BitvectorField {
    pub bit_width: u32,
    pub zeros: u64,
    pub ones: u64,
}

impl Display for BitvectorField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_zeros_ones(f, self.bit_width, self.zeros, self.ones)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ArrayFieldBitvector {
    pub ones: u64,
    pub zeros: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArrayField {
    pub bit_width: u32,
    pub bit_length: u32,
    // TODO: u64 keys
    pub inner: BTreeMap<String, ArrayFieldBitvector>,
}

#[derive(Debug, Serialize, Deserialize)]
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

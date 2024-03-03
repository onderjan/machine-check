use std::fmt::Debug;
use std::hash::Hash;

use super::{abstr, refin};

pub trait Input: Debug + PartialEq + Eq + Hash + Clone {}

pub trait State: Debug + PartialEq + Eq + Hash + Clone {}

pub trait Machine
where
    Self: std::marker::Sized,
{
    type Input;
    type State;

    #[must_use]
    fn init(&self, input: &Self::Input) -> Self::State;
    #[must_use]
    fn next(&self, state: &Self::State, input: &Self::Input) -> Self::State;
}

pub trait Test {
    fn into_bool(self) -> bool;
}

pub trait FullMachine: Machine {
    type Abstr: abstr::Machine<Self>;
    type Refin: refin::Machine<Self>;
}

pub trait IntoMck {
    type Type;
    #[must_use]
    fn into_mck(self) -> Self::Type;
}

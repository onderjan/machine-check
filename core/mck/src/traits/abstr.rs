use crate::abstr::{AbstractValue, PanicResult};
use crate::concr::FullMachine;
use crate::misc::Join;
use std::fmt::Debug;
use std::hash::Hash;

use super::misc::MetaEq;

pub trait Abstr<C> {
    fn from_concrete(value: C) -> Self;
    fn from_runtime(value: &AbstractValue) -> Self;
    fn to_runtime(&self) -> AbstractValue;
}

pub trait Input<C: FullMachine>:
    Debug + MetaEq + Hash + Clone + Abstr<C::Input> + Send + Sync
{
}

impl<C: FullMachine, A: Debug + MetaEq + Hash + Clone + Abstr<C::Input> + Send + Sync> Input<C>
    for A
{
}

pub trait Param<C: FullMachine>:
    Debug + MetaEq + Hash + Clone + Abstr<C::Param> + Send + Sync
{
}

impl<C: FullMachine, A: Debug + MetaEq + Hash + Clone + Abstr<C::Param> + Send + Sync> Param<C>
    for A
{
}

pub trait State<C: FullMachine>:
    Debug + MetaEq + Hash + Clone + Abstr<C::State> + Send + Sync + Phi
{
}

impl<C: FullMachine, A: Debug + MetaEq + Hash + Clone + Abstr<C::State> + Send + Sync + Phi>
    State<C> for A
{
}

pub trait Machine<C: FullMachine>: Abstr<C>
where
    Self: Sized + Send + Sync,
{
    type Input: Input<C>;
    type Param: Param<C>;
    type State: State<C>;

    #[must_use]
    fn init(&self, input: &Self::Input, param: &Self::Param) -> PanicResult<Self::State>;
    #[must_use]
    fn next(
        &self,
        state: &Self::State,
        input: &Self::Input,
        param: &Self::Param,
    ) -> PanicResult<Self::State>;
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
}

impl<T: Join> Phi for T {
    fn phi(self, other: Self) -> Self {
        self.join(&other)
    }
}

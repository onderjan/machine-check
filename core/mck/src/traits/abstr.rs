use crate::abstr::{self, PanicResult};
use crate::concr::FullMachine;
use crate::misc::FieldManipulate;
use std::fmt::Debug;
use std::hash::Hash;

use super::misc::MetaEq;

pub trait Abstr<C> {
    #[must_use]
    fn from_concrete(value: C) -> Self;
}

pub trait Input<C: FullMachine>:
    Debug + MetaEq + Hash + Clone + FieldManipulate<abstr::Bitvector<1>> + Abstr<C::Input>
{
}

pub trait State<C: FullMachine>:
    Debug + MetaEq + Hash + Clone + FieldManipulate<abstr::Bitvector<1>> + Abstr<C::State>
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

use crate::abstr;
use crate::misc::FieldManipulate;
use std::fmt::Debug;
use std::hash::Hash;

use super::misc::MetaEq;

pub trait Input: Debug + MetaEq + Hash + Clone + FieldManipulate<abstr::Bitvector<1>> {}

pub trait State: Debug + MetaEq + Hash + Clone + FieldManipulate<abstr::Bitvector<1>> {}

pub trait Machine<I: Input, S: State>
where
    Self: std::marker::Sized,
{
    #[must_use]
    fn init(&self, input: &I) -> S;
    #[must_use]
    fn next(&self, state: &S, input: &I) -> S;
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

use crate::abstr;
use crate::misc::FieldManipulate;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Input:
    Debug + PartialEq + Eq + Hash + Clone + FieldManipulate<abstr::Bitvector<1>>
{
}

pub trait State:
    Debug + PartialEq + Eq + Hash + Clone + FieldManipulate<abstr::Bitvector<1>>
{
}

pub trait Machine<I: Input, S: State> {
    #[must_use]
    fn init(input: &I) -> S;
    #[must_use]
    fn next(state: &S, input: &I) -> S;
}

pub trait Test {
    fn can_be_true(self) -> bool;
    fn can_be_false(self) -> bool;
}

pub trait Phi
where
    Self: std::marker::Sized,
{
    type Condition;
    // TODO: deduplicate mutable
    fn phi(self, other: Self, condition: Self::Condition) -> Self;
    fn phi_no_cond(self, other: Self) -> Self;

    fn uninit() -> Self;
}

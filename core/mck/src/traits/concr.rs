use std::fmt::Debug;
use std::hash::Hash;

use crate::concr;

use super::misc::FieldManipulate;

pub trait Input:
    Debug + PartialEq + Eq + Hash + Clone + FieldManipulate<concr::Bitvector<1>>
{
}

pub trait State:
    Debug + PartialEq + Eq + Hash + Clone + FieldManipulate<concr::Bitvector<1>>
{
}

pub trait Machine<I: Input, S: State> {
    #[must_use]
    fn init(input: &I) -> S;
    #[must_use]
    fn next(state: &S, input: &I) -> S;
}

pub trait Test {
    fn is_true(self) -> bool;
}

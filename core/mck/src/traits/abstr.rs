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
    fn init(&self, input: &I) -> S;
    fn next(&self, state: &S, input: &I) -> S;
}

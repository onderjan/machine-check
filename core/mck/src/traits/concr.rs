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

pub trait Machine {
    type Input: Input;
    type State: State;

    fn init(input: &Self::Input) -> Self::State;
    fn next(state: &Self::State, input: &Self::Input) -> Self::State;
}

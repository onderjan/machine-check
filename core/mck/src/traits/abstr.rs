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

pub trait Machine {
    type Input: Input;
    type State: State;

    fn init(input: &Self::Input) -> Self::State;
    fn next(state: &Self::State, input: &Self::Input) -> Self::State;
}

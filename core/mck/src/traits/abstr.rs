use crate::{FieldManipulate, ThreeValuedBitvector};
use std::fmt::Debug;
use std::hash::Hash;

pub trait AbstractMachine {
    type Input: AbstractInput;
    type State: AbstractState;

    fn init(input: &Self::Input) -> Self::State;
    fn next(state: &Self::State, input: &Self::Input) -> Self::State;
}

pub trait AbstractState:
    Debug + PartialEq + Eq + Hash + Clone + FieldManipulate<ThreeValuedBitvector<1>>
{
}

pub trait AbstractInput:
    Debug + PartialEq + Eq + Hash + Clone + FieldManipulate<ThreeValuedBitvector<1>>
{
}

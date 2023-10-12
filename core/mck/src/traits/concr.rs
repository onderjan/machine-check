pub use super::forward::*;

use std::fmt::Debug;
use std::hash::Hash;

pub trait Input: Debug + PartialEq + Eq + Hash + Clone {}

pub trait State: Debug + PartialEq + Eq + Hash + Clone {}

pub trait Machine {
    type Input: Input;
    type State: State;

    fn init(input: &Self::Input) -> Self::State;
    fn next(state: &Self::State, input: &Self::Input) -> Self::State;
}

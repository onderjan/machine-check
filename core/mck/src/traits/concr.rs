use std::fmt::Debug;
use std::hash::Hash;

use super::{abstr, misc::PanicMessage, refin};

pub trait Input: Debug + PartialEq + Eq + Hash + Clone + Send + Sync {}

pub trait State: Debug + PartialEq + Eq + Hash + Clone + Send + Sync {}

pub trait Machine
where
    Self: Sized + 'static + Send + Sync,
{
    /**
     * Machine input.
     */
    type Input: Input;
    /**
     * Machine state.
     */
    type State: State;

    /**
     * Creates an initial state from an initial input.
     *
     * This function must be pure.
     */
    #[must_use]
    fn init(&self, input: &Self::Input) -> Self::State;

    /**
     * Creates next state from current state, given the input.
     *
     * This function must be pure.
     */
    #[must_use]
    fn next(&self, state: &Self::State, input: &Self::Input) -> Self::State;
}

pub trait Test {
    fn into_bool(self) -> bool;
}

pub trait FullMachine: Machine + PanicMessage {
    type Abstr: abstr::Machine<Self>;
    type Refin: refin::Machine<Self>;
}

pub trait IntoMck {
    type Type;
    #[must_use]
    fn into_mck(self) -> Self::Type;
}

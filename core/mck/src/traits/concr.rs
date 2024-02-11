use std::fmt::Debug;
use std::hash::Hash;

pub trait Input: Debug + PartialEq + Eq + Hash + Clone {}

pub trait State: Debug + PartialEq + Eq + Hash + Clone {}

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
    fn into_bool(self) -> bool;
}

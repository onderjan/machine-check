use crate::bitvector::refin;
use crate::misc::FieldManipulate;

use std::fmt::Debug;
use std::hash::Hash;

use super::abstr;
use super::misc::Meta;

pub trait MarkSingle {
    fn apply_single_mark(&mut self, offer: &Self) -> bool;
}

pub trait Input:
    Debug
    + PartialEq
    + Eq
    + Hash
    + Clone
    + Meta<<Self as Input>::Abstract>
    + Join
    + Default
    + FieldManipulate<refin::Bitvector<1>>
    + MarkSingle
{
    type Abstract: abstr::Input;

    fn new_unmarked() -> Self {
        Default::default()
    }
}

pub trait State:
    Debug
    + PartialEq
    + Eq
    + Hash
    + Clone
    + Meta<<Self as State>::Abstract>
    + Join
    + Default
    + FieldManipulate<refin::Bitvector<1>>
    + MarkSingle
    + Decay<<Self as State>::Abstract>
{
    type Abstract: abstr::State;
    fn new_unmarked() -> Self {
        Default::default()
    }
}

pub trait Machine<I: Input, S: State> {
    type Abstract: abstr::Machine<<I as Input>::Abstract, <S as State>::Abstract>;

    fn abstr(&self) -> &Self::Abstract;

    fn init(&self, abstr_args: (&<I as Input>::Abstract,), later_mark: S) -> (I,);
    fn next(
        &self,
        abstr_args: (&<S as State>::Abstract, &<I as Input>::Abstract),
        later_mark: S,
    ) -> (S, I);
}

pub trait Markable {
    type Mark;
    fn create_clean_mark(&self) -> Self::Mark;
}

pub trait Join
where
    Self: Sized,
{
    fn apply_join(&mut self, other: Self);
}

pub trait Decay<A>
where
    Self: Sized,
{
    fn force_decay(&self, target: &mut A);
}

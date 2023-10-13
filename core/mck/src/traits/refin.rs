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
    + Meta
    + Join
    + Default
    + FieldManipulate<refin::Bitvector<1>>
    + MarkSingle
{
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
    + Join
    + Default
    + FieldManipulate<refin::Bitvector<1>>
    + MarkSingle
{
    fn new_unmarked() -> Self {
        Default::default()
    }
}

pub trait Machine {
    type Abstract: abstr::Machine;
    type Input: Input;
    type State: State;

    type InputIter: Iterator<Item = <Self::Abstract as abstr::Machine>::Input>;

    fn input_precision_iter(precision: &Self::Input) -> Self::InputIter;

    fn init(
        abstr_args: (&<Self::Abstract as abstr::Machine>::Input,),
        later_mark: Self::State,
    ) -> (Self::Input,);
    fn next(
        abstr_args: (
            &<Self::Abstract as abstr::Machine>::State,
            &<Self::Abstract as abstr::Machine>::Input,
        ),
        later_mark: Self::State,
    ) -> (Self::State, Self::Input);

    fn force_decay(decay: &Self::State, state: &mut <Self::Abstract as abstr::Machine>::State);
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

pub trait Decay
where
    Self: Sized,
{
    type Abstract;
    fn force_decay(&self, target: &mut Self::Abstract);
}

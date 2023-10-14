use crate::bitvector::refin;
use crate::misc::FieldManipulate;

use std::fmt::Debug;
use std::hash::Hash;

use super::abstr;
use super::misc::Meta;

pub trait Refine<A>
where
    Self: Sized,
{
    #[must_use]
    fn apply_refin(&mut self, offer: &Self) -> bool;
    fn apply_join(&mut self, other: &Self);
    fn force_decay(&self, target: &mut A);
}

pub trait Refinable {
    type Refin;
    #[must_use]
    fn clean_refin(&self) -> Self::Refin;
}

pub trait Input:
    Debug
    + PartialEq
    + Eq
    + Hash
    + Clone
    + Default
    + Meta<<Self as Input>::Abstract>
    + Refine<<Self as Input>::Abstract>
    + FieldManipulate<refin::Bitvector<1>>
{
    type Abstract: abstr::Input;
}

pub trait State:
    Debug
    + PartialEq
    + Eq
    + Hash
    + Clone
    + Default
    + Refine<<Self as State>::Abstract>
    + Meta<<Self as State>::Abstract>
    + FieldManipulate<refin::Bitvector<1>>
{
    type Abstract: abstr::State;
}

pub trait Machine<I: Input, S: State> {
    type Abstract: abstr::Machine<<I as Input>::Abstract, <S as State>::Abstract>;

    #[must_use]
    fn init(abstr_args: (&<I as Input>::Abstract,), later_mark: S) -> (I,);
    #[must_use]
    fn next(
        abstr_args: (&<S as State>::Abstract, &<I as Input>::Abstract),
        later_mark: S,
    ) -> (S, I);
}

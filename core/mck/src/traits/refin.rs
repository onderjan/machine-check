use crate::bitvector::refin;
use crate::misc::FieldManipulate;
use crate::refin::Boolean;

use std::fmt::Debug;
use std::hash::Hash;

use super::abstr;
use super::misc::{Meta, MetaEq};

pub trait Refine<A>
where
    Self: Sized,
{
    #[must_use]
    fn apply_refin(&mut self, offer: &Self) -> bool;
    fn apply_join(&mut self, other: &Self);
    fn to_condition(&self) -> Boolean;
    fn force_decay(&self, target: &mut A);
    fn clean() -> Self;
}

pub trait Input:
    Debug
    + MetaEq
    + Hash
    + Clone
    + Meta<<Self as Input>::Abstract>
    + Refine<<Self as Input>::Abstract>
    + FieldManipulate<refin::Bitvector<1>>
{
    type Abstract: abstr::Input;
}

pub trait State:
    Debug
    + MetaEq
    + Clone
    + Refine<<Self as State>::Abstract>
    + Meta<<Self as State>::Abstract>
    + FieldManipulate<refin::Bitvector<1>>
{
    type Abstract: abstr::State;
}

pub trait Machine<I: Input, S: State>
where
    Self: std::marker::Sized,
{
    type Abstract: abstr::Machine<<I as Input>::Abstract, <S as State>::Abstract>;

    #[must_use]
    fn init(abstr_args: (&Self::Abstract, &<I as Input>::Abstract), later_mark: S) -> (Self, I);
    #[must_use]
    fn next(
        abstr_args: (
            &Self::Abstract,
            &<S as State>::Abstract,
            &<I as Input>::Abstract,
        ),
        later_mark: S,
    ) -> (Self, S, I);
}

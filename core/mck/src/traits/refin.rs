use crate::bitvector::refin;
use crate::{Fabricator, FieldManipulate};

use std::fmt::Debug;
use std::hash::Hash;

use super::abstr;

pub trait MarkSingle {
    fn apply_single_mark(&mut self, offer: &Self) -> bool;
}

pub trait Input:
    Debug
    + PartialEq
    + Eq
    + Hash
    + Clone
    + Fabricator
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

pub trait Bitwise
where
    Self: Sized,
{
    type Mark;

    fn not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,);
    fn bitand(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn bitor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn bitxor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}

pub trait TypedEq
where
    Self: Sized,
{
    type MarkEarlier;
    type MarkLater;

    fn typed_eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
}

pub trait TypedCmp
where
    Self: Sized,
{
    type MarkEarlier;
    type MarkLater;

    fn typed_slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    fn typed_ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    fn typed_slte(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    fn typed_ulte(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
}

pub trait Ext<const M: u32> {
    type MarkEarlier;
    type MarkLater;

    fn uext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,);
    fn sext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,);
}

pub trait HwShift
where
    Self: Sized,
{
    type Mark;

    fn logic_shl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn logic_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn arith_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}
pub trait HwArith
where
    Self: Sized,
{
    type Mark;

    fn neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,);

    fn add(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn sub(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn mul(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);

    fn sdiv(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn udiv(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn smod(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn srem(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn urem(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}

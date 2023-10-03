use crate::{AbstractMachine, Fabricator, FieldManipulate, MarkBitvector};

use std::hash::Hash;

pub trait MarkInput:
    PartialEq + Eq + Hash + Clone + Fabricator + Join + Default + FieldManipulate<MarkBitvector<1>>
{
    fn new_unmarked() -> Self {
        Default::default()
    }
}

pub trait MarkState:
    PartialEq + Eq + Hash + Clone + Join + Default + FieldManipulate<MarkBitvector<1>>
{
    fn new_unmarked() -> Self {
        Default::default()
    }
}

pub trait MarkMachine {
    type Abstract: AbstractMachine;
    type Input: MarkInput;
    type State: MarkState;

    type InputIter: Iterator<Item = <Self::Abstract as AbstractMachine>::Input>;

    fn input_precision_iter(precision: &Self::Input) -> Self::InputIter;

    fn init(
        abstr_args: (&<Self::Abstract as AbstractMachine>::Input,),
        later_mark: Self::State,
    ) -> (Self::Input,);
    fn next(
        abstr_args: (
            &<Self::Abstract as AbstractMachine>::State,
            &<Self::Abstract as AbstractMachine>::Input,
        ),
        later_mark: Self::State,
    ) -> (Self::State, Self::Input);
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

pub trait Neg
where
    Self: Sized,
{
    type Mark;

    fn neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,);
}

pub trait Add
where
    Self: Sized,
{
    type Mark;

    fn add(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}
pub trait Sub
where
    Self: Sized,
{
    type Mark;

    fn sub(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}

pub trait Mul
where
    Self: Sized,
{
    type Mark;

    fn mul(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}

pub trait Not
where
    Self: Sized,
{
    type Mark;

    fn not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,);
}

pub trait BitAnd
where
    Self: Sized,
{
    type Mark;

    fn bitand(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}

pub trait BitOr
where
    Self: Sized,
{
    type Mark;

    fn bitor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}

pub trait BitXor
where
    Self: Sized,
{
    type Mark;

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

pub trait MachineExt<const M: u32> {
    type MarkEarlier;
    type MarkLater;

    fn uext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,);
    fn sext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,);
}

pub trait MachineShift
where
    Self: Sized,
{
    type Mark;

    fn sll(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn srl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn sra(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}
pub trait MachineDiv
where
    Self: Sized,
{
    type Mark;
    fn sdiv(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn udiv(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn smod(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn srem(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    fn urem(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}

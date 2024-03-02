use crate::bitvector::refin;
use crate::concr::MachineCheckMachine;
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

pub trait Input<C: MachineCheckMachine>:
    Debug
    + MetaEq
    + Hash
    + Clone
    + Meta<<C::Abstr as abstr::Machine<C>>::Input>
    + Refine<<C::Abstr as abstr::Machine<C>>::Input>
    + FieldManipulate<refin::Bitvector<1>>
{
}

pub trait State<C: MachineCheckMachine>:
    Debug
    + MetaEq
    + Clone
    + Refine<<C::Abstr as abstr::Machine<C>>::State>
    + Meta<<C::Abstr as abstr::Machine<C>>::State>
    + FieldManipulate<refin::Bitvector<1>>
{
}

pub trait Machine<C: MachineCheckMachine>
where
    Self: std::marker::Sized,
{
    type Input: Input<C>;
    type State: State<C>;

    #[must_use]
    fn init(
        abstr_args: (&C::Abstr, &<C::Abstr as abstr::Machine<C>>::Input),
        later_mark: crate::refin::PanicResult<Self::State>,
    ) -> (Self, Self::Input);
    #[allow(clippy::type_complexity)]
    #[must_use]
    fn next(
        abstr_args: (
            &C::Abstr,
            &<C::Abstr as abstr::Machine<C>>::State,
            &<C::Abstr as abstr::Machine<C>>::Input,
        ),
        later_mark: crate::refin::PanicResult<Self::State>,
    ) -> (Self, Self::State, Self::Input);
}

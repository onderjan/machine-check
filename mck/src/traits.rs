use std::hash::Hash;

use crate::ThreeValuedBitvector;

pub trait MachineDiv
where
    Self: Sized,
{
    fn sdiv(self, rhs: Self) -> Self;
    fn udiv(self, rhs: Self) -> Self;
    fn smod(self, rhs: Self) -> Self;
    fn srem(self, rhs: Self) -> Self;
    fn urem(self, rhs: Self) -> Self;
}

pub trait TypedEq {
    type Output;

    fn typed_eq(self, rhs: Self) -> Self::Output;
}

pub trait TypedCmp {
    type Output;

    fn typed_slt(self, rhs: Self) -> Self::Output;
    fn typed_ult(self, rhs: Self) -> Self::Output;
    fn typed_slte(self, rhs: Self) -> Self::Output;
    fn typed_ulte(self, rhs: Self) -> Self::Output;
}

pub trait MachineExt<const M: u32> {
    type Output;

    fn uext(self) -> Self::Output;
    fn sext(self) -> Self::Output;
}

pub trait MachineShift {
    type Output;

    fn sll(self, amount: Self) -> Self::Output;
    fn srl(self, amount: Self) -> Self::Output;
    fn sra(self, amount: Self) -> Self::Output;
}

pub trait Possibility {
    type Possibility;
    fn first_possibility(&self) -> Self::Possibility;
    fn increment_possibility(&self, possibility: &mut Self::Possibility) -> bool;
}

pub trait AbstractMachine {
    type Input: AbstractInput;
    type State: AbstractState;

    fn init(input: &Self::Input) -> Self::State;
    fn next(state: &Self::State, input: &Self::Input) -> Self::State;
}

pub trait AbstractState: PartialEq + Eq + Hash + Clone {
    fn new_unknown() -> Self;
    fn get_safe(&self) -> ThreeValuedBitvector<1>;
}

pub trait AbstractInput: PartialEq + Eq + Hash + Clone {
    fn new_unknown() -> Self;
}

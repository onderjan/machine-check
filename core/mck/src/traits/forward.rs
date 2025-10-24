use crate::{abstr::Phi, boolean::abstr};

pub trait TypedEq {
    type Output;

    #[must_use]
    fn eq(self, rhs: Self) -> Self::Output;

    #[must_use]
    fn ne(self, rhs: Self) -> Self::Output;
}

pub trait TypedCmp {
    type Output;

    #[must_use]
    fn ult(self, rhs: Self) -> Self::Output;
    #[must_use]
    fn slt(self, rhs: Self) -> Self::Output;
    #[must_use]
    fn ule(self, rhs: Self) -> Self::Output;
    #[must_use]
    fn sle(self, rhs: Self) -> Self::Output;
}

pub trait Bitwise
where
    Self: Sized,
{
    #[must_use]
    fn bit_not(self) -> Self;
    #[must_use]
    fn bit_and(self, rhs: Self) -> Self;
    #[must_use]
    fn bit_or(self, rhs: Self) -> Self;
    #[must_use]
    fn bit_xor(self, rhs: Self) -> Self;
}

pub trait HwArith
where
    Self: Sized,
{
    type DivRemResult;

    #[must_use]
    fn arith_neg(self) -> Self;

    #[must_use]
    fn add(self, rhs: Self) -> Self;
    #[must_use]
    fn sub(self, rhs: Self) -> Self;
    #[must_use]
    fn mul(self, rhs: Self) -> Self;

    #[must_use]
    fn udiv(self, rhs: Self) -> Self::DivRemResult;
    #[must_use]
    fn sdiv(self, rhs: Self) -> Self::DivRemResult;

    #[must_use]
    fn urem(self, rhs: Self) -> Self::DivRemResult;
    #[must_use]
    fn srem(self, rhs: Self) -> Self::DivRemResult;
}

pub trait HwShift {
    type Output;

    #[must_use]
    fn logic_shl(self, amount: Self) -> Self::Output;
    #[must_use]
    fn logic_shr(self, amount: Self) -> Self::Output;
    #[must_use]
    fn arith_shr(self, amount: Self) -> Self::Output;
}

pub trait Ext<const M: u32> {
    type Output;

    #[must_use]
    fn uext(self) -> Self::Output;
    #[must_use]
    fn sext(self) -> Self::Output;
}

pub trait RExt {
    #[must_use]
    fn uext(self, width: u32) -> Self;
    #[must_use]
    fn sext(self, width: u32) -> Self;
}

pub enum PhiArg<T: Phi> {
    Taken(T, abstr::Boolean),
    NotTaken(),
}

impl<T: Phi> PhiArg<T> {
    pub fn phi(self, other: Self) -> T {
        match (self, other) {
            (PhiArg::Taken(a, _), PhiArg::Taken(b, _)) => a.phi(b),
            (PhiArg::Taken(a, _), PhiArg::NotTaken()) => a,
            (PhiArg::NotTaken(), PhiArg::Taken(b, _)) => b,
            (PhiArg::NotTaken(), PhiArg::NotTaken()) => panic!("Neither branch taken"),
        }
    }
}

pub trait ReadWrite {
    type Index: Copy;
    type Element: Copy;
    type Deref: Sized;
    fn read(self, index: Self::Index) -> Self::Element;
    fn write(self, index: Self::Index, element: Self::Element) -> Self::Deref;
}

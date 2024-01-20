pub trait TypedEq {
    type Output;

    #[must_use]
    fn typed_eq(self, rhs: Self) -> Self::Output;
}

pub trait TypedCmp {
    type Output;

    #[must_use]
    fn typed_ult(self, rhs: Self) -> Self::Output;
    #[must_use]
    fn typed_slt(self, rhs: Self) -> Self::Output;
    #[must_use]
    fn typed_ulte(self, rhs: Self) -> Self::Output;
    #[must_use]
    fn typed_slte(self, rhs: Self) -> Self::Output;
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
    #[must_use]
    fn arith_neg(self) -> Self;

    #[must_use]
    fn add(self, rhs: Self) -> Self;
    #[must_use]
    fn sub(self, rhs: Self) -> Self;
    #[must_use]
    fn mul(self, rhs: Self) -> Self;

    #[must_use]
    fn udiv(self, rhs: Self) -> Self;
    #[must_use]
    fn sdiv(self, rhs: Self) -> Self;

    #[must_use]
    fn urem(self, rhs: Self) -> Self;
    #[must_use]
    fn srem(self, rhs: Self) -> Self;
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

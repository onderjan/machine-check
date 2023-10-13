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

pub trait Bitwise
where
    Self: Sized,
{
    fn not(self) -> Self;
    fn bitand(self, rhs: Self) -> Self;
    fn bitor(self, rhs: Self) -> Self;
    fn bitxor(self, rhs: Self) -> Self;
}

pub trait HwArith
where
    Self: Sized,
{
    fn neg(self) -> Self;

    fn add(self, rhs: Self) -> Self;
    fn sub(self, rhs: Self) -> Self;
    fn mul(self, rhs: Self) -> Self;

    fn udiv(self, rhs: Self) -> Self;
    fn sdiv(self, rhs: Self) -> Self;

    fn urem(self, rhs: Self) -> Self;
    fn smod(self, rhs: Self) -> Self;
    fn seuc(self, rhs: Self) -> Self;
}

pub trait HwShift {
    type Output;

    fn logic_shl(self, amount: Self) -> Self::Output;
    fn logic_shr(self, amount: Self) -> Self::Output;
    fn arith_shr(self, amount: Self) -> Self::Output;
}

pub trait Ext<const M: u32> {
    type Output;

    fn uext(self) -> Self::Output;
    fn sext(self) -> Self::Output;
}

pub trait TypedEq {
    type Output;

    fn typed_eq(self, rhs: Self) -> Self::Output;
}

pub trait TypedCmp {
    type Output;

    fn typed_sgt(self, rhs: Self) -> Self::Output;
    fn typed_ugt(self, rhs: Self) -> Self::Output;
    fn typed_sgte(self, rhs: Self) -> Self::Output;
    fn typed_ugte(self, rhs: Self) -> Self::Output;

    fn typed_slt(self, rhs: Self) -> Self::Output;
    fn typed_ult(self, rhs: Self) -> Self::Output;
    fn typed_slte(self, rhs: Self) -> Self::Output;
    fn typed_ulte(self, rhs: Self) -> Self::Output;
}

pub trait Uext<const M: u32> {
    type Output;

    fn uext(self) -> Self::Output;
}

pub trait Sext<const M: u32> {
    type Output;

    fn sext(self) -> Self::Output;
}

pub trait Sll {
    type Output;

    fn sll(self, amount: Self) -> Self::Output;
}

pub trait Srl {
    type Output;

    fn srl(self, amount: Self) -> Self::Output;
}

pub trait Sra {
    type Output;

    fn sra(self, amount: Self) -> Self::Output;
}

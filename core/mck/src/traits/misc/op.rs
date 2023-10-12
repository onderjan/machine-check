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

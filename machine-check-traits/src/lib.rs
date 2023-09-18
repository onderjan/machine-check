use num::traits::{PrimInt, WrappingAdd, WrappingSub};

pub trait MachinePrimitive: PrimInt + WrappingAdd + WrappingSub + MachineCastToUnsigned {}

impl<T: PrimInt + WrappingAdd + WrappingSub + MachineCastToUnsigned> MachinePrimitive for T {}

pub trait MachineCastToUnsigned {
    type Unsigned: MachinePrimitive;
    fn cast_to_unsigned(self) -> Self::Unsigned;
}

impl MachineCastToUnsigned for i8 {
    type Unsigned = u8;
    fn cast_to_unsigned(self) -> Self::Unsigned {
        self as <Self as MachineCastToUnsigned>::Unsigned
    }
}

impl MachineCastToUnsigned for u8 {
    type Unsigned = u8;
    fn cast_to_unsigned(self) -> Self::Unsigned {
        self as <Self as MachineCastToUnsigned>::Unsigned
    }
}

pub trait Abstraction {
    type AbstractType;
}

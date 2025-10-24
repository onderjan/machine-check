use crate::{
    concr::{Bitvector, RConcreteBitvector, Test},
    forward::Bitwise,
};
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Boolean(Bitvector<1>);

impl Test for Boolean {
    fn into_bool(self) -> bool {
        self.0.is_nonzero()
    }
}

impl Boolean {
    pub(crate) fn new(value: u64) -> Self {
        Boolean(Bitvector::new(value))
    }
}

impl From<Boolean> for Bitvector<1> {
    fn from(value: Boolean) -> Self {
        value.0
    }
}

impl Bitwise for Boolean {
    fn bit_not(self) -> Self {
        Self(self.0.bit_not())
    }

    fn bit_and(self, rhs: Self) -> Self {
        Self(self.0.bit_and(rhs.0))
    }

    fn bit_or(self, rhs: Self) -> Self {
        Self(self.0.bit_or(rhs.0))
    }

    fn bit_xor(self, rhs: Self) -> Self {
        Self(self.0.bit_xor(rhs.0))
    }
}

// this is used in tests
#[allow(dead_code)]
pub(crate) trait BoolConvert<T> {
    fn bool_from(value: T) -> Self;
    fn bool_into(value: Self) -> T;
}

impl<T> BoolConvert<T> for T {
    fn bool_from(value: T) -> Self {
        value
    }

    fn bool_into(value: Self) -> T {
        value
    }
}

impl BoolConvert<RConcreteBitvector> for super::concr::Boolean {
    fn bool_from(value: RConcreteBitvector) -> Self {
        assert_eq!(value.width(), 1);

        Self(crate::concr::ConcreteBitvector::from_runtime(value))
    }

    fn bool_into(value: Self) -> RConcreteBitvector {
        value.0.to_runtime()
    }
}

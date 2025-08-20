use std::fmt::{Debug, Display};

use machine_check_common::ThreeValued;

use crate::{
    abstr::{BooleanBitvector, Phi, Test},
    forward::Bitwise,
};

#[derive(Clone, Copy, Hash, Default)]
pub struct Boolean(pub(crate) BooleanBitvector);

impl Test for Boolean {
    fn can_be_true(self) -> bool {
        self.0.can_be_true()
    }

    fn can_be_false(self) -> bool {
        self.0.can_be_false()
    }
}

impl Boolean {
    pub fn from_three_valued(value: ThreeValued) -> Self {
        match value {
            ThreeValued::False => Self::from_bools(true, false),
            ThreeValued::True => Self::from_bools(false, true),
            ThreeValued::Unknown => Self::from_bools(true, true),
        }
    }

    pub(crate) fn from_zeros_ones(
        zeros: crate::concr::Bitvector<1>,
        ones: crate::concr::Bitvector<1>,
    ) -> Self {
        Boolean(BooleanBitvector::from_zeros_ones(zeros, ones))
    }

    pub(crate) fn from_bools(can_be_false: bool, can_be_true: bool) -> Self {
        Self::from_zeros_ones(
            crate::concr::Bitvector::new(can_be_false as u64),
            crate::concr::Bitvector::new(can_be_true as u64),
        )
    }
}

impl Debug for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.can_be_false(), self.can_be_true()) {
            (true, true) => write!(f, "both"),
            (true, false) => write!(f, "false"),
            (false, true) => write!(f, "true"),
            (false, false) => panic!("Three-valued Boolean should be true or false"),
        }
    }
}

impl Display for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Phi for Boolean {
    fn phi(self, other: Self) -> Self {
        Boolean(self.0.phi(other.0))
    }

    fn uninit() -> Self {
        Boolean(BooleanBitvector::uninit())
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

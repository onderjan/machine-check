use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use crate::{
    abstr::{BitvectorDomain, RBitvector, Test},
    bitvector::RBound,
    concr::{self, RConcreteBitvector},
    forward::Bitwise,
    misc::{Join, MetaEq},
    three_valued::ThreeValued,
};

#[derive(Clone, Copy, Hash, Serialize, Deserialize)]
pub struct Boolean(ThreeValued);

impl Test for Boolean {
    fn can_be_true(self) -> bool {
        matches!(self.0, ThreeValued::Unknown | ThreeValued::True)
    }

    fn can_be_false(self) -> bool {
        matches!(self.0, ThreeValued::Unknown | ThreeValued::False)
    }
}

impl Boolean {
    pub fn new(value: bool) -> Self {
        Self::from_bools(!value, value)
    }

    pub fn from_concrete(value: concr::Boolean) -> Self {
        Self::new(concr::Test::into_bool(value))
    }

    pub fn from_three_valued(value: ThreeValued) -> Self {
        match value {
            ThreeValued::False => Self::from_bools(true, false),
            ThreeValued::True => Self::from_bools(false, true),
            ThreeValued::Unknown => Self::from_bools(true, true),
        }
    }

    pub fn into_three_valued(self) -> ThreeValued {
        match (self.can_be_false(), self.can_be_true()) {
            (true, true) => ThreeValued::Unknown,
            (true, false) => ThreeValued::False,
            (false, true) => ThreeValued::True,
            (false, false) => unreachable!(),
        }
    }

    pub fn as_runtime_bitvector(self) -> RBitvector {
        let bound = RBound::new(1);

        match self.0 {
            ThreeValued::False => RBitvector::single_value(RConcreteBitvector::new(0, bound)),
            ThreeValued::True => RBitvector::single_value(RConcreteBitvector::new(1, bound)),
            ThreeValued::Unknown => RBitvector::top(bound),
        }
    }

    pub(crate) fn from_bools(can_be_false: bool, can_be_true: bool) -> Self {
        let inner = match (can_be_false, can_be_true) {
            (true, true) => ThreeValued::Unknown,
            (true, false) => ThreeValued::False,
            (false, true) => ThreeValued::True,
            (false, false) => panic!("Three-valued must have some value"),
        };
        Self(inner)
    }

    pub fn is_unknown(&self) -> bool {
        self.0.is_unknown()
    }

    pub fn contains(&self, value: &Boolean) -> bool {
        match self.0 {
            ThreeValued::False => value.0.is_false(),
            ThreeValued::True => value.0.is_true(),
            ThreeValued::Unknown => true,
        }
    }
}

impl Join for Boolean {
    fn join(self, other: &Self) -> Self {
        Self::from_three_valued(
            match (self.into_three_valued(), other.into_three_valued()) {
                (ThreeValued::Unknown, _) | (_, ThreeValued::Unknown) => ThreeValued::Unknown,
                (ThreeValued::False, ThreeValued::True)
                | (ThreeValued::True, ThreeValued::False) => ThreeValued::Unknown,
                (ThreeValued::False, ThreeValued::False) => ThreeValued::False,
                (ThreeValued::True, ThreeValued::True) => ThreeValued::True,
            },
        )
    }
}

impl Debug for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.can_be_false(), self.can_be_true()) {
            (true, true) => write!(f, "unknown"),
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

impl Bitwise for Boolean {
    fn bit_not(self) -> Self {
        Self(!self.0)
    }

    fn bit_and(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }

    fn bit_or(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }

    fn bit_xor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl MetaEq for Boolean {
    fn meta_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

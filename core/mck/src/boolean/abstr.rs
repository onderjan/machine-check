use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use crate::{
    abstr::Test,
    concr::{self},
    forward::Bitwise,
    misc::{Join, MetaEq},
    ParamValuation, ThreeValued,
};

#[derive(Clone, Copy, Hash, Serialize, Deserialize)]
pub struct Boolean(ParamValuation);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BooleanDisplay(ParamValuation);

impl Display for BooleanDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Test for Boolean {
    fn can_be_true(self) -> bool {
        self.0.can_be_true()
    }

    fn can_be_false(self) -> bool {
        matches!(self.0, ParamValuation::Unknown | ParamValuation::False)
    }
}

impl Boolean {
    pub fn new(value: bool) -> Self {
        Self::from_bools(!value, value)
    }

    pub fn from_concrete(value: concr::Boolean) -> Self {
        Self::new(concr::Test::into_bool(value))
    }

    pub fn from_param_valuation(value: ParamValuation) -> Self {
        Self(value)
    }

    pub fn from_three_valued(value: ThreeValued) -> Self {
        Self(ParamValuation::from_three_valued(value))
    }

    pub fn value(&self) -> ParamValuation {
        self.0
    }

    /*pub fn into_three_valued(self) -> Option<ThreeValued> {
        match self.0 {
            ParamValuation::False => Some(ThreeValued::False),
            ParamValuation::True => Some(ThreeValued::True),
            ParamValuation::Unknown => Some(ThreeValued::Unknown),
            ParamValuation::Dependent => None,
        }
    }

    pub fn as_runtime_bitvector(self) -> Option<RBitvector> {
        let three_valued = self.into_three_valued()?;

        let bound = RBound::new(1);

        Some(match three_valued {
            ThreeValued::False => RBitvector::single_value(RConcreteBitvector::new(0, bound)),
            ThreeValued::True => RBitvector::single_value(RConcreteBitvector::new(1, bound)),
            ThreeValued::Unknown => RBitvector::top(bound),
        })
    }*/

    pub(crate) fn from_bools(can_be_false: bool, can_be_true: bool) -> Self {
        let inner = match (can_be_false, can_be_true) {
            (true, true) => ParamValuation::Unknown,
            (true, false) => ParamValuation::False,
            (false, true) => ParamValuation::True,
            (false, false) => panic!("Abstract Boolean must have some possible value"),
        };
        Self(inner)
    }

    pub fn is_unknown(&self) -> bool {
        self.0.is_unknown()
    }

    pub fn can_contain(&self, value: &Boolean) -> bool {
        match self.0 {
            ParamValuation::False => matches!(value.0, ParamValuation::False),
            ParamValuation::True => matches!(value.0, ParamValuation::True),
            ParamValuation::Unknown => true,
            ParamValuation::Dependent => true,
        }
    }

    pub fn display(&self) -> BooleanDisplay {
        BooleanDisplay(self.0)
    }
}

impl Join for Boolean {
    fn join(self, other: &Self) -> Self {
        Self(self.0.join(&other.0))
    }
}

impl Debug for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
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

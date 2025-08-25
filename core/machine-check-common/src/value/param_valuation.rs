use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{BitAnd, BitOr, Not},
};

use serde::{Deserialize, Serialize};

use crate::value::three_valued::ThreeValued;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ParamValuation {
    False,
    True,
    Dependent,
    Unknown,
}

impl Not for ParamValuation {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            ParamValuation::False => ParamValuation::True,
            ParamValuation::True => ParamValuation::False,
            ParamValuation::Dependent => ParamValuation::Dependent,
            ParamValuation::Unknown => ParamValuation::Unknown,
        }
    }
}

impl BitAnd for ParamValuation {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        if self.upward_bitand_ordering(&rhs).is_ge() {
            self
        } else {
            rhs
        }
    }
}

impl BitOr for ParamValuation {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        if self.upward_bitor_ordering(&rhs).is_ge() {
            self
        } else {
            rhs
        }
    }
}

impl ParamValuation {
    pub fn from_bool(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }

    pub fn from_three_valued(three_valued: ThreeValued) -> Self {
        match three_valued {
            ThreeValued::False => ParamValuation::False,
            ThreeValued::True => ParamValuation::True,
            ThreeValued::Unknown => ParamValuation::Unknown,
        }
    }

    pub fn try_into_bool(self) -> Option<bool> {
        match self {
            ParamValuation::False => Some(false),
            ParamValuation::True => Some(true),
            ParamValuation::Dependent | ParamValuation::Unknown => None,
        }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, ParamValuation::Unknown)
    }

    pub fn is_known(&self) -> bool {
        !self.is_unknown()
    }

    pub fn upward_bitand_ordering(self, rhs: &Self) -> Ordering {
        // we order from lowest True (ground value) to greatest False
        // prefer False, then Unknown, then Dependent, then True

        match (self, rhs) {
            (ParamValuation::False, ParamValuation::False) => Ordering::Equal,
            (ParamValuation::False, _) => Ordering::Greater,
            (_, ParamValuation::False) => Ordering::Less,
            (ParamValuation::Unknown, ParamValuation::Unknown) => Ordering::Equal,
            (ParamValuation::Unknown, _) => Ordering::Greater,
            (_, ParamValuation::Unknown) => Ordering::Less,
            (ParamValuation::Dependent, ParamValuation::Dependent) => Ordering::Equal,
            (ParamValuation::Dependent, ParamValuation::True) => Ordering::Greater,
            (ParamValuation::True, ParamValuation::Dependent) => Ordering::Less,
            (ParamValuation::True, ParamValuation::True) => Ordering::Equal,
        }
    }

    pub fn upward_bitor_ordering(self, rhs: &Self) -> Ordering {
        // we order from lowest False (ground value) to greatest True
        // prefer True, then Unknown, then Dependent, then False

        match (self, rhs) {
            (ParamValuation::True, ParamValuation::True) => Ordering::Equal,
            (ParamValuation::True, _) => Ordering::Greater,
            (_, ParamValuation::True) => Ordering::Less,
            (ParamValuation::Unknown, ParamValuation::Unknown) => Ordering::Equal,
            (ParamValuation::Unknown, _) => Ordering::Greater,
            (_, ParamValuation::Unknown) => Ordering::Less,
            (ParamValuation::Dependent, ParamValuation::Dependent) => Ordering::Equal,
            (ParamValuation::Dependent, ParamValuation::False) => Ordering::Greater,
            (ParamValuation::False, ParamValuation::Dependent) => Ordering::Less,
            (ParamValuation::False, ParamValuation::False) => Ordering::Equal,
        }
    }
}

impl Display for ParamValuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ParamValuation::False => "false",
            ParamValuation::True => "true",
            ParamValuation::Dependent => "dependent",
            ParamValuation::Unknown => "unknown",
        };
        write!(f, "{}", str)
    }
}

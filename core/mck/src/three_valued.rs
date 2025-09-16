use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{BitAnd, BitOr, Not},
};

use serde::{Deserialize, Serialize};

/// An extension of a Boolean to three-valued logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreeValued {
    // Known false.
    False,
    // Known true.
    True,
    // Either false or true, but it is unknown which one.
    Unknown,
}

impl PartialOrd for ThreeValued {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ThreeValued {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (ThreeValued::False, ThreeValued::False) => Ordering::Equal,
            (ThreeValued::False, ThreeValued::Unknown) => Ordering::Less,
            (ThreeValued::False, ThreeValued::True) => Ordering::Less,

            (ThreeValued::Unknown, ThreeValued::False) => Ordering::Greater,
            (ThreeValued::Unknown, ThreeValued::Unknown) => Ordering::Equal,
            (ThreeValued::Unknown, ThreeValued::True) => Ordering::Less,

            (ThreeValued::True, ThreeValued::False) => Ordering::Greater,
            (ThreeValued::True, ThreeValued::Unknown) => Ordering::Greater,
            (ThreeValued::True, ThreeValued::True) => Ordering::Equal,
        }
    }
}

impl ThreeValued {
    /// Whether the value is unknown, i.e. neither false nor true.
    pub fn is_unknown(&self) -> bool {
        matches!(self, ThreeValued::Unknown)
    }

    /// Whether the value is known, i.e. false or true.
    pub fn is_known(&self) -> bool {
        !self.is_unknown()
    }

    /// Whether the value is definitely false.
    pub fn is_false(&self) -> bool {
        matches!(self, ThreeValued::False)
    }

    /// Whether the value is definitely true.
    pub fn is_true(&self) -> bool {
        matches!(self, ThreeValued::True)
    }

    pub fn from_bool(value: bool) -> ThreeValued {
        if value {
            ThreeValued::True
        } else {
            ThreeValued::False
        }
    }
}

impl Not for ThreeValued {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            ThreeValued::False => ThreeValued::True,
            ThreeValued::True => ThreeValued::False,
            ThreeValued::Unknown => ThreeValued::Unknown,
        }
    }
}

impl BitAnd for ThreeValued {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ThreeValued::False, _) => ThreeValued::False,
            (_, ThreeValued::False) => ThreeValued::False,
            (ThreeValued::Unknown, _) => ThreeValued::Unknown,
            (_, ThreeValued::Unknown) => ThreeValued::Unknown,
            (ThreeValued::True, ThreeValued::True) => ThreeValued::True,
        }
    }
}

impl BitOr for ThreeValued {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ThreeValued::True, _) => ThreeValued::True,
            (_, ThreeValued::True) => ThreeValued::True,
            (ThreeValued::Unknown, _) => ThreeValued::Unknown,
            (_, ThreeValued::Unknown) => ThreeValued::Unknown,
            (ThreeValued::False, ThreeValued::False) => ThreeValued::False,
        }
    }
}

impl Display for ThreeValued {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ThreeValued::False => "false",
            ThreeValued::True => "true",
            ThreeValued::Unknown => "unknown",
        };
        write!(f, "{}", str)
    }
}

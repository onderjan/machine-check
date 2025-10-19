//! Common structures concerning model-checking.
use std::{collections::VecDeque, fmt::Display};

use serde::{Deserialize, Serialize};

use mck::refin::RefinementValue;

use crate::StateId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnownConclusion {
    False,
    True,
    Dependent,
}

/// Three-valued model-checking result.
///
/// If the result is unknown, the culprit is given.
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum Conclusion {
    Known(KnownConclusion),
    Unknown(Culprit),
    NotCheckable,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct AtomicProperty {
    pub refin_value: RefinementValue,
    pub name: String,
}

/// The culprit of an unknown three-valued model-checking result.
///
/// Comprises of a path and an atomic property which is unknown in the last
/// state of the path.
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct Culprit {
    pub path: VecDeque<StateId>,
    pub atomic_property: AtomicProperty,
}

impl KnownConclusion {
    pub fn try_into_bool(self) -> Option<bool> {
        match self {
            KnownConclusion::False => Some(false),
            KnownConclusion::True => Some(true),
            KnownConclusion::Dependent => None,
        }
    }
}

impl Display for KnownConclusion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            KnownConclusion::False => "false",
            KnownConclusion::True => "true",
            KnownConclusion::Dependent => "dependent",
        };
        write!(f, "{}", str)
    }
}

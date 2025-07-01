//! Common structures concerning model-checking.
use std::{collections::VecDeque, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    property::{AtomicProperty, Property},
    StateId,
};

/// A Computation Tree Logic property prepared for model-checking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PreparedProperty {
    original: Property,
    canonical: Property,
}

impl PreparedProperty {
    /// Turns the CTL proposition into a form suitable for three-valued checking.
    ///
    /// The proposition is converted to the canonical form. This way,
    /// a normal mu-calculus model-checking algorithm can be used.
    pub fn new(original_prop: Property) -> Self {
        // transform proposition to canonical form to be able to verify
        let canonical = original_prop.canonical();
        PreparedProperty {
            original: original_prop,
            canonical,
        }
    }

    pub fn original(&self) -> &Property {
        &self.original
    }

    pub fn canonical(&self) -> &Property {
        &self.canonical
    }
}

impl Display for PreparedProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // the prepared property is just a reformulation of the original
        write!(f, "{}", self.original)
    }
}

/// Three-valued model-checking result.
///
/// If the result is unknown, the culprit is given.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Conclusion {
    Known(bool),
    Unknown(Culprit),
    NotCheckable,
}

/// The culprit of an unknown three-valued model-checking result.
///
/// Comprises of a path and an atomic property which is unknown in the last
/// state of the path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Culprit {
    pub path: VecDeque<StateId>,
    pub atomic_property: AtomicProperty,
}

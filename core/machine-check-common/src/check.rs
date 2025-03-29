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
    prepared: Property,
}

impl PreparedProperty {
    /// Turns the CTL proposition into a form suitable for three-valued checking.
    ///
    /// The proposition must be first converted to Positive Normal Form so that
    /// negations are turned into complementary literals, then converted to
    /// Existential Normal Form. This way, the complementary literals can be used
    /// for optimistic/pessimistic labelling while a normal ENF model-checking
    /// algorithm can be used.
    pub fn new(original_prop: Property) -> Self {
        // transform proposition to positive normal form to move negations to literals
        let prop = original_prop.pnf();
        // transform proposition to existential normal form to be able to verify
        let prop = prop.enf();
        PreparedProperty {
            original: original_prop,
            prepared: prop,
        }
    }

    pub fn original(&self) -> &Property {
        &self.original
    }

    pub fn prepared(&self) -> &Property {
        &self.prepared
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

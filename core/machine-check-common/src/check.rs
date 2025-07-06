//! Common structures concerning model-checking.
use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

pub use crate::property::Property;

use crate::{property::AtomicProperty, StateId};

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

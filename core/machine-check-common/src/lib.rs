#![doc = include_str!("../README.md")]

use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Not};
use thiserror::Error;

pub mod check;
mod node_id;
pub mod property;

pub use node_id::{NodeId, StateId};

/// Execution error that occured during **machine-check** execution.
#[derive(Error, Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub enum ExecError {
    #[error("incomplete verification")]
    Incomplete,
    #[error("field '{0}' of bit type not found")]
    FieldNotFound(String),
    #[error("property '{0}' could not be lexed: {1}")]
    PropertyNotLexable(String, String),
    #[error("property '{0}' could not be parsed: {1}")]
    PropertyNotParseable(String, String),
    #[error("inherent panic")]
    InherentPanic,
    #[error("cannot verify inherent property while assuming it")]
    VerifiedInherentAssumed,
    #[error("GUI error: {0}")]
    GuiError(String),
    #[error("no result")]
    NoResult,
    #[error("{0}")]
    OtherError(String),
}

/// Execution result of **machine-check**.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecResult {
    /// The verification result.
    ///
    /// A non-error result says whether the property holds or not.
    pub result: Result<bool, ExecError>,
    /// Execution statistics.
    pub stats: ExecStats,
}

/// Execution statistics.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ExecStats {
    /// Total number of refinements performed.
    pub num_refinements: usize,
    /// Total number of generated states.
    pub num_generated_states: usize,
    /// Number of states currently in the state space.
    pub num_final_states: usize,
    /// Total number of generated transitions.
    pub num_generated_transitions: usize,
    /// Number of transitions currently in the state space.
    pub num_final_transitions: usize,
    /// If present, the message of the panic causes inherent property violation.
    pub inherent_panic_message: Option<String>,
}

/// An extension of a Boolean to three-valued logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreeValued {
    False,
    True,
    Unknown,
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

/// Signedness of a bit-vector type.
///
/// In **machine-check**, bit-vectors can have no signedness,
/// as the interpretation of its value may completely depend on the operation performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Signedness {
    None,
    Unsigned,
    Signed,
}

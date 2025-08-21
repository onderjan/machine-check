#![doc = include_str!("../README.md")]

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod check;
pub mod iir;
pub mod ir_common;
pub mod property;

mod node_id;
pub use node_id::{NodeId, StateId};

/// Execution error that occured during **machine-check** execution.
#[derive(Error, Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub enum ExecError {
    /// The verification result could not be obtained as the abstraction is too coarse.
    ///  
    /// Currently, this should never happen, as only three-valued abstraction is supported.
    #[error("incomplete verification")]
    Incomplete,
    /// The specified property field was not found in the system.
    #[error("field '{0}' not found")]
    FieldNotFound(String),
    /// The used index was invalid.
    ///
    /// This can happen either due to the field not being indexable
    /// or the index being too high.
    #[error("index {0} is invalid for the field '{1}'")]
    IndexInvalid(u64, String),
    /// The use of an index is required to use the field in an operation.
    ///
    /// This happens because an array type was used where a bit-vector type
    /// was expected.
    #[error("indexing is required for the field '{0}'")]
    IndexRequired(String),
    /// The signedness of the field was required for a comparison, but not estabilished.
    ///
    /// Currently, if needed, the signedness must be forced by `as_unsigned` or `as_signed`,
    /// as field signedness currently does not yet propagate to property verification.
    #[error("signedness of the use of field '{0}' was not estabilished")]
    SignednessNotEstabilished(String),
    /// The specified property is invalid and could not be lexed into tokens.
    #[error("property '{0}' could not be lexed: {1}")]
    PropertyNotLexable(String, String),
    /// The specified property is invalid and could not be parsed.
    #[error("property '{0}' could not be parsed: {1}")]
    PropertyNotParseable(String, String),
    /// Verification of a standard property was requested, but the inherent property does not hold.
    #[error("inherent panic")]
    InherentPanic,
    /// It was requested to verify an inherent property while assuming that it holds.
    #[error("cannot verify inherent property while assuming it")]
    VerifiedInherentAssumed,
    /// The Graphical User Interface emitted an error.
    #[error("GUI error: {0}")]
    GuiError(String),
    /// No verification was requested, so there is no verification result.
    #[error("no result")]
    NoResult,
    /// Other error.
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

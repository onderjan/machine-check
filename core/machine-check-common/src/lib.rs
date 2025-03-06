#![doc = include_str!("../README.md")]

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub enum ExecError {
    #[error("incomplete verification")]
    Incomplete,
    #[error("field '{0}' of bit type not found")]
    FieldNotFound(String),
    #[error("property '{0}' part '{1}' could not be lexed")]
    PropertyNotLexable(String, String),
    #[error("property '{0}' could not be parsed")]
    PropertyNotParseable(String),
    #[error("inherent machine panic: '{0}'")]
    InherentPanic(String),
    #[error("cannot verify inherent property while assuming it")]
    VerifiedInherentAssumed,
    #[error("GUI error: {0}")]
    GuiError(String),
    #[error("no result")]
    NoResult,
    #[error("{0}")]
    OtherError(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecResult {
    pub result: Result<bool, ExecError>,
    pub stats: ExecStats,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ExecStats {
    pub num_refinements: usize,
    pub num_generated_states: usize,
    pub num_final_states: usize,
    pub num_generated_transitions: usize,
    pub num_final_transitions: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreeValued {
    True,
    False,
    Unknown,
}

impl Display for ThreeValued {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ThreeValued::True => "true",
            ThreeValued::False => "false",
            ThreeValued::Unknown => "unknown",
        };
        write!(f, "{}", str)
    }
}

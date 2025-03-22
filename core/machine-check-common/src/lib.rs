#![doc = include_str!("../README.md")]

use std::{fmt::Display, ops::Not};

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    False,
    True,
    Unknown,
}

impl ThreeValued {
    pub fn is_unknown(&self) -> bool {
        matches!(self, ThreeValued::Unknown)
    }

    pub fn is_known(&self) -> bool {
        !self.is_unknown()
    }

    pub fn is_false(&self) -> bool {
        matches!(self, ThreeValued::False)
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Signedness {
    None,
    Unsigned,
    Signed,
}

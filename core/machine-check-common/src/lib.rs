//! # Utility common-type crate for machine-check
//!
//! This crate contains types used for communication between the formal
//! verification tool [machine-check](https://docs.rs/machine-check) and generated
//! programs that call its utility crate [machine-check-exec](
//! https://docs.rs/machine-check-exec).
//!
//! # Usage and Compatibility
//!
//! This crate is a utility crate for [machine-check](https://docs.rs/machine-check)
//! and should not be used on its own. No compatibility guarantees are made.
//!
//! # License
//!
//! This crate is licensed under Apache 2.0 License or MIT License at your discretion.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum ExecError {
    #[error("incomplete verification")]
    Incomplete,
    #[error("field '{0}' of bit type not found")]
    FieldNotFound(String),
    #[error("property '{0}' part '{1}' could not be lexed")]
    PropertyNotLexable(String, String),
    #[error("property '{0}' could not be parsed")]
    PropertyNotParseable(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecResult {
    pub result: Result<bool, ExecError>,
    pub stats: ExecStats,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecStats {
    pub num_states: usize,
    pub num_refinements: usize,
}

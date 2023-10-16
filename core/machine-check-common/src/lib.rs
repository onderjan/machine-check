#![doc = include_str!("../README.md")]

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

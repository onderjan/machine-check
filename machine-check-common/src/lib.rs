// structures used for communication from the machine executable to model-check

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum Error {
    #[error("incomplete verification")]
    Incomplete(Culprit),
    #[error("field '{0}' of bit type not found")]
    FieldNotFound(String),
    #[error("property '{0}' part '{1}' could not be lexed")]
    PropertyNotLexable(String, String),
    #[error("property '{0}' could not be parsed")]
    PropertyNotParseable(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Culprit {
    pub path: VecDeque<usize>,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub num_states: usize,
    pub num_refinements: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecResult {
    pub conclusion: Result<bool, Error>,
    pub info: Info,
}

// structures used for communication from the machine executable to model-check

use std::{collections::VecDeque, num::NonZeroUsize};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum ExecError {
    #[error("incomplete verification")]
    Incomplete(Culprit),
    #[error("field '{0}' of bit type not found")]
    FieldNotFound(String),
    #[error("property '{0}' part '{1}' could not be lexed")]
    PropertyNotLexable(String, String),
    #[error("property '{0}' could not be parsed")]
    PropertyNotParseable(String),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StateId(pub NonZeroUsize);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Culprit {
    pub path: VecDeque<StateId>,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecStats {
    pub num_states: usize,
    pub num_refinements: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecResult {
    pub result: Result<bool, ExecError>,
    pub stats: ExecStats,
}

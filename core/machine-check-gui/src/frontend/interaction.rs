use serde::{Deserialize, Serialize};

use super::snapshot::Snapshot;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepSettings {
    pub num_steps: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    GetContent,
    Reset,
    Step(StepSettings),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub snapshot: Snapshot,
}

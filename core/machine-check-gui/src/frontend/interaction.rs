use machine_check_exec::PreparedProperty;
use serde::{Deserialize, Serialize};

use super::snapshot::Snapshot;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepSettings {
    pub num_steps: Option<u64>,
    pub selected_property: PreparedProperty,
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

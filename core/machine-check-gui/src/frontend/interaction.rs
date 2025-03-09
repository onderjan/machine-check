use machine_check_exec::PreparedProperty;
use serde::{Deserialize, Serialize};

use super::snapshot::Snapshot;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepSettings {
    pub max_refinements: Option<u64>,
    pub selected_property: PreparedProperty,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    GetContent,
    Reset,
    Step(StepSettings),
    AddProperty(PreparedProperty),
    RemoveProperty(usize),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub snapshot: Snapshot,
}

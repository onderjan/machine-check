use machine_check_exec::PreparedProperty;
use serde::{Deserialize, Serialize};
use snapshot::{RootPropertyIndex, Snapshot};

pub mod snapshot;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepSettings {
    pub max_refinements: Option<u64>,
    pub selected_property: PreparedProperty,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    InitialContent,
    GetContent,
    Query,
    Cancel,
    Reset,
    Step(StepSettings),
    AddProperty(PreparedProperty),
    RemoveProperty(RootPropertyIndex),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackendStatus {
    Cancelling,
    Waiting,
    Running,
}

impl BackendStatus {
    pub fn is_waiting(&self) -> bool {
        matches!(self, BackendStatus::Waiting)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendSpaceInfo {
    pub num_states: usize,
    pub num_transitions: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendInfo {
    pub status: BackendStatus,
    pub space_info: BackendSpaceInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    pub info: BackendInfo,
    pub snapshot: Option<Snapshot>,
}

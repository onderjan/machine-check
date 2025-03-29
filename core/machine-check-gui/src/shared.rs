use machine_check_common::check::PreparedProperty;
use serde::{Deserialize, Serialize};
use snapshot::{RootPropertyIndex, Snapshot};

pub mod snapshot;

/// Step settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepSettings {
    pub max_refinements: Option<u64>,
    pub selected_property: PreparedProperty,
}

/// Request for the backend to do something.
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

/// Backend status.
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
    pub num_refinements: usize,
    pub num_states: usize,
    pub num_transitions: usize,
}

/// Information about the backend.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendInfo {
    pub status: BackendStatus,
    pub space_info: BackendSpaceInfo,
}

/// Response from the backend to a request from the frontend.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    pub info: BackendInfo,
    pub snapshot: Option<Snapshot>,
}

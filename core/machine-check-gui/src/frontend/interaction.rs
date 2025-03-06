use serde::{Deserialize, Serialize};

use super::snapshot::Snapshot;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    GetContent,
    Step,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub snapshot: Snapshot,
}

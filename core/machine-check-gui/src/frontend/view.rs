use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreeValuedBool {
    pub zero: bool,
    pub one: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub incoming: BTreeSet<String>,
    pub outgoing: BTreeSet<String>,
    pub panic: Option<ThreeValuedBool>,
    pub fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateSpace {
    // represent the IDs by strings for now
    pub nodes: BTreeMap<String, Node>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateInfo {
    pub field_names: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub exec_name: String,
    pub state_space: StateSpace,
    pub state_info: StateInfo,
}

use std::collections::{BTreeMap, BTreeSet, HashMap};

use machine_check_common::ThreeValued;
use machine_check_exec::{NodeId, PreparedProperty, StateId};
use serde::{Deserialize, Serialize};

use mck::abstr::Field;

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub exec_name: String,
    pub state_space: StateSpace,
    pub state_info: StateInfo,
    pub properties: Vec<PropertySnapshot>,
    pub log: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateInfo {
    pub field_names: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateSpace {
    pub nodes: BTreeMap<NodeId, Node>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub incoming: BTreeSet<NodeId>,
    pub outgoing: BTreeSet<NodeId>,
    pub panic: Option<ThreeValued>,
    pub fields: BTreeMap<String, Field>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertySnapshot {
    pub property: PreparedProperty,
    pub labellings: HashMap<StateId, ThreeValued>,
    pub children: Vec<PropertySnapshot>,
}

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::shared::snapshot::log::Log;
use machine_check_common::{
    check::Conclusion, iir::property::IProperty, ExecError, NodeId, ParamValuation, StateId,
    ThreeValued,
};
use mck::abstr::Field;

pub mod log;

/// A snapshot of the current state of machine-check.
///
/// Provided by the backend to the frontend.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub exec_name: String,
    pub state_space: StateSpace,
    pub state_info: StateInfo,
    pub properties: Vec<PropertySnapshot>,
    pub log: Log,
    pub panic_message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateInfo {
    pub field_names: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateSpace {
    pub nodes: BTreeMap<NodeId, Node>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub incoming: BTreeSet<NodeId>,
    pub outgoing: BTreeSet<NodeId>,
    pub panic: Option<ThreeValued>,
    pub fields: BTreeMap<String, Field>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertySnapshot {
    pub property: IProperty,
    pub conclusion: Result<Conclusion, ExecError>,
    pub labellings: BTreeMap<usize, BTreeMap<StateId, ParamValuation>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyIndex(pub usize);

impl Snapshot {
    pub fn new(
        exec_name: String,
        state_space: StateSpace,
        state_info: StateInfo,
        properties: Vec<PropertySnapshot>,
        log: Log,
        panic_message: Option<String>,
    ) -> Self {
        Self {
            exec_name,
            state_space,
            state_info,
            properties,
            log,
            panic_message,
        }
    }

    pub fn last_property_index(&self) -> Option<PropertyIndex> {
        if !self.properties.is_empty() {
            Some(PropertyIndex(self.properties.len() - 1))
        } else {
            None
        }
    }

    pub fn contains_property(&self, property_index: PropertyIndex) -> bool {
        property_index.0 < self.properties.len()
    }

    pub fn contains_subproperty(
        &self,
        property_index: PropertyIndex,
        subproperty_index: usize,
    ) -> bool {
        self.properties
            .get(property_index.0)
            .is_some_and(|property_snapshot| {
                subproperty_index < property_snapshot.property.num_subproperties()
            })
    }

    pub fn subproperty_labellings(
        &self,
        property_index: PropertyIndex,
        subproperty_index: usize,
    ) -> Option<&BTreeMap<StateId, ParamValuation>> {
        self.properties
            .get(property_index.0)
            .and_then(|property_snapshot| property_snapshot.labellings.get(&subproperty_index))
    }

    pub fn property_snapshot(&self, property_index: PropertyIndex) -> &PropertySnapshot {
        &self.properties[property_index.0]
    }

    pub fn property_snapshots(&self) -> &Vec<PropertySnapshot> {
        &self.properties
    }
}

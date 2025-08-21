use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::shared::snapshot::log::Log;
use machine_check_common::{check::Conclusion, property::Subproperty, ExecError, NodeId, StateId};
use mck::{abstr::Field, three_valued::ThreeValued};

pub mod log;

/// A snapshot of the current state of machine-check.
///
/// Provided by the backend to the frontend.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub exec_name: String,
    pub state_space: StateSpace,
    pub state_info: StateInfo,
    subproperties: Vec<SubpropertySnapshot>,
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
pub struct SubpropertySnapshot {
    pub subproperty: Subproperty,
    pub conclusion: Result<Conclusion, ExecError>,
    pub labellings: BTreeMap<StateId, ThreeValued>,
    pub children: Vec<SubpropertySnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootPropertyIndex(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubpropertyIndex(pub usize);

impl Snapshot {
    pub fn new(
        exec_name: String,
        state_space: StateSpace,
        state_info: StateInfo,
        properties: Vec<SubpropertySnapshot>,
        log: Log,
        panic_message: Option<String>,
    ) -> Self {
        Self {
            exec_name,
            state_space,
            state_info,
            subproperties: properties,
            log,
            panic_message,
        }
    }

    pub fn root_properties_iter(&self) -> impl Iterator<Item = &SubpropertySnapshot> {
        self.subproperties.iter()
    }

    fn num_subproperties(&self) -> usize {
        fn recurse(property: &SubpropertySnapshot, count: &mut usize) {
            *count += 1;
            for child in &property.children {
                recurse(child, count);
            }
        }
        let mut result = 0;
        for property in &self.subproperties {
            recurse(property, &mut result);
        }
        result
    }

    pub fn contains_subindex(&self, index: SubpropertyIndex) -> bool {
        index.0 < self.num_subproperties()
    }

    pub fn select_root_property(&self, index: RootPropertyIndex) -> &SubpropertySnapshot {
        let Some(property) = self.subproperties.get(index.0) else {
            panic!(
                "Property index out of bounds: the len is {} but the index is {}",
                self.subproperties.len(),
                index.0
            );
        };
        property
    }

    pub fn select_subproperty(&self, subindex: SubpropertyIndex) -> &SubpropertySnapshot {
        fn recurse<'a>(
            property: &'a SubpropertySnapshot,
            subindex: usize,
            count: &mut usize,
        ) -> Option<&'a SubpropertySnapshot> {
            if *count == subindex {
                return Some(property);
            }
            *count += 1;
            for child in &property.children {
                if let Some(property) = recurse(child, subindex, count) {
                    return Some(property);
                }
            }
            None
        }
        let mut current_subindex = 0;
        for property in &self.subproperties {
            if let Some(property) = recurse(property, subindex.0, &mut current_subindex) {
                return property;
            }
        }
        panic!(
            "Property subindex out of bounds: the sublen is {} but the subindex is {}",
            current_subindex, subindex.0
        );
    }

    pub fn subindex_to_root_index(&self, subindex: SubpropertyIndex) -> RootPropertyIndex {
        fn recurse(property: &SubpropertySnapshot, subindex: usize, count: &mut usize) -> bool {
            if *count == subindex {
                return true;
            }
            *count += 1;
            for child in &property.children {
                if recurse(child, subindex, count) {
                    return true;
                }
            }
            false
        }
        let mut count = 0;
        for (property_index, property) in self.subproperties.iter().enumerate() {
            if recurse(property, subindex.0, &mut count) {
                return RootPropertyIndex(property_index);
            }
        }
        panic!(
            "Property subindex out of bounds: the sublen is {} but the subindex is {}",
            count, subindex.0
        );
    }

    pub fn root_index_to_subindex(&self, index: RootPropertyIndex) -> SubpropertyIndex {
        fn recurse(property: &SubpropertySnapshot, count: &mut usize) {
            *count += 1;
            for child in &property.children {
                recurse(child, count);
            }
        }
        let mut current_subindex = 0;
        let mut current_index = 0;
        for property in &self.subproperties {
            if current_index == index.0 {
                return SubpropertyIndex(current_subindex);
            }
            recurse(property, &mut current_subindex);
            current_index += 1;
        }
        panic!(
            "Property index out of bounds: the len is {} but the index is {}",
            self.subproperties.len(),
            current_index
        );
    }

    pub fn last_property_subindex(&self) -> Option<SubpropertyIndex> {
        let len = self.subproperties.len();
        if len > 0 {
            Some(self.root_index_to_subindex(RootPropertyIndex(len - 1)))
        } else {
            None
        }
    }
}

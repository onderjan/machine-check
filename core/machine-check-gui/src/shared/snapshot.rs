use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::shared::snapshot::log::Log;
use machine_check_common::ThreeValued;
use machine_check_exec::{NodeId, PreparedProperty, StateId};
use mck::abstr::Field;

pub mod log;

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub exec_name: String,
    pub state_space: StateSpace,
    pub state_info: StateInfo,
    properties: Vec<PropertySnapshot>,
    pub log: Log,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootPropertyIndex(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubpropertyIndex(pub usize);

impl Snapshot {
    pub fn new(
        exec_name: String,
        state_space: StateSpace,
        state_info: StateInfo,
        properties: Vec<PropertySnapshot>,
        log: Log,
    ) -> Self {
        Self {
            exec_name,
            state_space,
            state_info,
            properties,
            log,
        }
    }

    pub fn root_properties_iter(&self) -> impl Iterator<Item = &PropertySnapshot> {
        self.properties.iter()
    }

    fn num_subproperties(&self) -> usize {
        fn recurse(property: &PropertySnapshot, count: &mut usize) {
            *count += 1;
            for child in &property.children {
                recurse(child, count);
            }
        }
        let mut result = 0;
        for property in &self.properties {
            recurse(property, &mut result);
        }
        result
    }

    pub fn contains_subindex(&self, index: SubpropertyIndex) -> bool {
        index.0 < self.num_subproperties()
    }

    pub fn select_root_property(&self, index: RootPropertyIndex) -> &PropertySnapshot {
        let Some(property) = self.properties.get(index.0) else {
            panic!(
                "Property index out of bounds: the len is {} but the index is {}",
                self.properties.len(),
                index.0
            );
        };
        property
    }

    pub fn select_subproperty(&self, subindex: SubpropertyIndex) -> &PropertySnapshot {
        fn recurse<'a>(
            property: &'a PropertySnapshot,
            subindex: usize,
            count: &mut usize,
        ) -> Option<&'a PropertySnapshot> {
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
        for property in &self.properties {
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
        fn recurse(property: &PropertySnapshot, subindex: usize, count: &mut usize) -> bool {
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
        for (property_index, property) in self.properties.iter().enumerate() {
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
        fn recurse(property: &PropertySnapshot, count: &mut usize) {
            *count += 1;
            for child in &property.children {
                recurse(child, count);
            }
        }
        let mut current_subindex = 0;
        let mut current_index = 0;
        for property in &self.properties {
            if current_index == index.0 {
                return SubpropertyIndex(current_subindex);
            }
            recurse(property, &mut current_subindex);
            current_index += 1;
        }
        panic!(
            "Property index out of bounds: the len is {} but the index is {}",
            self.properties.len(),
            current_index
        );
    }

    pub fn last_property_subindex(&self) -> Option<SubpropertyIndex> {
        let len = self.properties.len();
        if len > 0 {
            Some(self.root_index_to_subindex(RootPropertyIndex(len - 1)))
        } else {
            None
        }
    }
}

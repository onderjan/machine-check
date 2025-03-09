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

impl Snapshot {
    pub fn num_subproperties(&self) -> usize {
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

    pub fn select_property_by_subindex(&self, subindex: usize) -> &PropertySnapshot {
        fn recurse<'a>(
            property: &'a PropertySnapshot,
            index: usize,
            count: &mut usize,
        ) -> Option<&'a PropertySnapshot> {
            if *count == index {
                return Some(property);
            }
            *count += 1;
            for child in &property.children {
                if let Some(property) = recurse(child, index, count) {
                    return Some(property);
                }
            }
            None
        }
        let mut count = 0;
        for property in &self.properties {
            if let Some(property) = recurse(property, subindex, &mut count) {
                return property;
            }
        }
        panic!(
            "Property subindex out of bounds: the len is {} but the index is {}",
            count, subindex
        );
    }

    pub fn property_subindex_to_index(&self, subindex: usize) -> usize {
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
        let mut property_index = 0;
        for property in &self.properties {
            if recurse(property, subindex, &mut count) {
                return property_index;
            }
            property_index += 1;
        }
        panic!(
            "Property subindex out of bounds: the len is {} but the index is {}",
            count, property_index
        );
    }
}

use std::collections::BTreeMap;

use machine_check_common::NodeId;

/// Precision configurable by state space nodes.
///
#[derive(Debug)]
pub struct Precision<T: Clone> {
    map: BTreeMap<NodeId, T>,
}

impl<T: Clone> Precision<T> {
    pub fn new() -> Self {
        Precision {
            map: BTreeMap::new(),
        }
    }

    pub fn get(&self, node_id: NodeId, default: &T) -> T {
        match self.map.get(&node_id) {
            Some(input) => input.clone(),
            None => default.clone(),
        }
    }

    pub fn insert(&mut self, node_id: NodeId, value: T) {
        self.map.insert(node_id, value);
    }
}

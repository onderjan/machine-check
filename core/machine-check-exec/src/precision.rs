use std::collections::BTreeMap;

use machine_check_common::NodeId;

/// Precision configurable by state space nodes.
///
#[derive(Debug)]
pub struct Precision<T: Clone> {
    map: BTreeMap<NodeId, T>,
    /*
    /// Input precision. Determines which inputs are qualified to be used.
    input: BTreeMap<NodeId, <M::Refin as refin::Machine<M>>::Input>,
    /// Step decay. Determines which parts of the state decay after using the step function.
    decay: BTreeMap<NodeId, refin::PanicResult<<M::Refin as refin::Machine<M>>::State>>,*/
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

    pub fn get_mut(&mut self, node_id: NodeId, default: &T) -> &mut T {
        self.map.entry(node_id).or_insert_with(|| default.clone())
    }
}

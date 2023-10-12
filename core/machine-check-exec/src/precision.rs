use std::collections::BTreeMap;

use mck::refin::Input;
use mck::refin::Machine;
use mck::refin::State;

use crate::space::NodeId;

pub struct Precision<M: Machine> {
    input: BTreeMap<NodeId, M::Input>,
    decay: BTreeMap<NodeId, M::State>,
}

impl<M: Machine> Precision<M> {
    pub fn new() -> Self {
        Precision {
            input: BTreeMap::new(),
            decay: BTreeMap::new(),
        }
    }

    pub fn get_input(&self, node_id: NodeId) -> M::Input {
        match self.input.get(&node_id) {
            Some(input) => input.clone(),
            None => M::Input::new_unmarked(),
        }
    }

    pub fn mut_input(&mut self, node_id: NodeId) -> &mut M::Input {
        self.input
            .entry(node_id)
            .or_insert_with(M::Input::new_unmarked)
    }

    pub fn get_decay(&self, node_id: NodeId) -> M::State {
        match self.decay.get(&node_id) {
            Some(decay) => decay.clone(),
            None => M::State::new_unmarked(),
        }
    }

    pub fn mut_decay(&mut self, node_id: NodeId) -> &mut M::State {
        self.decay
            .entry(node_id)
            .or_insert_with(M::State::new_unmarked)
    }

    pub fn retain_indices<F>(&mut self, predicate: F)
    where
        F: Fn(NodeId) -> bool,
    {
        self.input.retain(|k, _| predicate(*k));
        self.decay.retain(|k, _| predicate(*k));
    }
}

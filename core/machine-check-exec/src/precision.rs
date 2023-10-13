use std::collections::BTreeMap;

use mck::refin::Input;
use mck::refin::State;

use crate::space::NodeId;

pub struct Precision<I: Input, S: State> {
    input: BTreeMap<NodeId, I>,
    decay: BTreeMap<NodeId, S>,
}

impl<I: Input, S: State> Precision<I, S> {
    pub fn new() -> Self {
        Precision {
            input: BTreeMap::new(),
            decay: BTreeMap::new(),
        }
    }

    pub fn get_input(&self, node_id: NodeId) -> I {
        match self.input.get(&node_id) {
            Some(input) => input.clone(),
            None => I::new_unmarked(),
        }
    }

    pub fn mut_input(&mut self, node_id: NodeId) -> &mut I {
        self.input.entry(node_id).or_insert_with(I::new_unmarked)
    }

    pub fn get_decay(&self, node_id: NodeId) -> S {
        match self.decay.get(&node_id) {
            Some(decay) => decay.clone(),
            None => S::new_unmarked(),
        }
    }

    pub fn mut_decay(&mut self, node_id: NodeId) -> &mut S {
        self.decay.entry(node_id).or_insert_with(S::new_unmarked)
    }

    pub fn retain_indices<F>(&mut self, predicate: F)
    where
        F: Fn(NodeId) -> bool,
    {
        self.input.retain(|k, _| predicate(*k));
        self.decay.retain(|k, _| predicate(*k));
    }
}

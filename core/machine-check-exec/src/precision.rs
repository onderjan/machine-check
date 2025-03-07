use std::collections::BTreeMap;

use mck::{
    concr::FullMachine,
    refin::{self, Refine},
};

use crate::space::NodeId;

/// Current abstract state space precision.
///
#[derive(Debug)]
pub struct Precision<M: FullMachine> {
    /// Input precision. Determines which inputs are qualified to be used.
    input: BTreeMap<NodeId, <M::Refin as refin::Machine<M>>::Input>,
    /// Step decay. Determines which parts of the state decay after using the step function.
    decay: BTreeMap<NodeId, refin::PanicResult<<M::Refin as refin::Machine<M>>::State>>,
    /// Whether each input should immediately cover only a single concrete input.
    naive_inputs: bool,
}

impl<M: FullMachine> Precision<M> {
    pub fn new(naive_inputs: bool) -> Self {
        Precision {
            input: BTreeMap::new(),
            decay: BTreeMap::new(),
            naive_inputs,
        }
    }

    pub fn get_input(&self, node_id: NodeId) -> <M::Refin as refin::Machine<M>>::Input {
        match self.input.get(&node_id) {
            Some(input) => input.clone(),
            None => {
                if self.naive_inputs {
                    Refine::dirty()
                } else {
                    Refine::clean()
                }
            }
        }
    }

    pub fn mut_input(&mut self, node_id: NodeId) -> &mut <M::Refin as refin::Machine<M>>::Input {
        self.input
            .entry(node_id)
            .or_insert_with(if self.naive_inputs {
                Refine::dirty
            } else {
                Refine::clean
            })
    }

    pub fn get_decay(
        &self,
        node_id: NodeId,
    ) -> refin::PanicResult<<M::Refin as refin::Machine<M>>::State> {
        match self.decay.get(&node_id) {
            Some(decay) => decay.clone(),
            None => Refine::clean(),
        }
    }

    pub fn mut_decay(
        &mut self,
        node_id: NodeId,
    ) -> &mut refin::PanicResult<<M::Refin as refin::Machine<M>>::State> {
        self.decay.entry(node_id).or_insert_with(Refine::clean)
    }

    pub fn retain_indices<F>(&mut self, predicate: F)
    where
        F: Fn(NodeId) -> bool,
    {
        self.input.retain(|k, _| predicate(*k));
        self.decay.retain(|k, _| predicate(*k));
    }

    pub fn input_precision(&self) -> &BTreeMap<NodeId, <M::Refin as refin::Machine<M>>::Input> {
        &self.input
    }

    pub fn naive_inputs(&self) -> bool {
        self.naive_inputs
    }
}

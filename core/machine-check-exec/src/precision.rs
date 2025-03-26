use std::collections::BTreeMap;

use machine_check_common::NodeId;
use mck::{
    concr::FullMachine,
    refin::{self, Refine},
};

/// Current abstract state space precision.
///
#[derive(Debug)]
pub struct Precision<M: FullMachine> {
    /// Input precision. Determines which inputs are qualified to be used.
    input: BTreeMap<NodeId, <M::Refin as refin::Machine<M>>::Input>,
    /// Step decay. Determines which parts of the state decay after using the step function.
    decay: BTreeMap<NodeId, refin::PanicResult<<M::Refin as refin::Machine<M>>::State>>,
}

impl<M: FullMachine> Precision<M> {
    pub fn new() -> Self {
        Precision {
            input: BTreeMap::new(),
            decay: BTreeMap::new(),
        }
    }

    pub fn get_input(
        &self,
        node_id: NodeId,
        default: &<M::Refin as refin::Machine<M>>::Input,
    ) -> <M::Refin as refin::Machine<M>>::Input {
        match self.input.get(&node_id) {
            Some(input) => input.clone(),
            None => default.clone(),
            /*{
            if self.naive_inputs {
                Refine::dirty()
            } else {
                Refine::clean()
            }*/
        }
    }

    pub fn mut_input(
        &mut self,
        node_id: NodeId,
        default: &<M::Refin as refin::Machine<M>>::Input,
    ) -> &mut <M::Refin as refin::Machine<M>>::Input {
        self.input.entry(node_id).or_insert_with(
            || default.clone(), /*if self.naive_inputs {
                                    Refine::dirty
                                } else {
                                    Refine::clean
                                }*/
        )
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

    pub fn input_precision(&self) -> &BTreeMap<NodeId, <M::Refin as refin::Machine<M>>::Input> {
        &self.input
    }
}

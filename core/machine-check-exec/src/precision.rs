use std::collections::BTreeMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use machine_check_common::{NodeId, StateId};
use mck::abstr::Phi;
use mck::concr::FullMachine;
use mck::misc::MetaEq;

use crate::space::StateSpace;

/// Precision configurable by state space nodes.
///
#[derive(Debug)]
pub struct Precision<A, R: Debug + Clone + mck::refin::Refine<A>> {
    map: BTreeMap<NodeId, R>,
    phantom: PhantomData<A>,
}

impl<A, R: Debug + Clone + mck::refin::Refine<A>> Precision<A, R> {
    pub fn new() -> Self {
        Precision {
            map: BTreeMap::new(),
            phantom: PhantomData,
        }
    }

    pub fn get<M: FullMachine>(
        &self,
        state_space: &StateSpace<M>,
        node_id: NodeId,
        default: &R,
    ) -> R {
        let mut node_precision = match self.map.get(&node_id) {
            Some(input) => input.clone(),
            None => default.clone(),
        };

        let Ok(state_id) = StateId::try_from(node_id) else {
            // root node, just get the value normally
            return node_precision;
        };

        let node_data = state_space.state_data(state_id);

        // Ensure that the precision is monotone with respect to other states.
        // TODO: use a faster algorithm than O(n) iteration, although the number of precise states should be usually small

        for (map_id, map_precision) in self.map.iter() {
            let Ok(map_id) = StateId::try_from(*map_id) else {
                continue;
            };

            let map_data = state_space.state_data(map_id);

            // if the map data covers the node data, we must join the precision
            let phi_result = Phi::phi(node_data.clone(), map_data.clone());

            if phi_result.meta_eq(map_data) {
                // join the precision
                mck::refin::Refine::apply_join(&mut node_precision, map_precision);
            }
        }

        node_precision
    }

    pub fn used_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.map.keys().copied()
    }

    pub fn insert(&mut self, node_id: NodeId, value: R) {
        self.map.insert(node_id, value);
    }
}

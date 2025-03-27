use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Duration;

use machine_check_common::{NodeId, StateId};
use mck::abstr::Phi;
use mck::concr::FullMachine;
use mck::misc::MetaEq;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::GraphMap;
use petgraph::Directed;

use crate::space::StateSpace;

/// Precision configurable by state space nodes.
///
#[derive(Debug)]
pub struct Precision<A, R: Debug + Clone + mck::refin::Refine<A>> {
    map: BTreeMap<NodeId, R>,
    phantom: PhantomData<A>,
    get_elapsed: RefCell<Duration>,
    insert_elapsed: RefCell<Duration>,
    ordering_root: Option<StateId>,
    ordering_graph: GraphMap<StateId, (), Directed>,
}

impl<A, R: Debug + Clone + mck::refin::Refine<A>> Precision<A, R> {
    pub fn new() -> Self {
        Precision {
            map: BTreeMap::new(),
            phantom: PhantomData,
            get_elapsed: RefCell::new(Duration::ZERO),
            insert_elapsed: RefCell::new(Duration::ZERO),
            ordering_root: None,
            ordering_graph: GraphMap::new(),
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

        let start = std::time::Instant::now();

        /*for (map_id, map_precision) in self.map.iter() {
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
        }*/

        // search the graph
        if let Some(ordering_root) = self.ordering_root {
            let mut stack = Vec::new();
            stack.push(ordering_root);
            let mut opened = HashSet::new();
            let mut important = HashSet::new();

            while let Some(stack_id) = stack.pop() {
                let stack_data = state_space.state_data(stack_id);
                let phi = Phi::phi(node_data.clone(), stack_data.clone());
                if stack_data.meta_eq(&phi) {
                    important.insert(stack_id);
                    for child_id in self.ordering_graph.neighbors(stack_id) {
                        if !opened.contains(&child_id) {
                            opened.insert(child_id);
                            stack.push(stack_id);
                        }
                    }
                }
            }

            for important_id in important {
                let important_precision = self
                    .map
                    .get(&NodeId::from(important_id))
                    .expect("Precision should contain state in ordering graph");
                mck::refin::Refine::apply_join(&mut node_precision, important_precision);
            }
        }

        let elapsed = start.elapsed();
        let mut get_elapsed = self.get_elapsed.borrow_mut();
        *get_elapsed += elapsed;

        if (self.map.len() % 10) == 1 {
            log::debug!(
                "GET: Map size: {}, Duration monotone application elapsed now: {:?}, total: {:?}",
                self.map.len(),
                elapsed,
                *get_elapsed
            );
        }

        node_precision
    }

    pub fn insert<M: FullMachine>(
        &mut self,
        state_space: &mut StateSpace<M>,
        node_id: NodeId,
        value: R,
        default: &R,
    ) {
        let start = std::time::Instant::now();

        // if the node already exists, we do not need to change the graph

        if self.map.insert(node_id, value).is_some() {
            return;
        }

        // if we are changing the root node, we also do not need to change the graph
        let Ok(state_id) = StateId::try_from(node_id) else {
            return;
        };

        let a_data = state_space.state_data(state_id);

        // if we have no ordering root, just set it and add it to the graph
        let Some(ordering_root) = self.ordering_root else {
            self.ordering_root = Some(state_id);
            self.ordering_graph.add_node(state_id);
            return;
        };

        // depth first search for a suitable candidate
        let mut stack = Vec::new();
        stack.push(ordering_root);
        while let Some(stack_state_id) = stack.pop() {
            // look at whether it has a new join
            let b_data = state_space.state_data(stack_state_id);

            let phi_result = Phi::phi(a_data.clone(), b_data.clone());

            if !phi_result.meta_eq(a_data) && !phi_result.meta_eq(b_data) {
                // add this join to the map with the default value
                let phi_id = state_space.state_id(phi_result);

                self.map.insert(phi_id.into(), default.clone());
                // update the ordering root if necessary
                if stack_state_id == ordering_root {
                    self.ordering_root = Some(phi_id);
                }
                // break loop
                break;
            }

            // look at the descendants in the ordering graph
            let ordering_descendants =
                BTreeSet::from_iter(self.ordering_graph.neighbors(stack_state_id));
            for descendant in ordering_descendants.into_iter() {
                stack.push(descendant);
            }
        }

        // regenerate the ordering graph

        self.ordering_graph = GraphMap::<StateId, (), Directed>::new();

        let states = BTreeSet::from_iter(
            self.map
                .keys()
                .filter_map(|node_id| StateId::try_from(*node_id).ok()),
        );

        for a in states.iter().cloned() {
            for b in states.iter().cloned() {
                let a_data = state_space.state_data(a);
                let b_data = state_space.state_data(b);

                let phi_result = Phi::phi(a_data.clone(), b_data.clone());

                if phi_result.meta_eq(a_data) {
                    self.ordering_graph.add_edge(a, b, ());
                }
            }
            // reflexive reduction
            self.ordering_graph.remove_edge(a, a);
        }

        // transitive reduction
        let ordering_graph_nodes = BTreeSet::from_iter(self.ordering_graph.nodes());

        for a in ordering_graph_nodes.iter().cloned() {
            for b in ordering_graph_nodes.iter().cloned() {
                for c in ordering_graph_nodes.iter().cloned() {
                    if self.ordering_graph.contains_edge(a, b)
                        && self.ordering_graph.contains_edge(b, c)
                    {
                        self.ordering_graph.remove_edge(a, c);
                    }
                }
            }
        }

        //log::debug!("State ordering: {:?}", self.ordering_graph);

        let elapsed = start.elapsed();
        let mut insert_elapsed = self.insert_elapsed.borrow_mut();
        *insert_elapsed += elapsed;
        log::debug!(
            "INSERT: Map size: {}, Duration monotone application elapsed now: {:?}, total: {:?}",
            self.map.len(),
            elapsed,
            *insert_elapsed
        );
        /*println!(
            "{:?}",
            Dot::with_config(&self.ordering_graph, &[Config::EdgeNoLabel])
        );*/
    }
}

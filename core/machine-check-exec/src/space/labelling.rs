use std::collections::BTreeSet;

use crate::proposition::InequalityType;
use crate::proposition::Literal;

use super::{Space, StateId};
use mck::abstr::ManipField;
use mck::abstr::Manipulatable;
use mck::concr::FullMachine;
use petgraph::graphmap::GraphMap;
use petgraph::Directed;

impl<M: FullMachine> Space<M> {
    /// Returns an iterator of state ids labelled by a given literal with an optimistic/pessimistic interpretation.
    pub fn labelled_iter<'a>(
        &'a self,
        literal: &'a Literal,
        optimistic: bool,
    ) -> impl Iterator<Item = Result<StateId, ()>> + 'a {
        self.state_map.iter().filter_map(move |(state_id, state)| {
            let name = literal.name();
            let manip_field = if name == "__panic" {
                let manip_field: &dyn ManipField = &state.0.panic;
                manip_field
            } else {
                match state.0.result.get(name) {
                    Some(manip_field) => manip_field,
                    None => return Some(Err(())),
                }
            };
            let manip_field = if let Some(index) = literal.index() {
                let Some(indexed_manip_field) = manip_field.index(index) else {
                    return Some(Err(()));
                };
                indexed_manip_field
            } else {
                manip_field
            };

            let (Some(min_unsigned), Some(max_unsigned)) =
                (manip_field.min_unsigned(), manip_field.max_unsigned())
            else {
                return Some(Err(()));
            };
            let right_unsigned = literal.right_number_unsigned();
            let comparison_result = match literal.comparison_type() {
                crate::proposition::ComparisonType::Eq => {
                    if min_unsigned == max_unsigned {
                        Some(min_unsigned == right_unsigned)
                    } else {
                        None
                    }
                }
                crate::proposition::ComparisonType::Neq => {
                    if min_unsigned == max_unsigned {
                        Some(min_unsigned != right_unsigned)
                    } else {
                        None
                    }
                }
                crate::proposition::ComparisonType::Unsigned(inequality_type) => {
                    Self::resolve_inequality(
                        inequality_type,
                        min_unsigned,
                        max_unsigned,
                        right_unsigned,
                    )
                }
                crate::proposition::ComparisonType::Signed(inequality_type) => {
                    let (Some(min_signed), Some(max_signed)) =
                        (manip_field.min_signed(), manip_field.max_signed())
                    else {
                        return Some(Err(()));
                    };
                    let right_signed = literal.right_number_signed();
                    Self::resolve_inequality(inequality_type, min_signed, max_signed, right_signed)
                }
            };

            let labelled = match comparison_result {
                Some(comparison_result) => {
                    // negate if necessary
                    if literal.is_complementary() {
                        !comparison_result
                    } else {
                        comparison_result
                    }
                }
                None => {
                    // never negate here, just consider if it is optimistic
                    // see https://patricegodefroid.github.io/public_psfiles/marktoberdorf2013.pdf
                    optimistic
                }
            };
            if labelled {
                Some(Ok(*state_id))
            } else {
                None
            }
        })
    }

    /// Returns state ids in nontrivial strongly connected components.
    ///
    /// Used for EG[phi] labelling computation.
    pub fn labelled_nontrivial_scc_indices(
        &self,
        labelled: &BTreeSet<StateId>,
    ) -> BTreeSet<StateId> {
        // construct a new state graph that only contains labelled vertices and transitions between them
        let mut labelled_graph = GraphMap::<StateId, (), Directed>::new();

        for labelled_id in labelled.iter().cloned() {
            labelled_graph.add_node(labelled_id);
            for direct_successor_id in self.direct_successor_iter(labelled_id.into()) {
                labelled_graph.add_edge(labelled_id, direct_successor_id, ());
            }
        }

        // get out the indices in trivial SCC
        let sccs = petgraph::algo::kosaraju_scc(&labelled_graph);
        let mut result = BTreeSet::new();
        for scc in sccs {
            if scc.len() == 1 {
                let state_id = scc[0];
                if !labelled_graph.contains_edge(state_id, state_id) {
                    // trivial SCC, do not add to result, but continue over other SCCs
                    continue;
                }
            }
            // we only labelled states, so they must be
            result.extend(scc.into_iter());
        }
        result
    }

    fn resolve_inequality<T: Ord>(
        inequality_type: &InequalityType,
        min_left: T,
        max_left: T,
        right: T,
    ) -> Option<bool> {
        // TODO: resolve inequality using mck types
        match inequality_type {
            InequalityType::Lt => {
                if max_left < right {
                    Some(true)
                } else if min_left >= right {
                    Some(false)
                } else {
                    None
                }
            }
            InequalityType::Le => {
                if max_left <= right {
                    Some(true)
                } else if min_left > right {
                    Some(false)
                } else {
                    None
                }
            }
            InequalityType::Gt => {
                if max_left > right {
                    Some(true)
                } else if min_left <= right {
                    Some(false)
                } else {
                    None
                }
            }
            InequalityType::Ge => {
                if max_left <= right {
                    Some(true)
                } else if min_left > right {
                    Some(false)
                } else {
                    None
                }
            }
        }
    }
}

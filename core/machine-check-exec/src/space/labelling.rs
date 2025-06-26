use std::collections::BTreeSet;

use super::StateId;
use super::StateSpace;
use machine_check_common::property::AtomicProperty;
use machine_check_common::property::ComparisonType;
use machine_check_common::ExecError;
use machine_check_common::Signedness;
use machine_check_common::ThreeValued;
use mck::abstr::{ManipField, Manipulatable};
use mck::concr::FullMachine;
use petgraph::graphmap::GraphMap;
use petgraph::Directed;

impl<M: FullMachine> StateSpace<M> {
    /// Returns an iterator of state ids labelled by a given literal with an optimistic/pessimistic interpretation.
    pub fn labelled_iter<'a>(
        &'a self,
        atomic_property: &'a AtomicProperty,
    ) -> impl Iterator<Item = Result<(StateId, ThreeValued), ExecError>> + 'a {
        self.nodes().filter_map(move |node_id| {
            let Ok(state_id) = StateId::try_from(node_id) else {
                return None;
            };
            let state = self.state_data(state_id);

            let left = atomic_property.left();
            let left_name = left.name();
            let manip_field = if left_name == "__panic" {
                let manip_field: &dyn ManipField = &state.panic;
                manip_field
            } else {
                match state.result.get(left_name) {
                    Some(manip_field) => manip_field,
                    None => return Some(Err(ExecError::FieldNotFound(String::from(left_name)))),
                }
            };
            let manip_field = if let Some(index) = left.index() {
                let Some(indexed_manip_field) = manip_field.index(index) else {
                    return Some(Err(ExecError::IndexInvalid(index, String::from(left_name))));
                };
                indexed_manip_field
            } else {
                manip_field
            };

            let (Some(min_unsigned), Some(max_unsigned)) =
                (manip_field.min_unsigned(), manip_field.max_unsigned())
            else {
                return Some(Err(ExecError::IndexRequired(String::from(left_name))));
            };
            let right_unsigned = atomic_property.right_number_unsigned();
            let mut comparison_result = match atomic_property.comparison_type() {
                ComparisonType::Eq => {
                    if min_unsigned == max_unsigned {
                        ThreeValued::from_bool(min_unsigned == right_unsigned)
                    } else {
                        ThreeValued::Unknown
                    }
                }
                ComparisonType::Ne => {
                    if min_unsigned == max_unsigned {
                        ThreeValued::from_bool(min_unsigned != right_unsigned)
                    } else {
                        ThreeValued::Unknown
                    }
                }
                comparison_type => {
                    match left.forced_signedness() {
                        Signedness::None => {
                            // signedness not specified
                            // TODO: try to estabilish signedness by using the types in this case
                            return Some(Err(ExecError::SignednessNotEstabilished(
                                left.to_string(),
                            )));
                        }
                        Signedness::Unsigned => Self::resolve_inequality(
                            comparison_type,
                            min_unsigned,
                            max_unsigned,
                            right_unsigned,
                        ),
                        Signedness::Signed => {
                            let (Some(min_signed), Some(max_signed)) =
                                (manip_field.min_signed(), manip_field.max_signed())
                            else {
                                return Some(Err(ExecError::IndexRequired(String::from(
                                    left_name,
                                ))));
                            };
                            let right_signed = atomic_property.right_number_signed();
                            Self::resolve_inequality(
                                comparison_type,
                                min_signed,
                                max_signed,
                                right_signed,
                            )
                        }
                    }
                }
            };

            // handle the complementary values
            if atomic_property.is_complementary() {
                comparison_result = !comparison_result;
            }

            Some(Ok((state_id, comparison_result)))
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
            for direct_successor_id in self.graph.direct_successor_iter(labelled_id.into()) {
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
        inequality_type: &ComparisonType,
        min_left: T,
        max_left: T,
        right: T,
    ) -> ThreeValued {
        // TODO: resolve inequality using mck types
        match inequality_type {
            ComparisonType::Lt => {
                if max_left < right {
                    ThreeValued::True
                } else if min_left >= right {
                    ThreeValued::False
                } else {
                    ThreeValued::Unknown
                }
            }
            ComparisonType::Le => {
                if max_left <= right {
                    ThreeValued::True
                } else if min_left > right {
                    ThreeValued::False
                } else {
                    ThreeValued::Unknown
                }
            }
            ComparisonType::Gt => {
                if min_left > right {
                    ThreeValued::True
                } else if max_left <= right {
                    ThreeValued::False
                } else {
                    ThreeValued::Unknown
                }
            }
            ComparisonType::Ge => {
                if min_left >= right {
                    ThreeValued::True
                } else if max_left < right {
                    ThreeValued::False
                } else {
                    ThreeValued::Unknown
                }
            }
            _ => panic!("Inequality comparison should be supplied"),
        }
    }
}

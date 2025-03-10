use crate::frontend::snapshot::log::Log;
use crate::{
    backend::BackendSettings,
    frontend::snapshot::{Node, PropertySnapshot, Snapshot, StateInfo, StateSpace},
};
use machine_check_common::ThreeValued;
use machine_check_exec::NodeId;
use machine_check_exec::{Framework, PreparedProperty, Proposition};
use mck::concr::FullMachine;
use std::collections::{BTreeMap, HashMap};

pub struct Workspace<M: FullMachine> {
    pub framework: Framework<M>,
    pub properties: Vec<WorkspaceProperty>,
    pub log: Log,
}

pub struct WorkspaceProperty {
    pub property: PreparedProperty,
    pub children: Vec<WorkspaceProperty>,
}

impl WorkspaceProperty {
    pub fn new(property: PreparedProperty) -> WorkspaceProperty {
        let children = property
            .original()
            .children()
            .into_iter()
            .map(|child| WorkspaceProperty::new(PreparedProperty::new(child)))
            .collect();
        WorkspaceProperty { property, children }
    }
}

impl<M: FullMachine> Workspace<M> {
    pub fn new(framework: Framework<M>, property: Option<Proposition>) -> Self {
        // always put the inherent property first, add the other property afterwards if there is one
        let mut properties = vec![WorkspaceProperty::new(PreparedProperty::new(
            Proposition::inherent(),
        ))];

        if let Some(property) = property {
            properties.push(WorkspaceProperty::new(PreparedProperty::new(property)));
        }

        Workspace {
            framework,
            properties,
            log: Log::new(),
        }
    }

    pub fn generate_snapshot(&mut self, settings: &BackendSettings) -> Snapshot {
        let state_field_names: Vec<String> =
        <<M::Abstr as mck::abstr::Machine<M>>::State as mck::abstr::Manipulatable>::field_names()
            .into_iter()
            .map(String::from)
            .collect();

        let state_info = StateInfo {
            field_names: state_field_names.clone(),
        };

        let framework = &self.framework;

        let state_map = framework.space().state_map();
        let node_graph = framework.space().node_graph();

        let node_iter = std::iter::once((NodeId::ROOT, None)).chain(
            state_map
                .iter()
                .map(|(state_id, state)| ((*state_id).into(), Some(state))),
        );

        let mut nodes = BTreeMap::new();
        for (node_id, state) in node_iter {
            let incoming = node_graph
                .neighbors_directed(node_id, petgraph::Direction::Incoming)
                .collect();
            let outgoing = node_graph
                .neighbors_directed(node_id, petgraph::Direction::Outgoing)
                .collect();
            let (fields, panic) = if let Some(state) = state {
                let panic_result = &state.0;
                let can_be_nonpanic = panic_result.panic.umin().is_zero();
                let can_be_panic = panic_result.panic.umax().is_nonzero();
                let panic = match (can_be_nonpanic, can_be_panic) {
                    (true, true) => ThreeValued::Unknown,
                    (false, true) => ThreeValued::True,
                    (true, false) => ThreeValued::False,
                    (false, false) => panic!("Panic result should always contain some value"),
                };
                let mut fields = BTreeMap::new();
                for field_name in state_field_names.iter() {
                    let field_get =
                        mck::abstr::Manipulatable::get(&panic_result.result, field_name)
                            .expect("Field name should correspond to a field");
                    let description = field_get.description();

                    fields.insert(field_name.clone(), description);
                }
                (fields, Some(panic))
            } else {
                (BTreeMap::new(), None)
            };

            let node_info = Node {
                incoming,
                outgoing,
                panic,
                fields,
            };
            nodes.insert(node_id, node_info);
        }

        let state_space = StateSpace { nodes };

        let mut properties = Vec::new();
        for business_property in &self.properties {
            properties.push(Self::create_property_snapshot(
                &self.framework,
                business_property,
                &mut self.log,
            ));
        }

        Snapshot::new(
            settings.exec_name.clone(),
            state_space,
            state_info,
            properties,
            self.log.clone(),
        )
    }

    fn create_property_snapshot(
        framework: &Framework<M>,
        business_property: &WorkspaceProperty,
        log: &mut Log,
    ) -> PropertySnapshot {
        let labellings = match framework.compute_property_labelling(&business_property.property) {
            Ok(ok) => ok,
            Err(err) => {
                log.error(err.to_string());

                HashMap::new()
            }
        };
        let children = business_property
            .children
            .iter()
            .map(|child| Self::create_property_snapshot(framework, child, log))
            .collect();
        PropertySnapshot {
            property: business_property.property.clone(),
            labellings,
            children,
        }
    }
}

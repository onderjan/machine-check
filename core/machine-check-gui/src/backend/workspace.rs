use crate::backend::BackendSettings;
use crate::shared::snapshot::log::Log;
use crate::shared::snapshot::{Node, Snapshot, StateInfo, StateSpace, SubpropertySnapshot};
use machine_check_common::property::{Property, Subproperty};
use machine_check_common::ThreeValued;
use machine_check_exec::Framework;
use mck::abstr::BitvectorDomain;
use mck::concr::FullMachine;
use std::collections::{BTreeMap, BTreeSet};

/// Backend workspace.
///
/// Contains the verification framework among others.
pub struct Workspace<M: FullMachine> {
    pub framework: Framework<M>,
    pub properties: Vec<WorkspaceProperty>,
    pub log: Log,
}

/// A property stored in the backend.
pub struct WorkspaceProperty {
    pub subproperty: Subproperty,
    pub children: Vec<WorkspaceProperty>,
}

impl WorkspaceProperty {
    pub fn new(property: Property) -> WorkspaceProperty {
        Self::new_from_subproperty(property.root_subproperty())
    }

    pub fn new_from_subproperty(subproperty: Subproperty) -> WorkspaceProperty {
        let children = subproperty
            .displayed_children()
            .into_iter()
            .map(WorkspaceProperty::new_from_subproperty)
            .collect();
        WorkspaceProperty {
            subproperty,
            children,
        }
    }
}

impl<M: FullMachine> Workspace<M> {
    pub fn new(framework: Framework<M>, property: Option<Property>) -> Self {
        // always put the inherent property first, add the other property afterwards if there is one
        let mut properties = vec![WorkspaceProperty::new(Property::inherent())];

        if let Some(property) = property {
            properties.push(WorkspaceProperty::new(property));
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

        let space = &self.framework.space();

        let mut nodes = BTreeMap::new();

        //let graph_nodes = BTreeSet::from_iter();

        for node_id in space.nodes() {
            let incoming = BTreeSet::from_iter(space.direct_predecessor_iter(node_id));
            let outgoing = BTreeSet::from_iter(
                space
                    .direct_successor_iter(node_id)
                    .map(|state_id| state_id.into()),
            );

            let (fields, panic) = if let Ok(state_id) = node_id.try_into() {
                let state = space.state_data(state_id);
                let panic_unsigned = state.panic.unsigned_interval();
                let can_be_nonpanic = panic_unsigned.min().is_zero();
                let can_be_panic = panic_unsigned.max().is_nonzero();
                let panic = match (can_be_nonpanic, can_be_panic) {
                    (true, true) => ThreeValued::Unknown,
                    (false, true) => ThreeValued::True,
                    (true, false) => ThreeValued::False,
                    (false, false) => panic!("Panic result should always contain some value"),
                };
                let mut fields = BTreeMap::new();
                for field_name in state_field_names.iter() {
                    let field_get = mck::abstr::Manipulatable::get(&state.result, field_name)
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
                &mut self.framework,
                business_property,
            ));
        }

        let panic_message = self.framework.find_panic_string().map(String::from);

        Snapshot::new(
            settings.exec_name.clone(),
            state_space,
            state_info,
            properties,
            self.log.clone(),
            panic_message,
        )
    }

    fn create_property_snapshot(
        framework: &mut Framework<M>,
        business_property: &WorkspaceProperty,
    ) -> SubpropertySnapshot {
        let (conclusion, labellings) =
            match framework.check_subproperty_with_labelling(&business_property.subproperty) {
                Ok((conclusion, labellings)) => (Ok(conclusion), labellings),
                Err(error) => (Err(error), BTreeMap::new()),
            };
        let children = business_property
            .children
            .iter()
            .map(|child| Self::create_property_snapshot(framework, child))
            .collect();
        SubpropertySnapshot {
            subproperty: business_property.subproperty.clone(),
            conclusion,
            labellings,
            children,
        }
    }
}

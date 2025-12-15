use crate::backend::BackendSettings;
use crate::shared::snapshot::log::Log;
use crate::shared::snapshot::{Node, PropertySnapshot, Snapshot, StateInfo, StateSpace};
use machine_check_common::check::KnownConclusion;
use machine_check_common::iir::property::IProperty;
use machine_check_common::{ParamValuation, PropertyMacros, ThreeValued};
use machine_check_exec::Framework;
use mck::{abstr::BitvectorDomain, concr::FullMachine};
use std::collections::{BTreeMap, BTreeSet};

/// Backend workspace.
///
/// Contains the verification framework among others.
pub struct Workspace<M: FullMachine, D> {
    pub framework: Framework<M>,
    pub properties: Vec<IProperty>,
    property_macros: PropertyMacros<D>,
    pub log: Log,
}

impl<M: FullMachine, D> Workspace<M, D> {
    pub fn new(
        framework: Framework<M>,
        property: Option<IProperty>,
        property_macros: PropertyMacros<D>,
    ) -> Self {
        // always put the inherent property first, add the other property afterwards if there is one
        let mut properties = vec![machine_check_machine::inherent_property()];

        if let Some(property) = property {
            properties.push(property);
        }

        Workspace {
            framework,
            properties,
            property_macros,
            log: Log::new(),
        }
    }

    pub fn generate_snapshot(&mut self, settings: &BackendSettings) -> Snapshot {
        let state_info = StateInfo {
            fields: self.framework.machine().state().fields.clone(),
        };

        let space = &self.framework.space();

        let mut nodes = BTreeMap::new();

        for node_id in space.nodes() {
            let incoming = BTreeSet::from_iter(space.direct_predecessor_iter(node_id));
            let outgoing = BTreeSet::from_iter(
                space
                    .direct_successor_iter(node_id)
                    .map(|state_id| state_id.into()),
            );

            let panic_state = if let Ok(state_id) = node_id.try_into() {
                let panic_state = space.state_data(state_id);

                use mck::abstr::{Abstr, AbstractValue};

                // convert panic to boolean
                let panic = mck::abstr::Boolean::from_three_valued(
                    match panic_state.panic.concrete_value() {
                        Some(value) => ThreeValued::from_bool(value.is_nonzero()),
                        None => ThreeValued::Unknown,
                    },
                );

                let panic_state = AbstractValue::Struct(vec![
                    panic_state.result.to_runtime(),
                    AbstractValue::Boolean(panic),
                ]);

                Some(panic_state)
            } else {
                None
            };

            let node_info = Node {
                incoming,
                outgoing,
                panic_state: panic_state.map(|panic_state| panic_state.display()),
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
        property: &IProperty,
    ) -> PropertySnapshot {
        let (conclusion, labellings) = match framework.current_conclusion(property) {
            Ok(conclusion) => match framework.current_labellings(property) {
                Ok(labellings) => {
                    let conclusion = match conclusion {
                        machine_check_common::check::Conclusion::Known(known_conclusion) => {
                            match known_conclusion {
                                KnownConclusion::False => ParamValuation::False,
                                KnownConclusion::True => ParamValuation::True,
                                KnownConclusion::Dependent => ParamValuation::Dependent,
                            }
                        }
                        machine_check_common::check::Conclusion::Unknown(_) => {
                            ParamValuation::Unknown
                        }
                        machine_check_common::check::Conclusion::NotCheckable => {
                            ParamValuation::Unknown
                        }
                    };
                    (Ok(conclusion), labellings)
                }
                Err(error) => (Err(error), BTreeMap::new()),
            },
            Err(error) => (Err(error), BTreeMap::new()),
        };

        PropertySnapshot {
            property: property.clone(),
            conclusion,
            labellings,
        }
    }

    pub fn property_macros(&self) -> &PropertyMacros<D> {
        &self.property_macros
    }
}

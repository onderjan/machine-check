use machine_check_common::ThreeValued;
use machine_check_exec::{Framework, NodeId};
use mck::concr::FullMachine;
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

use crate::frontend::{
    interaction::Response,
    snapshot::{log::Log, Node, PropertySnapshot, Snapshot, StateInfo, StateSpace},
};

use super::{Business, BusinessProperty};

pub fn api_response<M: FullMachine>(
    business: &mut Business<M>,
) -> Result<Cow<'static, [u8]>, Box<dyn std::error::Error>> {
    let state_field_names: Vec<String> =
        <<M::Abstr as mck::abstr::Machine<M>>::State as mck::abstr::Manipulatable>::field_names()
            .into_iter()
            .map(String::from)
            .collect();

    let state_info = StateInfo {
        field_names: state_field_names.clone(),
    };

    let framework = &business.framework;

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
                let field_get = mck::abstr::Manipulatable::get(&panic_result.result, field_name)
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
    for business_property in &business.properties {
        properties.push(create_property_snapshot(
            &business.framework,
            business_property,
            &mut business.log,
        ));
    }

    let snapshot = Snapshot {
        exec_name: business.exec_name.clone(),
        state_space,
        state_info,
        properties,
        log: business.log.clone(),
    };

    let response = Response { snapshot };

    let content_msgpack = rmp_serde::to_vec(&response)?;
    Ok(Cow::Owned(content_msgpack))
}

fn create_property_snapshot<M: FullMachine>(
    framework: &Framework<M>,
    business_property: &BusinessProperty,
    business_log: &mut Log,
) -> PropertySnapshot {
    let labellings = match framework.compute_property_labelling(&business_property.property) {
        Ok(ok) => ok,
        Err(err) => {
            business_log.error(err.to_string());

            HashMap::new()
        }
    };
    let children = business_property
        .children
        .iter()
        .map(|child| create_property_snapshot(framework, child, business_log))
        .collect();
    PropertySnapshot {
        property: business_property.property.clone(),
        labellings,
        children,
    }
}

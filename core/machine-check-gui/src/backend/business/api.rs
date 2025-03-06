use machine_check_exec::NodeId;
use mck::concr::FullMachine;
use std::{borrow::Cow, collections::BTreeMap};

use crate::frontend::{
    interaction::Response,
    snapshot::{Node, Snapshot, StateInfo, StateSpace, ThreeValuedBool},
};

use super::Business;

pub fn api_response<M: FullMachine>(
    business: &Business<M>,
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

    let node_iter = std::iter::once((NodeId::START, None)).chain(
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
            let panic = ThreeValuedBool {
                zero: panic_result.panic.umin().is_zero(),
                one: panic_result.panic.umax().is_nonzero(),
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

    let snapshot = Snapshot {
        exec_name: business.exec_name.clone(),
        state_space,
        state_info,
    };

    let response = Response { snapshot };

    let content_msgpack = rmp_serde::to_vec(&response)?;
    Ok(Cow::Owned(content_msgpack))
}

use bimap::BiHashMap;
use std::collections::HashMap;
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};

use crate::shared::snapshot::Snapshot;

use super::{NodeAux, Tile, TileType};
use machine_check_exec::NodeId;

pub fn compute_tiling_aux(
    snapshot: &Snapshot,
) -> (BiHashMap<Tile, TileType>, HashMap<NodeId, NodeAux>) {
    // compute predecessor/successor reserved y-positions using reverse topological sort
    let (sorted, canonical_predecessors) = topological_sort(snapshot);
    let mut reserved = HashMap::<NodeId, usize>::new();

    for node_id in sorted.iter().rev().cloned() {
        let node = snapshot.state_space.nodes.get(&node_id).unwrap();
        // reserve one position if there is one non-identity predecessor, two positions if there are two
        let nonidentity_predecessors = node
            .incoming
            .iter()
            .filter(|successor_id| **successor_id != node_id)
            .count();
        let predecessor_reserve = if nonidentity_predecessors > 1 { 2 } else { 1 };

        let mut successor_reserve = 0;

        for successor_id in node.outgoing.iter().cloned() {
            if successor_id == node_id {
                // do not reserve anything for the identity successor, it is a loop
                continue;
            }

            // reserve the y-positions of each non-identity successor
            // but reserve only one if they do not consider this a canonical precedessor
            if *canonical_predecessors.get(&successor_id).unwrap() == node_id {
                successor_reserve += *reserved.entry(successor_id).or_default();
            } else {
                successor_reserve += 1;
            }
        }

        reserved.insert(node_id, predecessor_reserve.max(successor_reserve).max(1));
    }

    console_log!("Reserved: {:?}", reserved);

    // stage tile positions by topological sort, taking the reserved y-positions into account
    let mut tiling = BiHashMap::new();
    let mut node_aux = HashMap::new();
    tiling.insert(Tile { x: 0, y: 0 }, TileType::Node(NodeId::ROOT));
    let mut stack = Vec::new();
    stack.push(NodeId::ROOT);

    for node_id in sorted {
        let parent_node_id = canonical_predecessors.get(&node_id).copied();
        let node = snapshot.state_space.nodes.get(&node_id).unwrap();
        let node_tile = *tiling
            .get_by_right(&TileType::Node(node_id))
            .expect("Node should be in tiling");

        // ignore loops and canonical predecessors for the incoming reference
        let incoming_reference_predecessors: BTreeSet<NodeId> = node
            .incoming
            .iter()
            .cloned()
            .filter(|predecessor_id| {
                *predecessor_id != node_id && *predecessor_id != parent_node_id.unwrap()
            })
            .collect();

        let predecessor_split_len = if !incoming_reference_predecessors.is_empty() {
            let (left, right) = (
                Tile {
                    x: node_tile.x - 1,
                    y: node_tile.y + 1 as i64,
                },
                TileType::IncomingReference(incoming_reference_predecessors, node_id),
            );

            if tiling.insert(left, right).did_overwrite() {
                panic!(
                    "Should not overwrite tile {:?} by an ingoing reference",
                    left
                );
            }
            1
        } else {
            0
        };

        let mut y_add = 0u64;

        let has_multiple_node_successors = node
            .outgoing
            .iter()
            .filter(|successor_id| {
                **successor_id != node_id
                    && *canonical_predecessors.get(*successor_id).unwrap() == node_id
            })
            .count()
            > 1;
        let some_node_successor_has_complex_incoming = node.outgoing.iter().any(|successor_id| {
            if *successor_id != node_id
                && *canonical_predecessors.get(successor_id).unwrap() == node_id
            {
                let successor = snapshot.state_space.nodes.get(successor_id).unwrap();
                successor
                    .incoming
                    .iter()
                    .any(|sibling_id| *sibling_id != node_id && sibling_id != successor_id)
            } else {
                false
            }
        });

        let successor_x_offset: u64 =
            if has_multiple_node_successors && some_node_successor_has_complex_incoming {
                2
            } else {
                1
            };

        let mut successor_split_len = 0;
        let mut self_loop = false;
        let mut tiling_children = Vec::new();
        for successor_id in snapshot
            .state_space
            .nodes
            .get(&node_id)
            .unwrap()
            .outgoing
            .iter()
            .cloned()
        {
            if successor_id == node_id {
                // skip identity successors and mark self-loop
                self_loop = true;
                continue;
            }

            let (left, right) = if !tiling.contains_right(&TileType::Node(successor_id)) {
                tiling_children.push(successor_id);
                stack.push(successor_id);

                (
                    Tile {
                        x: node_tile.x + successor_x_offset as i64,
                        y: node_tile.y + y_add as i64,
                    },
                    TileType::Node(successor_id),
                )
            } else {
                (
                    Tile {
                        x: node_tile.x + successor_x_offset as i64,
                        y: node_tile.y + y_add as i64,
                    },
                    TileType::OutgoingReference(node_id, successor_id),
                )
            };

            if tiling.insert(left, right).did_overwrite() {
                panic!(
                    "Should not overwrite tile {:?} by a node or outgoing reference",
                    left
                );
            }

            successor_split_len = y_add;

            if *canonical_predecessors.get(&successor_id).unwrap() == node_id {
                y_add += *reserved.get(&successor_id).unwrap() as u64;
            } else {
                y_add += 1;
            }
        }
        node_aux.insert(
            node_id,
            NodeAux {
                tiling_parent: parent_node_id,
                tiling_children,
                predecessor_split_len,
                successor_split_len,
                successor_x_offset,
                self_loop,
            },
        );
    }

    (tiling, node_aux)
}

fn topological_sort(content: &Snapshot) -> (Vec<NodeId>, HashMap<NodeId, NodeId>) {
    // construct a topological ordering using Kahn's algorithm on a DAG
    // the node without any incoming edge is the root
    let (mut dag_outgoing, mut dag_incoming_degree, canonical_predecessors) = {
        let mut seen = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(NodeId::ROOT);

        // construct Directed Acyclic Graph

        let mut dag_outgoing: BTreeMap<NodeId, BTreeSet<NodeId>> = BTreeMap::new();
        let mut dag_incoming_degree: BTreeMap<NodeId, usize> = BTreeMap::new();
        let mut canonical_predecessors = HashMap::new();

        while let Some(node_id) = queue.pop_front() {
            seen.insert(node_id);
            visited.insert(node_id);

            for successor_id in content
                .state_space
                .nodes
                .get(&node_id)
                .unwrap()
                .outgoing
                .iter()
                .cloned()
            {
                if !seen.contains(&successor_id) {
                    seen.insert(successor_id);
                    dag_outgoing
                        .entry(node_id)
                        .or_default()
                        .insert(successor_id);
                    *dag_incoming_degree.entry(successor_id).or_default() += 1;
                    canonical_predecessors.insert(successor_id, node_id);

                    queue.push_back(successor_id);
                }
            }
        }
        (dag_outgoing, dag_incoming_degree, canonical_predecessors)
    };

    // use Kahn's algorithn

    let mut queue = VecDeque::new();
    queue.push_back(NodeId::ROOT);
    let mut sorted = Vec::new();

    while let Some(node_id) = queue.pop_front() {
        sorted.push(node_id);

        for successor_id in dag_outgoing.entry(node_id).or_default().iter().cloned() {
            let incoming = dag_incoming_degree.entry(successor_id).or_default();

            assert_ne!(*incoming, 0);
            *incoming -= 1;
            if *incoming == 0 {
                queue.push_back(successor_id);
            }
        }
    }

    (sorted, canonical_predecessors)
}

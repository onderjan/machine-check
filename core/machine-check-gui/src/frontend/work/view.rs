pub mod camera;
mod canvas;
mod compute;
mod constants;
mod fields;

use std::collections::HashMap;

use bimap::BiHashMap;
use camera::Camera;
use machine_check_exec::NodeId;

use crate::frontend::snapshot::Snapshot;

#[derive(Debug)]
pub struct View {
    pub snapshot: Snapshot,
    pub tiling: BiHashMap<Tile, TileType>,
    pub node_aux: HashMap<NodeId, NodeAux>,
    pub camera: Camera,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Tile {
    pub x: u64,
    pub y: u64,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TileType {
    Node(NodeId),
    IncomingReference(NodeId, NodeId),
    OutgoingReference(NodeId, NodeId),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct NodeAux {
    pub tiling_parent: Option<NodeId>,
    pub tiling_children: Vec<NodeId>,

    pub predecessor_split_len: u64,
    pub successor_split_len: u64,
    pub successor_x_offset: u64,
    pub self_loop: bool,
}

pub enum NavigationTarget {
    Root,
    Up,
    Left,
    Right,
    Down,
}

impl View {
    pub fn new(snapshot: Snapshot, camera: Camera) -> View {
        let (tiling, node_aux) = compute::compute_tiling_aux(&snapshot);
        View {
            snapshot,
            tiling,
            node_aux,
            camera,
        }
    }

    pub fn render(&self, force: bool) {
        canvas::render(self, force);
        fields::display(self);
    }

    pub fn navigate(&mut self, target: NavigationTarget) {
        if let Some(selected_node_id) = self.camera.selected_node_id {
            let selected_node_aux = self
                .node_aux
                .get(&selected_node_id)
                .expect("Selected node id should point to a valid node aux");

            match target {
                NavigationTarget::Root => {
                    // go to the root node
                    self.camera.selected_node_id = Some(NodeId::START);
                }
                NavigationTarget::Up => {
                    // go to the previous child of the tiling parent
                    if let Some(tiling_parent_id) = selected_node_aux.tiling_parent {
                        let tiling_parent_aux = self
                            .node_aux
                            .get(&tiling_parent_id)
                            .expect("Tiling parent id should point to a valid node aux");
                        let selected_index = tiling_parent_aux
                            .tiling_children
                            .iter()
                            .position(|e| *e == selected_node_id)
                            .unwrap();
                        let new_selected_index = selected_index.saturating_sub(1);
                        self.camera.selected_node_id =
                            Some(tiling_parent_aux.tiling_children[new_selected_index]);
                    }
                }
                NavigationTarget::Left => {
                    // go to the tiling parent
                    if let Some(tiling_parent_id) = selected_node_aux.tiling_parent {
                        self.camera.selected_node_id = Some(tiling_parent_id);
                    }
                }
                NavigationTarget::Right => {
                    // go to first tiling child
                    if let Some(first_child_id) = selected_node_aux.tiling_children.first() {
                        self.camera.selected_node_id = Some(*first_child_id);
                    }
                }
                NavigationTarget::Down => {
                    // go to the next child of the tiling parent
                    if let Some(tiling_parent_id) = selected_node_aux.tiling_parent {
                        let tiling_parent_aux = self
                            .node_aux
                            .get(&tiling_parent_id)
                            .expect("Tiling parent id should point to a valid node aux");
                        let selected_index = tiling_parent_aux
                            .tiling_children
                            .iter()
                            .position(|e| *e == selected_node_id)
                            .unwrap();
                        if let Some(new_selected_node_id) =
                            tiling_parent_aux.tiling_children.get(selected_index + 1)
                        {
                            self.camera.selected_node_id = Some(*new_selected_node_id);
                        }
                    }
                }
            }
        } else {
            self.camera.selected_node_id = Some(NodeId::START);
        }
    }
}

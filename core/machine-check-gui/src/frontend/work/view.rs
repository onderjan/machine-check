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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeAux {
    pub predecessor_split_len: u64,
    pub successor_split_len: u64,
    pub successor_x_offset: u64,
    pub self_loop: bool,
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
}

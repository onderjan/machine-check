use std::collections::{BTreeSet, HashMap};

use bimap::BiHashMap;
use machine_check_common::NodeId;

use crate::shared::snapshot::Snapshot;

mod compute;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Tile {
    pub x: i64,
    pub y: i64,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TileType {
    Node(NodeId),
    IncomingReference(BTreeSet<NodeId>, NodeId),
    OutgoingReference(NodeId, NodeId),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct NodeTileInfo {
    pub tile: Tile,
    pub tiling_parent: Option<NodeId>,
    pub tiling_children: Vec<NodeId>,

    pub predecessor_split_len: u64,
    pub successor_split_len: u64,
    pub successor_x_offset: u64,
    pub self_loop: bool,
}

impl NodeTileInfo {
    pub fn rendering_bounds(&self) -> (Tile, Tile) {
        let top_left = Tile {
            x: self.tile.x - 1,
            y: self.tile.y,
        };
        let bottom_right = Tile {
            x: self.tile.x + self.successor_x_offset as i64,
            y: self.tile.y + self.predecessor_split_len.max(self.successor_split_len) as i64,
        };
        (top_left, bottom_right)
    }
}

#[derive(Debug)]
pub struct Tiling {
    snapshot: Snapshot,
    tile_types: BiHashMap<Tile, TileType>,
    node_tile_info: HashMap<NodeId, NodeTileInfo>,
}
impl Tiling {
    pub fn new(snapshot: Snapshot) -> Self {
        let (tile_types, node_tile_info) = compute::compute_tiling_aux(&snapshot);

        Self {
            snapshot,
            tile_types,
            node_tile_info,
        }
    }

    pub fn at_tile(&self, tile: Tile) -> Option<&TileType> {
        self.tile_types.get_by_left(&tile)
    }

    pub fn tile_types(&self) -> &BiHashMap<Tile, TileType> {
        &self.tile_types
    }

    pub fn node_tile_info(&self) -> &HashMap<NodeId, NodeTileInfo> {
        &self.node_tile_info
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
}

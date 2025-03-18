pub mod camera;
mod compute;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use bimap::BiHashMap;
use camera::Camera;
use machine_check_exec::NodeId;

use crate::shared::{
    snapshot::{PropertySnapshot, Snapshot},
    BackendInfo,
};

use super::util::PixelPoint;

#[derive(Debug)]
pub struct View {
    snapshot: Snapshot,
    pub backend_info: BackendInfo,
    pub tiling: BiHashMap<Tile, TileType>,
    pub node_aux: HashMap<NodeId, NodeAux>,
    pub camera: Camera,
    pub column_widths: BTreeMap<i64, u64>,
    pub column_starts: BTreeMap<i64, i64>,
}

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
pub struct NodeAux {
    pub tile: Tile,
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
    pub fn new(snapshot: Snapshot, backend_info: BackendInfo, mut camera: Camera) -> View {
        let (tiling, node_aux) = compute::compute_tiling_aux(&snapshot);

        camera.apply_snapshot(&snapshot);

        let column_widths = BTreeMap::new();
        let column_starts = BTreeMap::new();

        View {
            snapshot,
            backend_info,
            tiling,
            node_aux,
            camera,
            column_widths,
            column_starts,
        }
    }

    pub fn apply_snapshot(&mut self, snapshot: Snapshot) {
        (self.tiling, self.node_aux) = compute::compute_tiling_aux(&snapshot);
        self.camera.apply_snapshot(&snapshot);
        self.snapshot = snapshot;
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
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
                    self.camera.selected_node_id = Some(NodeId::ROOT);
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
            self.camera.selected_node_id = Some(NodeId::ROOT);
        };

        // make sure the newly selected node is in view

        if let Some(selected_node_id) = self.camera.selected_node_id {
            let selected_node_aux = self
                .node_aux
                .get(&selected_node_id)
                .expect("Selected node id should point to a valid node aux");

            // make sure the selected node is in view
            self.show_tile(selected_node_aux.tile);
        }
    }

    fn show_tile(&mut self, tile: Tile) {
        let (mut top_left, mut bottom_right) = self.tile_rect(tile);
        // adjust by half of default tile size
        let node_size = self.camera.scheme.tile_size / 2;
        let node_size_point = PixelPoint {
            x: node_size as i64,
            y: node_size as i64,
        };
        top_left -= node_size_point;
        bottom_right += node_size_point;

        self.camera.ensure_in_view(top_left);
        self.camera.ensure_in_view(bottom_right);
    }

    pub fn selected_subproperty(&self) -> Option<&PropertySnapshot> {
        self.camera
            .selected_subproperty
            .map(|selected_property_index| {
                self.snapshot.select_subproperty(selected_property_index)
            })
    }

    pub fn selected_root_property(&self) -> Option<&PropertySnapshot> {
        self.camera
            .selected_subproperty
            .map(|selected_subproperty_index| {
                let selected_property_index = self
                    .snapshot
                    .subindex_to_root_index(selected_subproperty_index);

                self.snapshot.select_root_property(selected_property_index)
            })
    }

    pub fn global_point_to_tile(&self, point: PixelPoint, ceil: bool) -> Tile {
        let tile_size = self.camera.scheme.tile_size;

        let func = if ceil { f64::ceil } else { f64::floor };

        // TODO: constant-time tile position from point
        let tile_x = if point.x < 0 {
            func(point.x as f64 / tile_size as f64) as i64
        } else {
            let mut selected_column = None;
            for (column, column_start) in self.column_starts.iter() {
                if point.x < *column_start {
                    if ceil {
                        selected_column = Some(*column)
                    } else {
                        selected_column = Some(*column - 1)
                    }
                    break;
                }
            }

            if let Some(selected_column) = selected_column {
                selected_column
            } else {
                let (last_column, last_column_start) = self
                    .column_starts
                    .last_key_value()
                    .map(|(k, v)| (*k, *v))
                    .unwrap_or((0, 0));
                last_column + (func((point.x - last_column_start) as f64 / tile_size as f64) as i64)
            }
        };

        let tile_y = func(point.y as f64 / tile_size as f64) as i64;

        Tile {
            x: tile_x,
            y: tile_y,
        }

        //let tile_x = func(point.x as f64 / tile_size as f64) as i64;
    }

    pub fn viewport_point_to_tile(&self, point: PixelPoint, ceil: bool) -> Tile {
        let global_point = point + self.camera.view_offset();
        self.global_point_to_tile(global_point, ceil)
    }

    pub fn column_width(&self, column: i64) -> u64 {
        if let Some(width) = self.column_widths.get(&column) {
            return *width;
        }
        self.camera.scheme.tile_size
    }

    pub fn column_start(&self, column: i64) -> i64 {
        let tile_size = self.camera.scheme.tile_size;
        if column < 0 {
            return column * tile_size as i64;
        }
        if let Some(start) = self.column_starts.get(&column) {
            return *start;
        }

        let (last_column, last_column_start) = self
            .column_starts
            .last_key_value()
            .map(|(k, v)| (*k, *v))
            .unwrap_or((0, 0));

        let from_last_column = column - last_column;
        last_column_start + from_last_column * tile_size as i64
    }

    pub fn tile_rect(&self, tile: Tile) -> (PixelPoint, PixelPoint) {
        let tile_size = self.camera.scheme.tile_size as i64;
        let top_left_x = self.column_start(tile.x);
        let top_left_y = tile_size * tile.y;
        let bottom_right_x = self.column_start(tile.x + 1) - 1;
        let bottom_right_y = tile_size * (tile.y + 1) - 1;
        (
            PixelPoint {
                x: top_left_x,
                y: top_left_y,
            },
            PixelPoint {
                x: bottom_right_x,
                y: bottom_right_y,
            },
        )
    }
}

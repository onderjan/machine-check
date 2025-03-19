pub mod camera;

use std::collections::BTreeMap;

use bimap::BiBTreeMap;
use camera::{Camera, Scheme};
use machine_check_exec::NodeId;
use rstar::{
    primitives::{GeomWithData, Rectangle},
    RTree, AABB,
};

use crate::{
    frontend::tiling::TileType,
    shared::{
        snapshot::{PropertySnapshot, Snapshot, SubpropertyIndex},
        BackendInfo,
    },
};

use super::{
    tiling::{NodeTileInfo, Tile, Tiling},
    util::{web_idl::main_canvas_with_context, PixelPoint, PixelRect},
};

#[derive(Debug)]
struct AxisSizing {
    normal_size: u64,
    sizes: BTreeMap<i64, u64>,
    starts: BiBTreeMap<i64, i64>,
}

impl AxisSizing {
    fn new(normal_size: u64, sizes: BTreeMap<i64, u64>) -> Self {
        let mut starts = BiBTreeMap::new();

        if let (Some((smallest_size_index, _)), Some((largest_size_index, _))) =
            (sizes.first_key_value(), sizes.last_key_value())
        {
            let (smallest_size_index, largest_size_index) =
                (*smallest_size_index, *largest_size_index);

            // the offset at the index 0 will be 0
            // first, work down from it to the smallest size index
            // do not process 0 in this manner and always pre-subtract the size at index one higher
            {
                let mut offset: i64 = 0;
                for index in (smallest_size_index..0).rev() {
                    offset -= sizes.get(&(index + 1)).copied().unwrap_or(normal_size) as i64;
                    starts.insert(index, offset);
                }
            }

            // next, work up from index 0 to the largest size index
            // process 0 and always post-add the current element size
            {
                let mut offset: i64 = 0;
                for index in 0..=largest_size_index + 1 {
                    starts.insert(index, offset);
                    offset += sizes.get(&index).copied().unwrap_or(normal_size) as i64;
                }
            }

            // this should make sure that the offsets from min(smallest_size_index, 0)
            // to max(0, largest_size_index) inclusive are present in the starts map
        }

        Self {
            normal_size,
            sizes,
            starts,
        }
    }

    fn index_size(&self, index: i64) -> u64 {
        self.sizes.get(&index).copied().unwrap_or(self.normal_size)
    }

    fn index_lower_offset(&self, index: i64) -> i64 {
        if let Some(start) = self.starts.get_by_left(&index) {
            return *start;
        }

        let (boundary_index, boundary_start) = if index >= 0 {
            // select last
            self.starts.iter().next_back()
        } else {
            // select first
            self.starts.iter().next()
        }
        .map(|(k, v)| (*k, *v))
        .unwrap_or((0, 0));

        let boundary_offset = index - boundary_index;
        boundary_start + boundary_offset * self.normal_size as i64
    }

    fn index_higher_offset(&self, index: i64) -> i64 {
        let lower_offset = self.index_lower_offset(index);
        let width = self.index_size(index);
        if width >= 1 {
            lower_offset + width as i64 - 1
        } else {
            lower_offset
        }
    }

    fn index_of(&self, offset: i64) -> i64 {
        // search by offset: should be this or lower
        let this_or_lower = self.starts.right_range(..=offset).next_back();

        let index = if let Some((lower_index, lower_offset)) = this_or_lower {
            // we have to make sure it is high enough
            let lower_size = self.index_size(*lower_index);
            if lower_offset + (lower_size as i64) < offset {
                // we can assume the lower size is normal size
                lower_index + (offset - lower_offset).div_euclid(self.normal_size as i64)
            } else {
                *lower_index
            }
        } else {
            // no lower offset, get the lowest offset and compute from there
            if let Some((lowest_index, lowest_offset)) = self.starts.iter().next() {
                lowest_index + (offset - lowest_offset).div_euclid(self.normal_size as i64)
            } else {
                // no lowest offset, just compute from normal size
                offset.div_euclid(self.normal_size as i64)
            }
        };
        index
    }
}

#[derive(Debug)]
struct Selection {
    selected_node_id: Option<NodeId>,
    selected_subproperty: Option<SubpropertyIndex>,
}

impl Selection {
    fn new(snapshot: &Snapshot) -> Self {
        // select the last property if it is available
        Self {
            selected_node_id: None,
            selected_subproperty: snapshot.last_property_subindex(),
        }
    }

    fn apply_snapshot(&mut self, snapshot: &Snapshot) {
        // make sure the selected things are still available
        if let Some(selected_node_id) = self.selected_node_id {
            if !snapshot.state_space.nodes.contains_key(&selected_node_id) {
                self.selected_node_id = None;
            }
        }

        if let Some(selected_property_index) = self.selected_subproperty {
            if !snapshot.contains_subindex(selected_property_index) {
                self.selected_subproperty = None;
            }
        }

        // if no property is selected, select the last one if it is available
        if self.selected_subproperty.is_none() {
            if let Some(last_property_subindex) = snapshot.last_property_subindex() {
                self.selected_subproperty = Some(last_property_subindex);
            }
        }
    }
}

#[derive(Debug)]
struct Presentation {
    tiling: Tiling,
    column_sizing: AxisSizing,
    row_sizing: AxisSizing,
    visibility_tree: RTree<GeomWithData<Rectangle<(i64, i64)>, Tile>>,
}

impl Presentation {
    fn new(snapshot: Snapshot, scheme: &Scheme) -> Self {
        let tiling = Tiling::new(snapshot);

        let (column_sizes, row_sizes) = Self::compute_column_row_sizes(&tiling, scheme);
        let column_sizing = AxisSizing::new(scheme.tile_size, column_sizes);
        let row_sizing = AxisSizing::new(scheme.tile_size, row_sizes);

        let tiles_vec: Vec<_> = tiling
            .tile_types()
            .iter()
            .map(|(tile, tile_type)| {
                let rect = if let TileType::Node(node_id) = tile_type {
                    let (top_left, bottom_right) = tiling
                        .node_tile_info()
                        .get(node_id)
                        .unwrap()
                        .rendering_bounds();
                    let (top_left, _) = tile_rect(&column_sizing, &row_sizing, top_left);
                    let (_, bottom_right) = tile_rect(&column_sizing, &row_sizing, bottom_right);
                    (top_left, bottom_right)
                } else {
                    tile_rect(&column_sizing, &row_sizing, *tile)
                };

                GeomWithData::new(
                    Rectangle::from_corners((rect.0.x, rect.0.y), (rect.1.x, rect.1.y)),
                    *tile,
                )
            })
            .collect();

        let visibility_tree = RTree::bulk_load(tiles_vec);
        Self {
            tiling,
            column_sizing,
            row_sizing,
            visibility_tree,
        }
    }

    fn compute_column_row_sizes(
        tiling: &Tiling,
        scheme: &Scheme,
    ) -> (BTreeMap<i64, u64>, BTreeMap<i64, u64>) {
        let default_tile_size = scheme.tile_size;
        let mut column_sizes = BTreeMap::new();
        let mut row_sizes = BTreeMap::new();

        let context = main_canvas_with_context().1;
        let context_scheme = scheme.context_scheme(&context);
        for (tile, tile_type) in tiling.tile_types() {
            let (required_column_size, required_row_size) =
                context_scheme.required_tile_size(tile_type);
            let column_size = column_sizes.entry(tile.x).or_insert(default_tile_size);
            let row_size = row_sizes.entry(tile.y).or_insert(default_tile_size);
            *column_size = (*column_size).max(required_column_size);
            *row_size = (*row_size).max(required_row_size);
        }
        (column_sizes, row_sizes)
    }

    fn tiles_in_pixel_rect(&self, rect: PixelRect) -> impl Iterator<Item = Tile> + use<'_> {
        let aabb = AABB::from_corners(rect.top_left().to_tuple(), rect.bottom_right().to_tuple());
        let located = self.visibility_tree.locate_in_envelope_intersecting(&aabb);
        located.map(|geom_with_data| geom_with_data.data)
    }

    fn tile_rect(&self, tile: Tile) -> PixelRect {
        let top_left = PixelPoint::new(
            self.column_sizing.index_lower_offset(tile.x),
            self.row_sizing.index_lower_offset(tile.y),
        );
        let bottom_right = PixelPoint::new(
            self.column_sizing.index_higher_offset(tile.x),
            self.row_sizing.index_higher_offset(tile.y),
        );
        PixelRect::new(top_left, bottom_right)
    }
}

fn tile_rect(
    column_sizing: &AxisSizing,
    row_sizing: &AxisSizing,
    tile: Tile,
) -> (PixelPoint, PixelPoint) {
    let top_left_x = column_sizing.index_lower_offset(tile.x);
    let top_left_y = row_sizing.index_lower_offset(tile.y);
    let bottom_right_x = column_sizing.index_higher_offset(tile.x);
    let bottom_right_y = row_sizing.index_higher_offset(tile.y);
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

#[derive(Debug)]
pub struct View {
    backend_info: BackendInfo,
    presentation: Presentation,
    selection: Selection,
    pub camera: Camera,
}

pub enum NavigationTarget {
    Root,
    Up,
    Left,
    Right,
    Down,
}

impl View {
    pub fn backend_info(&self) -> &BackendInfo {
        &self.backend_info
    }

    pub fn update_backend_info(&mut self, backend_info: BackendInfo) {
        self.backend_info = backend_info;
    }

    pub fn new(snapshot: Snapshot, backend_info: BackendInfo) -> View {
        let selection = Selection::new(&snapshot);
        let camera = Camera::new();
        let presentation = Presentation::new(snapshot, &camera.scheme);

        Self {
            backend_info,
            presentation,
            selection,
            camera,
        }
    }

    pub fn apply_snapshot(&mut self, snapshot: Snapshot) {
        self.selection.apply_snapshot(&snapshot);
        self.presentation = Presentation::new(snapshot, &self.camera.scheme);
    }

    pub fn snapshot(&self) -> &Snapshot {
        self.presentation.tiling.snapshot()
    }

    pub fn navigate(&mut self, target: NavigationTarget) {
        if let Some(selected_node_id) = self.selected_node_id() {
            let selected_node_aux = self
                .node_tile_info(selected_node_id)
                .expect("Selected node id should point to a valid node");

            match target {
                NavigationTarget::Root => {
                    // go to the root node
                    self.set_selected_node_id(Some(NodeId::ROOT));
                }
                NavigationTarget::Up => {
                    // go to the previous child of the tiling parent
                    if let Some(tiling_parent_id) = selected_node_aux.tiling_parent {
                        let tiling_parent_aux = self
                            .node_tile_info(tiling_parent_id)
                            .expect("Tiling parent id should point to a valid node");
                        let selected_index = tiling_parent_aux
                            .tiling_children
                            .iter()
                            .position(|e| *e == selected_node_id)
                            .unwrap();
                        let new_selected_index = selected_index.saturating_sub(1);
                        self.set_selected_node_id(Some(
                            tiling_parent_aux.tiling_children[new_selected_index],
                        ));
                    }
                }
                NavigationTarget::Left => {
                    // go to the tiling parent
                    if let Some(tiling_parent_id) = selected_node_aux.tiling_parent {
                        self.set_selected_node_id(Some(tiling_parent_id));
                    }
                }
                NavigationTarget::Right => {
                    // go to first tiling child
                    if let Some(first_child_id) = selected_node_aux.tiling_children.first() {
                        self.set_selected_node_id(Some(*first_child_id));
                    }
                }
                NavigationTarget::Down => {
                    // go to the next child of the tiling parent
                    if let Some(tiling_parent_id) = selected_node_aux.tiling_parent {
                        let tiling_parent_aux = self
                            .node_tile_info(tiling_parent_id)
                            .expect("Tiling parent id should point to a valid node");
                        let selected_index = tiling_parent_aux
                            .tiling_children
                            .iter()
                            .position(|e| *e == selected_node_id)
                            .unwrap();
                        if let Some(new_selected_node_id) =
                            tiling_parent_aux.tiling_children.get(selected_index + 1)
                        {
                            self.set_selected_node_id(Some(*new_selected_node_id));
                        }
                    }
                }
            }
        } else {
            self.set_selected_node_id(Some(NodeId::ROOT));
        };

        // make sure the newly selected node is in view

        if let Some(selected_node_id) = self.selected_node_id() {
            let selected_node_aux = self
                .node_tile_info(selected_node_id)
                .expect("Selected node id should point to a valid node");

            // make sure the selected node is in view
            self.show_tile(selected_node_aux.tile);
        }
    }

    fn show_tile(&mut self, tile: Tile) {
        let tile_rect = self.tile_rect(tile);
        console_log!("Ensuring in view: {:?}", tile_rect);
        let mut top_left = tile_rect.top_left();
        let mut bottom_right = tile_rect.bottom_right();
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
        self.selection
            .selected_subproperty
            .map(|selected_property_index| {
                self.snapshot().select_subproperty(selected_property_index)
            })
    }

    pub fn selected_subproperty_index(&self) -> Option<SubpropertyIndex> {
        self.selection.selected_subproperty
    }

    pub fn selected_root_property(&self) -> Option<&PropertySnapshot> {
        self.selection
            .selected_subproperty
            .map(|selected_subproperty_index| {
                let selected_property_index = self
                    .snapshot()
                    .subindex_to_root_index(selected_subproperty_index);

                self.snapshot()
                    .select_root_property(selected_property_index)
            })
    }

    pub fn selected_node_id(&self) -> Option<NodeId> {
        self.selection.selected_node_id
    }

    pub fn tile_at_global_point(&self, point: PixelPoint) -> Tile {
        let x = self.presentation.column_sizing.index_of(point.x);
        let y = self.presentation.row_sizing.index_of(point.y);
        Tile { x, y }
    }

    pub fn tile_at_viewport_point(&self, point: PixelPoint) -> Tile {
        let global_point = point + self.camera.view_offset();
        self.tile_at_global_point(global_point)
    }

    pub fn select_subproperty_index(&mut self, index: Option<SubpropertyIndex>) {
        self.selection.selected_subproperty = index;
    }

    pub fn scheme(&self) -> &Scheme {
        &self.camera.scheme
    }

    pub fn tile_rect(&self, tile: Tile) -> PixelRect {
        self.presentation.tile_rect(tile)
    }

    pub fn tiles_in_rect(&self, rect: PixelRect) -> impl Iterator<Item = Tile> + use<'_> {
        self.presentation.tiles_in_pixel_rect(rect)
    }

    pub fn node_rect(&self, tile: Tile) -> PixelRect {
        let node_half_margin = self.scheme().node_half_margin();
        self.tile_rect(tile)
            .without_half_margin(node_half_margin, node_half_margin)
    }

    pub fn view_offset(&self) -> PixelPoint {
        self.camera.view_offset()
    }

    pub fn tile_type(&self, tile: Tile) -> Option<&TileType> {
        self.presentation.tiling.at_tile(tile)
    }

    pub fn node_tile_info(&self, node_id: NodeId) -> Option<&NodeTileInfo> {
        self.presentation.tiling.node_tile_info().get(&node_id)
    }

    pub fn set_selected_node_id(&mut self, node_id: Option<NodeId>) {
        self.selection.selected_node_id = node_id;
    }

    pub fn mouse_drag_start(&mut self, mouse_coords: PixelPoint) -> bool {
        self.camera.mouse_down_coords = Some(mouse_coords);
        self.camera.mouse_current_coords = Some(mouse_coords);
        true
    }

    pub fn mouse_drag_update(&mut self, mouse_coords: PixelPoint) -> bool {
        self.camera.mouse_current_coords = Some(mouse_coords);
        self.camera.mouse_down_coords.is_some()
    }

    pub fn mouse_drag_end(&mut self, mouse_coords: PixelPoint) -> bool {
        let Some(mouse_down_coords) = self.camera.mouse_down_coords.take() else {
            return false;
        };
        let offset = mouse_coords - mouse_down_coords;
        self.camera.view_offset -= offset;
        true
    }

    pub fn set_view_size(&mut self, width: u32, height: u32) {
        self.camera.view_size = PixelPoint {
            x: width as i64,
            y: height as i64,
        }
    }
}

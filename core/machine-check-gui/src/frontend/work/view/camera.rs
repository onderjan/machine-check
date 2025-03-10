use machine_check_exec::NodeId;

use crate::frontend::{
    snapshot::SubpropertyIndex,
    util::{web_idl::window, PixelPoint},
};

use super::{
    constants::{RAW_ARROWHEAD_SIZE, RAW_FONT_SIZE, RAW_NODE_SIZE, RAW_TILE_SIZE},
    Tile,
};

#[derive(Debug, Clone)]
pub struct Camera {
    pub scheme: Scheme,
    pub view_offset: PixelPoint,

    pub mouse_current_coords: Option<PixelPoint>,
    pub mouse_down_coords: Option<PixelPoint>,
    pub selected_node_id: Option<NodeId>,
    pub selected_subproperty: Option<SubpropertyIndex>,
}

#[derive(Debug, Clone)]
pub struct Scheme {
    pub pixel_ratio: f64,
    pub tile_size: u64,
    pub node_size: u64,
    pub arrowhead_size: f64,
    pub font_size: f64,
}

impl Scheme {
    fn new() -> Self {
        let pixel_ratio = window().device_pixel_ratio();

        let tile_size = adjust_size(RAW_TILE_SIZE * pixel_ratio) as u64;
        let node_size = adjust_size(RAW_NODE_SIZE * pixel_ratio) as u64;
        let arrowhead_size = adjust_size(RAW_ARROWHEAD_SIZE * pixel_ratio);

        let font_size = RAW_FONT_SIZE * pixel_ratio;

        Scheme {
            pixel_ratio,
            tile_size,
            node_size,
            arrowhead_size,
            font_size,
        }
    }

    pub fn global_px_tile(&self, global_point: PixelPoint) -> Option<Tile> {
        let tile_size = adjust_size(RAW_TILE_SIZE * self.pixel_ratio) as i64;
        let tile_pos = global_point / tile_size;

        let tile_x: Result<u64, _> = tile_pos.x.try_into();
        let tile_y: Result<u64, _> = tile_pos.y.try_into();

        if let (Ok(tile_x), Ok(tile_y)) = (tile_x, tile_y) {
            Some(Tile {
                x: tile_x,
                y: tile_y,
            })
        } else {
            None
        }
    }
}

fn adjust_size(unadjusted: f64) -> f64 {
    // make sure half-size is even
    (unadjusted / 2.).round() * 2.
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            scheme: Scheme::new(),
            view_offset: PixelPoint { x: 0, y: 0 },
            mouse_current_coords: None,
            mouse_down_coords: None,
            selected_node_id: None,
            selected_subproperty: None,
        }
    }

    pub fn view_offset(&self) -> PixelPoint {
        let mut result = self.view_offset;
        if let (Some(mouse_down_px), Some(mouse_current_px)) =
            (self.mouse_down_coords, self.mouse_current_coords)
        {
            let mouse_offset = mouse_current_px - mouse_down_px;
            result -= mouse_offset;
        }
        result
    }

    pub fn viewport_px_tile(&self, viewport_point: PixelPoint) -> Option<Tile> {
        let global_point = viewport_point + self.view_offset;

        self.scheme.global_px_tile(global_point)
    }

    pub fn apply_snapshot(&mut self, snapshot: &crate::frontend::snapshot::Snapshot) {
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

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

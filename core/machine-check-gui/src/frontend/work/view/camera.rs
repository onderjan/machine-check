use machine_check_exec::NodeId;

use crate::frontend::util::PixelPoint;

use super::Tile;

#[derive(Debug, Clone)]
pub struct Camera {
    pub scheme: Scheme,
    pub view_offset: PixelPoint,

    pub mouse_current_coords: Option<PixelPoint>,
    pub mouse_down_coords: Option<PixelPoint>,
    pub selected_node_id: Option<NodeId>,
}

#[derive(Debug, Clone)]
pub struct Scheme {
    pub pixel_ratio: f64,
    pub tile_size: u64,
    pub node_size: u64,
    pub arrowhead_size: f64,
}

impl Scheme {
    fn new() -> Self {
        let pixel_ratio = web_sys::window()
            .expect("Window should exist")
            .device_pixel_ratio();

        let tile_size = adjust_size(RAW_TILE_SIZE * pixel_ratio) as u64;
        let node_size = adjust_size(RAW_NODE_SIZE * pixel_ratio) as u64;
        let arrowhead_size = adjust_size(RAW_ARROWHEAD_SIZE * pixel_ratio);

        Scheme {
            pixel_ratio,
            tile_size,
            node_size,
            arrowhead_size,
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

const RAW_TILE_SIZE: f64 = 46.;
const RAW_NODE_SIZE: f64 = 30.;
const RAW_ARROWHEAD_SIZE: f64 = 4.;

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
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

use machine_check_exec::NodeId;
use web_sys::CanvasRenderingContext2d;

use crate::frontend::{
    tiling::TileType,
    util::{
        constants::{
            RAW_ARROWHEAD_SIZE, RAW_FONT_MARGIN, RAW_FONT_SIZE, RAW_NODE_SIZE, RAW_TILE_SIZE,
        },
        web_idl::window,
        PixelPoint,
    },
};

#[derive(Debug, Clone)]
pub struct Camera {
    pub scheme: Scheme,
    pub view_offset: PixelPoint,
    pub view_size: PixelPoint,

    pub mouse_current_coords: Option<PixelPoint>,
    pub mouse_down_coords: Option<PixelPoint>,
}

#[derive(Debug, Clone)]
pub struct Scheme {
    pub tile_size: u64,
    pub node_size: u64,
    pub arrowhead_size: u64,
    pub font_size: f64,
    pub font_margin: f64,
}

impl Scheme {
    fn new() -> Self {
        let pixel_ratio = window().device_pixel_ratio();

        let tile_size = adjust_size(RAW_TILE_SIZE * pixel_ratio) as u64;
        let node_size = adjust_size(RAW_NODE_SIZE * pixel_ratio) as u64;
        let arrowhead_size = adjust_size(RAW_ARROWHEAD_SIZE * pixel_ratio) as u64;

        let font_size = RAW_FONT_SIZE * pixel_ratio;
        let font_margin = RAW_FONT_MARGIN * pixel_ratio;

        Scheme {
            tile_size,
            node_size,
            arrowhead_size,
            font_size,
            font_margin,
        }
    }

    pub fn node_margin(&self) -> u64 {
        self.tile_size - self.node_size
    }

    pub fn node_half_margin(&self) -> u64 {
        self.node_margin() / 2
    }

    pub fn context_scheme<'a>(
        &'a self,
        context: &'a CanvasRenderingContext2d,
    ) -> ContextScheme<'a> {
        ContextScheme {
            scheme: self,
            context,
        }
    }

    pub fn node_text(node_id: NodeId) -> String {
        node_id.to_string()
    }

    pub fn reference_text(
        head_node_ids: impl Iterator<Item = NodeId>,
        tail_node_id: NodeId,
    ) -> String {
        let mut head_string = String::new();

        let mut first = true;

        for head_node_id in head_node_ids {
            if first {
                first = false;
            } else {
                head_string += ",";
            }
            head_string += &head_node_id.to_string();
        }

        format!("{}\u{1f852}{}", head_string, tail_node_id)
    }
}

pub struct ContextScheme<'a> {
    scheme: &'a Scheme,
    context: &'a CanvasRenderingContext2d,
}

impl ContextScheme<'_> {
    pub fn required_tile_size(&self, tile_type: &TileType) -> (u64, u64) {
        let text = match tile_type {
            TileType::Node(node_id) => Scheme::node_text(*node_id),
            TileType::IncomingReference(head_node_id, tail_node_id) => {
                Scheme::reference_text(head_node_id.iter().copied(), *tail_node_id)
            }
            TileType::OutgoingReference(head_node_id, tail_node_id) => {
                Scheme::reference_text(std::iter::once(*head_node_id), *tail_node_id)
            }
        };
        let text_metrics = self.context.measure_text(&text).unwrap();
        let mut width =
            text_metrics.actual_bounding_box_left() + text_metrics.actual_bounding_box_right();
        let height =
            text_metrics.actual_bounding_box_ascent() + text_metrics.actual_bounding_box_descent();

        if matches!(
            tile_type,
            TileType::IncomingReference(_, _) | TileType::OutgoingReference(_, _)
        ) {
            // add the beak to the consideration
            // the beak blocks width equal to the half of the height
            width += height;
        }

        width += self.scheme.font_margin;
        width += self.scheme.node_margin() as f64;

        let (width, height) = (width.ceil() as u64, height.ceil() as u64);

        (width, height)
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
            view_size: PixelPoint { x: 0, y: 0 },
            mouse_current_coords: None,
            mouse_down_coords: None,
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

    pub fn ensure_in_view(&mut self, point: PixelPoint) {
        if self.view_offset.x > point.x {
            self.view_offset.x = point.x;
        }
        if self.view_offset.y > point.y {
            self.view_offset.y = point.y;
        }
        let min_offset_x = point.x - self.view_size.x;
        if self.view_offset.x < min_offset_x {
            self.view_offset.x = min_offset_x;
        }
        let min_offset_y = point.y - self.view_size.y;
        if self.view_offset.y < min_offset_y {
            self.view_offset.y = min_offset_y
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

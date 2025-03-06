use machine_check_exec::NodeId;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement};

use crate::frontend::{content::Node, util::PixelPoint};

use super::{
    view::{Tile, TileType},
    PointOfView, View,
};

pub fn render(view: &View, point_of_view: &PointOfView, resize: bool) {
    LOCAL.with(|local| {
        Renderer {
            view,
            point_of_view,
            local,
        }
        .render(resize);
    });
}

pub fn get_tile_from_px(point: PixelPoint) -> Option<Tile> {
    LOCAL.with(|local| {
        let tile_size = adjust_size(RAW_TILE_SIZE * local.pixel_ratio) as i64;
        let tile_pos = point / tile_size;

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
    })
}

const RAW_TILE_SIZE: f64 = 46.;
const RAW_NODE_SIZE: f64 = 30.;
const RAW_ARROWHEAD_SIZE: f64 = 4.;

struct Renderer<'a> {
    view: &'a View,
    point_of_view: &'a PointOfView,
    local: &'a Local,
}

impl Renderer<'_> {
    fn render(&self, resize: bool) {
        if resize {
            self.fix_resized_canvas();
        }

        // clear canvas
        self.local.main_context.clear_rect(
            0.,
            0.,
            self.local.main_canvas.width() as f64,
            self.local.main_canvas.height() as f64,
        );

        self.local.main_context.save();
        let view_offset = self.point_of_view.view_offset();
        // the view offset must be subtracted to render to the viewport
        self.local
            .main_context
            .translate(-view_offset.x as f64, -view_offset.y as f64)
            .unwrap();

        for (tile, tile_type) in &self.view.tiling {
            match tile_type {
                TileType::Node(node_id) => {
                    let node = self
                        .view
                        .content
                        .state_space
                        .nodes
                        .get(node_id)
                        .expect("Tiling should have a node");
                    let aux = self.view.node_aux.get(node_id).unwrap();

                    self.render_node(*tile, *node_id, node);

                    if !node.outgoing.is_empty() {
                        self.render_arrow_start(*tile, aux.successor_x_offset);
                    }
                    self.render_arrow_split(*tile, 0, aux.predecessor_split_len);
                    self.render_arrow_split(
                        *tile,
                        aux.successor_x_offset as i64,
                        aux.successor_split_len,
                    );

                    if !node.incoming.is_empty() {
                        self.render_arrow_end(*tile);
                    }
                    if aux.self_loop {
                        self.render_self_loop(*tile);
                    }
                }
                TileType::IncomingReference(head_node_id, tail_node_id) => {
                    self.render_reference(*tile, *head_node_id, *tail_node_id, false);
                    self.render_arrow_start(*tile, 1);
                }
                TileType::OutgoingReference(head_node_id, tail_node_id) => {
                    self.render_arrow_end(*tile);
                    self.render_reference(*tile, *head_node_id, *tail_node_id, true);
                }
            }
        }

        self.local.main_context.restore();
    }

    fn tile_size(&self) -> f64 {
        adjust_size(RAW_TILE_SIZE * self.local.pixel_ratio)
    }

    fn node_size(&self) -> f64 {
        adjust_size(RAW_NODE_SIZE * self.local.pixel_ratio)
    }

    fn arrowhead_size(&self) -> f64 {
        adjust_size(RAW_ARROWHEAD_SIZE * self.local.pixel_ratio)
    }

    fn render_node(&self, tile: Tile, node_id: NodeId, node: &Node) {
        let context = &self.local.main_context;

        let is_selected = if let Some(selected_node_id) = self.point_of_view.selected_node_id {
            selected_node_id == node_id
        } else {
            false
        };

        context.set_fill_style_str(match &node.panic {
            None => "lightblue",
            Some(tv) => match (tv.zero, tv.one) {
                (true, true) => "#CCCCCC",  // grey
                (false, true) => "#CC2222", // red
                (true, false) => "#4CBF50", // green
                (false, false) => "blue",
            },
        });
        //if is_selected { "lightblue" } else { "white" });

        let (tile_size, node_size) = (self.tile_size(), self.node_size());

        let node_start_x = tile.x as f64 * tile_size + (tile_size - node_size) / 2.;
        let node_start_y = tile.y as f64 * tile_size + (tile_size - node_size) / 2.;

        let radius = 4.;

        context.set_line_width(if is_selected { 3. } else { 1. });

        context.begin_path();
        context
            .round_rect_with_f64(node_start_x, node_start_y, node_size, node_size, radius)
            .unwrap();
        context.fill();
        context.stroke();

        context.set_line_width(1.);
        context.set_fill_style_str("black");

        context
            .fill_text(
                &node_id.to_string(),
                node_start_x + node_size / 2.,
                node_start_y + node_size / 2.,
            )
            .unwrap();
    }

    fn render_reference(
        &self,
        tile: Tile,
        head_node_id: NodeId,
        tail_node_id: NodeId,
        outgoing: bool,
    ) {
        let outgoing = if outgoing { 1. } else { -1. };
        let context = &self.local.main_context;
        context.begin_path();

        let (tile_size, node_size) = (self.tile_size(), self.node_size());

        let middle_x = tile.x as f64 * tile_size + tile_size / 2.;
        let middle_y = tile.y as f64 * tile_size + tile_size / 2.;
        let upper_y = (middle_y - node_size / 3.).round();
        let lower_y = (middle_y + node_size / 3.).round();
        let sharp_x = middle_x - outgoing * (node_size / 4.);
        let sharper_x = middle_x - outgoing * (node_size / 2.);
        let blunt_x = middle_x + outgoing * (node_size / 2.);

        context.set_fill_style_str("#F5F5DC"); // very light yellow

        context.move_to(blunt_x, upper_y);
        context.line_to(sharp_x, upper_y);
        context.line_to(sharper_x, middle_y);
        context.line_to(sharp_x, lower_y);
        context.line_to(blunt_x, lower_y);
        context.close_path();
        context.fill();
        context.stroke();

        context.set_fill_style_str("black");

        context
            .fill_text(
                &format!("{}|{}", head_node_id, tail_node_id),
                middle_x + outgoing * (node_size / 12.),
                middle_y,
            )
            .unwrap();
    }

    fn render_arrow_start(&self, head_tile: Tile, successor_x_offset: u64) {
        let context = &self.local.main_context;

        let (tile_size, node_size) = (self.tile_size(), self.node_size());

        // draw the arrowshaft
        context.begin_path();
        let right_x = head_tile.x as f64 * tile_size + (successor_x_offset as f64 * tile_size);
        let tile_right_border_x =
            head_tile.x as f64 * tile_size + tile_size - (tile_size - node_size) / 2.;
        let tile_middle_y = head_tile.y as f64 * tile_size + tile_size / 2.;
        context.move_to(tile_right_border_x, tile_middle_y);
        context.line_to(right_x, tile_middle_y);
        context.stroke();
    }

    fn render_arrow_split(&self, node_tile: Tile, x_offset: i64, split_len: u64) {
        let context = &self.local.main_context;

        let tile_size = self.tile_size();

        // draw the arrow split
        context.begin_path();
        let split_x = node_tile.x as f64 * tile_size + tile_size * x_offset as f64;
        let split_upper_y = node_tile.y as f64 * tile_size + tile_size / 2.;
        let split_lower_y = split_upper_y + split_len as f64 * tile_size;
        context.move_to(split_x, split_upper_y);
        context.line_to(split_x, split_lower_y);
        context.stroke();
    }

    fn render_arrow_end(&self, tail_tile: Tile) {
        let context = &self.local.main_context;

        let (tile_size, node_size) = (self.tile_size(), self.node_size());

        // draw the arrowshaft
        context.begin_path();
        let tile_left_x = tail_tile.x as f64 * tile_size;
        let tile_left_border_x = tile_left_x + (tile_size - node_size) / 2.;
        let tile_middle_y = tail_tile.y as f64 * tile_size + tile_size / 2.;
        context.move_to(tile_left_x, tile_middle_y);
        context.line_to(tile_left_border_x, tile_middle_y);
        context.stroke();

        // draw the arrowhead
        let arrowhead_size = self.arrowhead_size();

        let arrowhead_left_x = tile_left_border_x - arrowhead_size;
        let arrowhead_upper_y = tile_middle_y - arrowhead_size / 2.;
        let arrowhead_lower_y = tile_middle_y + arrowhead_size / 2.;
        context.begin_path();
        context.move_to(tile_left_border_x, tile_middle_y);
        context.line_to(arrowhead_left_x, arrowhead_upper_y);
        context.line_to(arrowhead_left_x, arrowhead_lower_y);
        context.close_path();
        context.fill();
    }

    fn render_self_loop(&self, node_tile: Tile) {
        let context = &self.local.main_context;

        let (tile_size, node_size) = (self.tile_size(), self.node_size());

        // draw the arrowshaft
        context.begin_path();
        let tile_right_x = node_tile.x as f64 * tile_size + tile_size;
        let tile_middle_x = node_tile.x as f64 * tile_size + tile_size / 2.;
        let tile_middle_y = node_tile.y as f64 * tile_size + tile_size / 2.;
        let tile_upper_y = node_tile.y as f64 * tile_size;
        let tile_upper_border_y = tile_upper_y + (tile_size - node_size) / 2.;
        context.move_to(tile_right_x, tile_middle_y);
        context.line_to(tile_right_x, tile_upper_y);
        context.line_to(tile_middle_x, tile_upper_y);
        context.line_to(tile_middle_x, tile_upper_border_y);
        context.stroke();

        // draw the arrowhead
        let arrowhead_size = self.arrowhead_size();

        let arrowhead_left_x = tile_middle_x - arrowhead_size / 2.;
        let arrowhead_right_x = tile_middle_x + arrowhead_size / 2.;
        let arrowhead_upper_y = tile_upper_border_y - arrowhead_size;
        context.begin_path();
        context.move_to(tile_middle_x, tile_upper_border_y);
        context.line_to(arrowhead_left_x, arrowhead_upper_y);
        context.line_to(arrowhead_right_x, arrowhead_upper_y);
        context.close_path();
        context.fill();
    }

    fn fix_resized_canvas(&self) {
        // fix for device pixel ratio
        let pixel_ratio = self.local.pixel_ratio;
        let main_area_rect = self.local.main_area.get_bounding_client_rect();
        let width = main_area_rect.width();
        let height = main_area_rect.height();

        // the actual canvas width and height must be a whole number
        let pr_width = (width * pixel_ratio) as u32;
        let pr_height = (height * pixel_ratio) as u32;
        self.local.main_canvas.set_width(pr_width);
        self.local.main_canvas.set_height(pr_height);

        // set canvas element width and height exactly as divided by the pixel ratio so there is no error
        let width = pr_width as f64 / pixel_ratio;
        let height = pr_height as f64 / pixel_ratio;

        let canvas_style = self.local.main_canvas.style();
        canvas_style
            .set_property("width", &format!("{}px", width))
            .unwrap();
        canvas_style
            .set_property("height", &format!("{}px", height))
            .unwrap();

        // set font size
        let font_size = 12. * pixel_ratio;
        self.local
            .main_context
            .set_font(&format!("{}px sans-serif", font_size));
        self.local.main_context.set_text_align("center");
        self.local.main_context.set_text_baseline("middle");

        // make sure we stroke true pixels
        self.local.main_context.reset_transform().unwrap();
        self.local.main_context.translate(0.5, 0.5).unwrap();
    }
}

struct Local {
    main_area: Element,
    main_canvas: HtmlCanvasElement,
    main_context: CanvasRenderingContext2d,
    pixel_ratio: f64,
}

impl Local {
    fn new() -> Local {
        let window = web_sys::window().expect("HTML Window should exist");
        let document = window.document().expect("HTML document should exist");
        let main_area = document
            .get_element_by_id("main_area")
            .expect("Main area should exist");
        let main_canvas = document
            .get_element_by_id("main_canvas")
            .expect("Main canvas should exist");
        let main_canvas: HtmlCanvasElement = main_canvas
            .dyn_into()
            .expect("Main canvas should be a Canvas element");
        let main_context: CanvasRenderingContext2d = main_canvas
            .get_context("2d")
            .expect("Main canvas 2D context should be obtainable without an error")
            .expect("Main canvas should have a 2D context")
            .dyn_into()
            .expect("Main canvas 2D rendering context should be castable");
        let pixel_ratio = window.device_pixel_ratio();

        Local {
            main_area,
            main_canvas,
            main_context,
            pixel_ratio,
        }
    }
}

thread_local! {
    static LOCAL: Local = Local::new();
}

fn adjust_size(unadjusted: f64) -> f64 {
    // make sure half-size is even
    (unadjusted / 2.).round() * 2.
}

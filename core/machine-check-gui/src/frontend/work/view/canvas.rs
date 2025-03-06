mod primitives;

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement};

use super::{TileType, View};

pub fn render(view: &View, force: bool) {
    CanvasRenderer::new(view).render(force);
}

struct CanvasRenderer<'a> {
    view: &'a View,
    main_area: Element,
    main_canvas: HtmlCanvasElement,
    main_context: CanvasRenderingContext2d,
}

impl CanvasRenderer<'_> {
    fn render(&self, force: bool) {
        if force {
            self.fix_resized_canvas();
        }

        // clear canvas
        self.main_context.clear_rect(
            0.,
            0.,
            self.main_canvas.width() as f64,
            self.main_canvas.height() as f64,
        );

        self.main_context.save();
        let view_offset = self.view.camera.view_offset();
        // the view offset must be subtracted to render to the viewport
        self.main_context
            .translate(-view_offset.x as f64, -view_offset.y as f64)
            .unwrap();

        for (tile, tile_type) in &self.view.tiling {
            match tile_type {
                TileType::Node(node_id) => {
                    let node = self
                        .view
                        .snapshot
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

        self.main_context.restore();
    }

    fn new(view: &View) -> CanvasRenderer {
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

        CanvasRenderer {
            view,
            main_area,
            main_canvas,
            main_context,
        }
    }

    fn fix_resized_canvas(&self) {
        // fix for device pixel ratio
        let pixel_ratio = self.view.camera.scheme.pixel_ratio;
        let main_area_rect = self.main_area.get_bounding_client_rect();
        let width = main_area_rect.width();
        let height = main_area_rect.height();

        // the actual canvas width and height must be a whole number
        let pr_width = (width * pixel_ratio) as u32;
        let pr_height = (height * pixel_ratio) as u32;
        self.main_canvas.set_width(pr_width);
        self.main_canvas.set_height(pr_height);

        // set canvas element width and height exactly as divided by the pixel ratio so there is no error
        let width = pr_width as f64 / pixel_ratio;
        let height = pr_height as f64 / pixel_ratio;

        let canvas_style = self.main_canvas.style();
        canvas_style
            .set_property("width", &format!("{}px", width))
            .unwrap();
        canvas_style
            .set_property("height", &format!("{}px", height))
            .unwrap();

        // set font size
        self.main_context.set_font(&format!(
            "{}px sans-serif",
            self.view.camera.scheme.font_size
        ));
        self.main_context.set_text_align("center");
        self.main_context.set_text_baseline("middle");

        // make sure we stroke true pixels
        self.main_context.reset_transform().unwrap();
        self.main_context.translate(0.5, 0.5).unwrap();
    }
}

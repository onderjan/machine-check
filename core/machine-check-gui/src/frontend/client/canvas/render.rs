mod primitives;

use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement};

use crate::frontend::{
    tiling::{Tile, TileType},
    util::{
        web_idl::{get_element_by_id, main_canvas_with_context, window},
        PixelPoint, PixelRect,
    },
    view::View,
};

pub fn setup() {
    CanvasRenderer::new().setup();
}

pub fn render(view: &mut View) {
    CanvasRenderer::new().render(view);
}

struct CanvasRenderer {
    main_area: Element,
    main_canvas: HtmlCanvasElement,
    main_context: CanvasRenderingContext2d,
}

impl CanvasRenderer {
    fn render(&self, view: &mut View) {
        let canvas_width = self.main_canvas.width();
        let canvas_height = self.main_canvas.height();
        // make sure the view size is correct
        view.set_view_size(canvas_width, canvas_height);

        // clear canvas
        self.main_context
            .clear_rect(0., 0., canvas_width as f64, canvas_height as f64);

        // set font size
        self.main_context
            .set_font(&format!("{}px sans-serif", view.scheme().font_size));
        self.main_context.set_text_align("center");
        self.main_context.set_text_baseline("middle");

        // the view offset must be subtracted to render to the viewport
        self.main_context.save();
        let view_offset = view.view_offset();
        self.main_context
            .translate(-view_offset.x as f64, -view_offset.y as f64)
            .unwrap();

        self.render_background(view);

        // use the labellings corresponding to the selected subproperty
        let labellings = view
            .selected_subproperty()
            .map(|selected_property| &selected_property.labellings);

        let lesser_visible_point = view_offset;
        let greater_visible_point = lesser_visible_point
            + PixelPoint {
                x: self.main_canvas.width() as i64,
                y: self.main_canvas.height() as i64,
            };
        let viewport = PixelRect::new(lesser_visible_point, greater_visible_point);

        let tiles_in_view: Vec<_> = view.tiles_in_rect(viewport).collect();

        console_log!("Tiles in view: {:?}", tiles_in_view);

        for tile in tiles_in_view {
            let tile_type = view
                .tile_type(tile)
                .expect("Tile in view should be populated");
            match tile_type {
                TileType::Node(node_id) => {
                    let node = view
                        .snapshot()
                        .state_space
                        .nodes
                        .get(node_id)
                        .expect("Tiling should have a node");

                    let node_tile_info = view.node_tile_info(*node_id).unwrap();
                    let labelling = if let (Some(labellings), Ok(state_id)) =
                        (labellings, (*node_id).try_into())
                    {
                        labellings.get(&state_id).copied()
                    } else {
                        None
                    };

                    let is_selected = if let Some(selected_node_id) = view.selected_node_id() {
                        selected_node_id == *node_id
                    } else {
                        false
                    };

                    self.render_node(view, tile, *node_id, labelling, is_selected);

                    if !node.incoming.is_empty() {
                        self.render_arrow_end(view, tile);
                    }
                    if node_tile_info.self_loop {
                        self.render_self_loop(view, tile);
                    }

                    if !node.outgoing.is_empty() {
                        self.render_arrow_start(view, tile, node_tile_info.successor_x_offset);
                    }

                    self.render_arrow_split(view, tile, 0, node_tile_info.predecessor_split_len);

                    self.render_arrow_split(
                        view,
                        tile,
                        node_tile_info.successor_x_offset as i64,
                        node_tile_info.successor_split_len,
                    );
                }
                TileType::IncomingReference(head_node_ids, tail_node_id) => {
                    self.render_reference(
                        view,
                        tile,
                        head_node_ids.iter().copied(),
                        *tail_node_id,
                        false,
                    );
                    self.render_arrow_start(view, tile, 1);
                }
                TileType::OutgoingReference(head_node_id, tail_node_id) => {
                    self.render_arrow_end(view, tile);
                    self.render_reference(
                        view,
                        tile,
                        std::iter::once(*head_node_id),
                        *tail_node_id,
                        true,
                    );
                }
            }
        }

        self.main_context.restore();
    }

    fn visible_tile_range(
        &self,
        view: &View,
    ) -> (std::ops::RangeInclusive<i64>, std::ops::RangeInclusive<i64>) {
        let lesser_visible_point = view.view_offset();
        let greater_visible_point = lesser_visible_point
            + PixelPoint {
                x: self.main_canvas.width() as i64,
                y: self.main_canvas.height() as i64,
            };

        let lesser_tile = view.tile_at_global_point(lesser_visible_point);
        let greater_tile = view.tile_at_global_point(greater_visible_point);

        let x_range = lesser_tile.x..=greater_tile.x;
        let y_range = lesser_tile.y..=greater_tile.y;

        (x_range, y_range)
    }

    fn render_background(&self, view: &View) {
        self.main_context.save();

        self.main_context.set_fill_style_str("#FAFAFA");
        self.main_context.set_stroke_style_str("#DDD");

        let (range_x, range_y) = self.visible_tile_range(view);

        for tile_x in range_x {
            for tile_y in range_y.clone() {
                let tile_rect = view.tile_rect(Tile {
                    x: tile_x,
                    y: tile_y,
                });

                self.main_context.begin_path();
                self.main_context.rect(
                    tile_rect.left_x() as f64,
                    tile_rect.top_y() as f64,
                    tile_rect.width() as f64,
                    tile_rect.height() as f64,
                );

                self.main_context.fill();
                self.main_context.stroke();
            }
        }

        self.main_context.restore();
    }

    fn new() -> CanvasRenderer {
        let main_area = get_element_by_id("main_area");
        let (main_canvas, main_context) = main_canvas_with_context();
        CanvasRenderer {
            main_area,
            main_canvas,
            main_context,
        }
    }

    fn setup(&self) {
        // fix for device pixel ratio
        let pixel_ratio = window().device_pixel_ratio();
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

        // make sure we stroke true pixels
        self.main_context.reset_transform().unwrap();
        self.main_context.translate(0.5, 0.5).unwrap();
    }
}

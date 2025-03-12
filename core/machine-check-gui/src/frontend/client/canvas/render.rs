mod primitives;

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement};

use crate::frontend::{
    util::{
        web_idl::{get_element_by_id, window},
        PixelPoint,
    },
    view::{
        camera::{Camera, Scheme},
        Tile, TileType, View,
    },
};

pub fn setup() {
    CanvasRenderer::new().setup();
}

pub fn render(view: &View) {
    CanvasRenderer::new().render(view);
}

struct CanvasRenderer {
    main_area: Element,
    main_canvas: HtmlCanvasElement,
    main_context: CanvasRenderingContext2d,
}

impl CanvasRenderer {
    fn render(&self, view: &View) {
        // clear canvas
        self.main_context.clear_rect(
            0.,
            0.,
            self.main_canvas.width() as f64,
            self.main_canvas.height() as f64,
        );

        // set font size
        self.main_context
            .set_font(&format!("{}px sans-serif", view.camera.scheme.font_size));
        self.main_context.set_text_align("center");
        self.main_context.set_text_baseline("middle");

        // the view offset must be subtracted to render to the viewport
        self.main_context.save();
        let view_offset = view.camera.view_offset();
        self.main_context
            .translate(-view_offset.x as f64, -view_offset.y as f64)
            .unwrap();

        self.render_background(view);

        let scheme = &view.camera.scheme;

        // use the labellings corresponding to the selected subproperty
        let labellings = view
            .selected_subproperty()
            .map(|selected_property| &selected_property.labellings);

        let (x_range, y_range) = self.visible_tile_range(&view.camera);

        for (tile, tile_type) in &view.tiling {
            let tile_visible = is_tile_visible(&x_range, &y_range, *tile);

            match tile_type {
                TileType::Node(node_id) => {
                    let node = view
                        .snapshot()
                        .state_space
                        .nodes
                        .get(node_id)
                        .expect("Tiling should have a node");

                    let aux = view.node_aux.get(node_id).unwrap();
                    if tile_visible {
                        let labelling = if let (Some(labellings), Ok(state_id)) =
                            (labellings, (*node_id).try_into())
                        {
                            labellings.get(&state_id).copied()
                        } else {
                            None
                        };

                        let is_selected =
                            if let Some(selected_node_id) = view.camera.selected_node_id {
                                selected_node_id == *node_id
                            } else {
                                false
                            };

                        self.render_node(scheme, *tile, *node_id, labelling, is_selected);

                        if !node.incoming.is_empty() {
                            self.render_arrow_end(scheme, *tile);
                        }
                        if aux.self_loop {
                            self.render_self_loop(scheme, *tile);
                        }

                        if !node.outgoing.is_empty() {
                            self.render_arrow_start(scheme, *tile, aux.successor_x_offset);
                        }
                    }

                    let predecessor_bound_tile = Tile {
                        x: tile.x - 1,
                        y: tile.y + aux.predecessor_split_len as i64,
                    };

                    if is_tile_rectangle_visible(&x_range, &y_range, *tile, predecessor_bound_tile)
                    {
                        self.render_arrow_split(scheme, *tile, 0, aux.predecessor_split_len);
                    }

                    let successor_bound_tile = Tile {
                        x: tile.x + aux.successor_x_offset as i64,
                        y: tile.y + aux.successor_split_len as i64,
                    };

                    if is_tile_rectangle_visible(&x_range, &y_range, *tile, successor_bound_tile) {
                        self.render_arrow_split(
                            scheme,
                            *tile,
                            aux.successor_x_offset as i64,
                            aux.successor_split_len,
                        );
                    }
                }
                TileType::IncomingReference(head_node_id, tail_node_id) => {
                    if tile_visible {
                        self.render_reference(scheme, *tile, *head_node_id, *tail_node_id, false);
                        self.render_arrow_start(scheme, *tile, 1);
                    }
                }
                TileType::OutgoingReference(head_node_id, tail_node_id) => {
                    if tile_visible {
                        self.render_arrow_end(scheme, *tile);
                        self.render_reference(scheme, *tile, *head_node_id, *tail_node_id, true);
                    }
                }
            }
        }

        self.main_context.restore();
    }

    fn visible_tile_range(
        &self,
        camera: &Camera,
    ) -> (std::ops::RangeInclusive<i64>, std::ops::RangeInclusive<i64>) {
        let lesser_visible_point = camera.view_offset();
        let greater_visible_point = lesser_visible_point
            + PixelPoint {
                x: self.main_canvas.width() as i64,
                y: self.main_canvas.height() as i64,
            };

        let lesser_tile = tile_position_from_point(&camera.scheme, lesser_visible_point, false);
        let greater_tile = tile_position_from_point(&camera.scheme, greater_visible_point, true);

        let x_range = lesser_tile.0..=greater_tile.0;
        let y_range = lesser_tile.1..=greater_tile.1;

        (x_range, y_range)
    }

    fn render_background(&self, view: &View) {
        self.main_context.save();

        self.main_context.set_fill_style_str("#FAFAFA");
        self.main_context.set_stroke_style_str("#DDD");

        let tile_size = view.camera.scheme.tile_size;

        let (range_x, range_y) = self.visible_tile_range(&view.camera);

        for tile_x in range_x {
            for tile_y in range_y.clone() {
                /*if (tile_x as u64).wrapping_add(tile_y as u64) % 2 == 1 {
                    self.main_context.set_fill_style_str("#FFFFFF");
                } else {
                    self.main_context.set_fill_style_str("#FAFAFA");
                }*/

                let start = PixelPoint {
                    x: tile_x * tile_size as i64,
                    y: tile_y * tile_size as i64,
                };

                self.main_context.begin_path();
                self.main_context.rect(
                    start.x as f64,
                    start.y as f64,
                    tile_size as f64,
                    tile_size as f64,
                );

                self.main_context.fill();
                self.main_context.stroke();
            }
        }

        self.main_context.restore();
    }

    fn new() -> CanvasRenderer {
        let main_area = get_element_by_id("main_area");
        let main_canvas = get_element_by_id("main_canvas");
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

fn tile_position_from_point(scheme: &Scheme, point: PixelPoint, ceil: bool) -> (i64, i64) {
    let tile_size = scheme.tile_size;

    let func = if ceil { f64::ceil } else { f64::floor };

    let tile_x = func(point.x as f64 / tile_size as f64) as i64;
    let tile_y = func(point.y as f64 / tile_size as f64) as i64;
    (tile_x, tile_y)
}

fn is_tile_visible(
    x_range: &std::ops::RangeInclusive<i64>,
    y_range: &std::ops::RangeInclusive<i64>,
    tile: Tile,
) -> bool {
    x_range.contains(&tile.x) && y_range.contains(&tile.y)
}

fn is_tile_rectangle_visible(
    x_range: &std::ops::RangeInclusive<i64>,
    y_range: &std::ops::RangeInclusive<i64>,
    tile_a: Tile,
    tile_b: Tile,
) -> bool {
    let low_x = tile_a.x.min(tile_b.x);
    let high_x = tile_a.x.max(tile_b.x);
    let tile_range_x = low_x..=high_x;

    let low_y = tile_a.y.min(tile_b.y);
    let high_y = tile_a.y.max(tile_b.y);
    let tile_range_y = low_y..=high_y;

    ranges_intersect(x_range, &tile_range_x) && ranges_intersect(y_range, &tile_range_y)
}

fn ranges_intersect(
    range_a: &std::ops::RangeInclusive<i64>,
    range_b: &std::ops::RangeInclusive<i64>,
) -> bool {
    range_a.start().max(range_b.start()) <= range_a.end().min(range_b.end())
}

mod primitives;

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement};

use crate::frontend::{
    util::{
        web_idl::{get_element_by_id, window},
        PixelPoint,
    },
    view::{Tile, TileType, View},
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
        self.adjust_view(view);

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

        // use the labellings corresponding to the selected subproperty
        let labellings = view
            .selected_subproperty()
            .map(|selected_property| &selected_property.labellings);

        let (x_range, y_range) = self.visible_tile_range(view);

        let lesser_visible_point = view.camera.view_offset();
        let greater_visible_point = lesser_visible_point
            + PixelPoint {
                x: self.main_canvas.width() as i64,
                y: self.main_canvas.height() as i64,
            };

        let tiles_in_view: Vec<_> = view
            .tiling
            .tiles_in_rect(lesser_visible_point, greater_visible_point)
            .collect();

        console_log!("Tiles in view: {:?}", tiles_in_view);

        //for (tile, tile_type) in view.tiling.map_iter() {
        //    let tile_visible = is_tile_visible(&x_range, &y_range, *tile);
        for tile in view
            .tiling
            .tiles_in_rect(lesser_visible_point, greater_visible_point)
        {
            let tile = &tile;
            let tile_type = view.tiling.map.get_by_left(&tile).unwrap();
            let tile_visible = true;
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

                        self.render_node(view, *tile, *node_id, labelling, is_selected);

                        if !node.incoming.is_empty() {
                            self.render_arrow_end(view, *tile);
                        }
                        if aux.self_loop {
                            self.render_self_loop(view, *tile);
                        }

                        if !node.outgoing.is_empty() {
                            self.render_arrow_start(view, *tile, aux.successor_x_offset);
                        }
                    }

                    let predecessor_bound_tile = Tile {
                        x: tile.x - 1,
                        y: tile.y + aux.predecessor_split_len as i64,
                    };

                    if is_tile_rectangle_visible(&x_range, &y_range, *tile, predecessor_bound_tile)
                    {
                        self.render_arrow_split(view, *tile, 0, aux.predecessor_split_len);
                    }

                    let successor_bound_tile = Tile {
                        x: tile.x + aux.successor_x_offset as i64,
                        y: tile.y + aux.successor_split_len as i64,
                    };

                    if is_tile_rectangle_visible(&x_range, &y_range, *tile, successor_bound_tile) {
                        self.render_arrow_split(
                            view,
                            *tile,
                            aux.successor_x_offset as i64,
                            aux.successor_split_len,
                        );
                    }
                }
                TileType::IncomingReference(head_node_ids, tail_node_id) => {
                    if tile_visible {
                        self.render_reference(
                            view,
                            *tile,
                            head_node_ids.iter().copied(),
                            *tail_node_id,
                            false,
                        );
                        self.render_arrow_start(view, *tile, 1);
                    }
                }
                TileType::OutgoingReference(head_node_id, tail_node_id) => {
                    if tile_visible {
                        self.render_arrow_end(view, *tile);
                        self.render_reference(
                            view,
                            *tile,
                            std::iter::once(*head_node_id),
                            *tail_node_id,
                            true,
                        );
                    }
                }
            }
        }

        self.main_context.restore();
    }

    fn visible_tile_range(
        &self,
        view: &View,
    ) -> (std::ops::RangeInclusive<i64>, std::ops::RangeInclusive<i64>) {
        let lesser_visible_point = view.camera.view_offset();
        let greater_visible_point = lesser_visible_point
            + PixelPoint {
                x: self.main_canvas.width() as i64,
                y: self.main_canvas.height() as i64,
            };

        let lesser_tile = view.global_point_to_tile(lesser_visible_point, false);
        let greater_tile = view.global_point_to_tile(greater_visible_point, true);

        let x_range = lesser_tile.x..=greater_tile.x;
        let y_range = lesser_tile.y..=greater_tile.y;

        (x_range, y_range)
    }

    fn render_background(&self, view: &View) {
        self.main_context.save();

        self.main_context.set_fill_style_str("#FAFAFA");
        self.main_context.set_stroke_style_str("#DDD");

        let tile_height = view.camera.scheme.tile_size;

        let (range_x, range_y) = self.visible_tile_range(view);

        for tile_x in range_x {
            for tile_y in range_y.clone() {
                let x = view.column_start(tile_x);
                let tile_width = view.column_width(tile_x);
                /*if (tile_x as u64).wrapping_add(tile_y as u64) % 2 == 1 {
                    self.main_context.set_fill_style_str("#FFFFFF");
                } else {
                    self.main_context.set_fill_style_str("#FAFAFA");
                }*/

                let start = PixelPoint {
                    x,
                    y: tile_y * tile_height as i64,
                };

                self.main_context.begin_path();
                self.main_context.rect(
                    start.x as f64,
                    start.y as f64,
                    tile_width as f64,
                    tile_height as f64,
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

    fn adjust_view(&self, view: &mut View) {
        view.column_starts.clear();
        view.column_widths.clear();

        view.camera.view_size = PixelPoint {
            x: self.main_canvas.width() as i64,
            y: self.main_canvas.height() as i64,
        };

        let default_tile_width = view.camera.scheme.tile_size;

        for (tile, tile_type) in view.tiling.map_iter() {
            let (min_width, _min_height) = self.minimal_bounding_box(view, tile_type);

            let column_width = view
                .column_widths
                .entry(tile.x)
                .or_insert(default_tile_width);
            if min_width > *column_width {
                *column_width = min_width;
            }
        }
        //console_log!("Column widths: {:?}", view.column_widths);

        if let Some((last_width_column, _last_width)) = view.column_widths.last_key_value() {
            let mut start = 0;
            for column in 0..=*last_width_column {
                view.column_starts.insert(column, start);
                start += (*view
                    .column_widths
                    .get(&column)
                    .unwrap_or(&default_tile_width)) as i64;
            }
            view.column_starts.insert(*last_width_column + 1, start);
        }
        //console_log!("Column starts: {:?}", view.column_starts);
    }

    fn minimal_bounding_box(&self, view: &View, tile_type: &TileType) -> (u64, u64) {
        let text = match tile_type {
            TileType::Node(node_id) => Self::node_text(*node_id),
            TileType::IncomingReference(head_node_id, tail_node_id) => {
                Self::reference_text(head_node_id.iter().copied(), *tail_node_id)
            }
            TileType::OutgoingReference(head_node_id, tail_node_id) => {
                Self::reference_text(std::iter::once(*head_node_id), *tail_node_id)
            }
        };
        let text_metrics = self.main_context.measure_text(&text).unwrap();
        let width =
            text_metrics.actual_bounding_box_left() + text_metrics.actual_bounding_box_right();
        let height =
            text_metrics.actual_bounding_box_ascent() + text_metrics.actual_bounding_box_descent();

        let mut width = width;
        if matches!(
            tile_type,
            TileType::IncomingReference(_, _) | TileType::OutgoingReference(_, _)
        ) {
            width /= 0.75;
        } else {
            width /= 0.9;
        }

        width += (view.camera.scheme.tile_size - view.camera.scheme.node_size) as f64;

        let (width, height) = (width.ceil() as u64, height.ceil() as u64);

        (width, height)
    }
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

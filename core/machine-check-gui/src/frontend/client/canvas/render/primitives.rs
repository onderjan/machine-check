use machine_check_common::{NodeId, ParamValuation};
use web_sys::CanvasRenderingContext2d;

use crate::frontend::{
    tiling::Tile,
    util::constants,
    view::{camera::Scheme, View},
};

use super::CanvasRenderer;

impl CanvasRenderer {
    pub fn render_node(
        &self,
        view: &View,
        tile: Tile,
        node_id: NodeId,
        labelling: Option<ParamValuation>,
        is_selected: bool,
    ) {
        let context = &self.main_context;

        let node_rect = view.node_rect(tile);
        let radius = 4.;

        context.save();
        constants::setup_node_context(context, labelling, is_selected);

        context.begin_path();
        context
            .round_rect_with_f64(
                node_rect.left_x() as f64,
                node_rect.top_y() as f64,
                node_rect.width() as f64,
                node_rect.height() as f64,
                radius,
            )
            .unwrap();
        context.fill();
        context.stroke();

        context.restore();

        context
            .fill_text(
                &Scheme::node_text(node_id),
                node_rect.middle_x() as f64,
                node_rect.middle_y() as f64,
            )
            .unwrap();
    }

    pub fn render_reference(
        &self,
        view: &View,
        tile: Tile,
        head_node_ids: impl Iterator<Item = NodeId>,
        tail_node_id: NodeId,
        outgoing: bool,
    ) {
        let outgoing = if outgoing { 1 } else { -1 };
        let context = &self.main_context;

        let node_rect = view.node_rect(tile);

        let reference_half_width = node_rect.width() / 2;
        let reference_half_height = node_rect.height() / 3;
        let beak_size = reference_half_height;
        let reference_sharp_half_width = reference_half_width.saturating_sub(beak_size);

        let middle_x = node_rect.middle_x();
        let middle_y = node_rect.middle_y();
        let upper_y = middle_y - reference_half_height as i64;
        let lower_y = middle_y + reference_half_height as i64;
        let sharp_x = middle_x - outgoing * reference_sharp_half_width as i64;
        let sharper_x = middle_x - outgoing * reference_half_width as i64;
        let blunt_x = middle_x + outgoing * reference_half_width as i64;

        let points = [
            (blunt_x, upper_y),
            (sharp_x, upper_y),
            (sharper_x, middle_y),
            (sharp_x, lower_y),
            (blunt_x, lower_y),
        ];

        draw_primitive(context, Some(constants::colors::REFERENCE), &points);

        context
            .fill_text(
                &Scheme::reference_text(head_node_ids, tail_node_id),
                middle_x as f64
                    + (outgoing as f64
                        * ((beak_size as f64 / 2.) - view.scheme().font_margin / 2.)),
                middle_y as f64,
            )
            .unwrap();
    }

    pub fn render_arrow_start(&self, view: &View, head_tile: Tile, successor_x_offset: u64) {
        let successor_tile = Tile {
            x: head_tile.x + successor_x_offset as i64,
            y: head_tile.y,
        };

        let head_node_rect = view.node_rect(head_tile);
        let successor_tile_rect = view.tile_rect(successor_tile);

        // draw the arrowshaft
        let arrowshaft = [
            (head_node_rect.right_x(), head_node_rect.middle_y()),
            (successor_tile_rect.left_x(), head_node_rect.middle_y()),
        ];
        draw_simple(&self.main_context, &arrowshaft);
    }

    pub fn render_arrow_split(&self, view: &View, node_tile: Tile, x_offset: i64, split_len: u64) {
        let last_split_tile = Tile {
            x: node_tile.x + x_offset,
            y: node_tile.y + split_len as i64,
        };

        let node_rect = view.node_rect(node_tile);
        let last_split_tile_rect = view.tile_rect(last_split_tile);

        // draw the arrow split
        let arrow_split = [
            (last_split_tile_rect.left_x(), node_rect.middle_y()),
            (
                last_split_tile_rect.left_x(),
                last_split_tile_rect.middle_y(),
            ),
        ];
        draw_simple(&self.main_context, &arrow_split);
    }

    pub fn render_arrow_end(&self, view: &View, tail_tile: Tile) {
        let node_rect = view.node_rect(tail_tile);
        let tile_rect = view.tile_rect(tail_tile);

        let middle_y = node_rect.middle_y();

        // draw the arrowshaft
        let arrowshaft = [
            (tile_rect.left_x(), middle_y),
            (node_rect.left_x(), middle_y),
        ];
        draw_simple(&self.main_context, &arrowshaft);

        // draw the arrowhead
        let arrowhead_size = view.scheme().arrowhead_size;

        let arrowhead_right_x = node_rect.left_x();
        let arrowhead_left_x = node_rect.left_x() - arrowhead_size as i64;
        let arrowhead_upper_y = middle_y - arrowhead_size as i64 / 2;
        let arrowhead_lower_y = middle_y + arrowhead_size as i64 / 2;

        let arrowhead = [
            (arrowhead_right_x, middle_y),
            (arrowhead_left_x, arrowhead_upper_y),
            (arrowhead_left_x, arrowhead_lower_y),
        ];
        draw_primitive(
            &self.main_context,
            Some(constants::colors::ARROWHEAD),
            &arrowhead,
        );
    }

    pub fn render_self_loop(&self, view: &View, node_tile: Tile) {
        let next_tile = Tile {
            x: node_tile.x + 1,
            y: node_tile.y,
        };

        // the arrow start goes to the left side of the next tile
        let node_rect = view.node_rect(node_tile);
        let next_tile_rect = view.tile_rect(next_tile);

        let middle_x = node_rect.middle_x();

        let tile_top_y = next_tile_rect.top_y();
        let node_top_y = node_rect.top_y();
        let middle_y = node_rect.middle_y();

        // draw the arrowshaft that snakes around from right to top
        let arrowshaft = [
            (node_rect.right_x(), middle_y),
            (next_tile_rect.left_x(), middle_y),
            (next_tile_rect.left_x(), tile_top_y),
            (middle_x, tile_top_y),
            (node_rect.middle_x(), node_top_y),
        ];

        draw_simple(&self.main_context, &arrowshaft);

        // draw the arrowhead
        let arrowhead_size = view.scheme().arrowhead_size;

        let arrowhead_left_x = middle_x - arrowhead_size as i64 / 2;
        let arrowhead_right_x = middle_x + arrowhead_size as i64 / 2;
        let arrowhead_upper_y = node_top_y - arrowhead_size as i64;

        let arrowhead = [
            (middle_x, node_top_y),
            (arrowhead_left_x, arrowhead_upper_y),
            (arrowhead_right_x, arrowhead_upper_y),
        ];
        draw_primitive(
            &self.main_context,
            Some(constants::colors::ARROWHEAD),
            &arrowhead,
        );
    }
}

fn draw_primitive(
    context: &CanvasRenderingContext2d,
    fill_style: Option<&str>,
    points: &[(i64, i64)],
) {
    let filling = if let Some(fill_style) = fill_style {
        context.save();
        context.set_fill_style_str(fill_style);
        true
    } else {
        false
    };

    context.begin_path();
    for (x, y) in points {
        context.line_to(*x as f64, *y as f64);
    }
    if filling {
        context.close_path();
        context.fill();
    }
    context.stroke();

    if filling {
        context.restore();
    }
}

fn draw_simple(context: &CanvasRenderingContext2d, points: &[(i64, i64)]) {
    draw_primitive(context, None, points);
}

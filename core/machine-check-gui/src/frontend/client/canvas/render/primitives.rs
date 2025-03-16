use machine_check_common::ThreeValued;
use machine_check_exec::NodeId;

use crate::frontend::{
    util::constants,
    view::{Tile, View},
};

use super::CanvasRenderer;

impl CanvasRenderer {
    pub fn render_node(
        &self,
        view: &View,
        tile: Tile,
        node_id: NodeId,
        labelling: Option<ThreeValued>,
        is_selected: bool,
    ) {
        let context = &self.main_context;

        let scheme = &view.camera.scheme;

        let (tile_height, node_height) = (scheme.tile_size as f64, scheme.node_size as f64);
        let node_half_margin = scheme.node_half_margin();

        let tile_width = self.column_width(view, tile.x) as f64;

        let node_width = tile_width - scheme.node_margin();

        let node_start_x = self.column_start(view, tile.x) as f64 + node_half_margin;
        let node_start_y = tile.y as f64 * tile_height + node_half_margin;

        let radius = 4.;

        context.save();
        constants::setup_node_context(context, labelling, is_selected);

        context.begin_path();
        context
            .round_rect_with_f64(node_start_x, node_start_y, node_width, node_height, radius)
            .unwrap();
        context.fill();
        context.stroke();

        context.restore();

        context
            .fill_text(
                &Self::node_text(node_id),
                node_start_x + node_width / 2.,
                node_start_y + node_height / 2.,
            )
            .unwrap();
    }

    pub fn node_text(node_id: NodeId) -> String {
        node_id.to_string()
    }

    pub fn render_reference(
        &self,
        view: &View,
        tile: Tile,
        head_node_id: NodeId,
        tail_node_id: NodeId,
        outgoing: bool,
    ) {
        let scheme = &view.camera.scheme;

        let outgoing = if outgoing { 1. } else { -1. };
        let context = &self.main_context;

        let (tile_height, node_height) = (scheme.tile_size as f64, scheme.node_size as f64);

        let tile_width = self.column_width(view, tile.x) as f64;
        let node_width = tile_width - scheme.node_margin();
        let start_x = self.column_start(view, tile.x) as f64;

        let middle_x = start_x + tile_width / 2.;
        let middle_y = tile.y as f64 * tile_height + tile_height / 2.;
        let upper_y = (middle_y - node_height / 3.).round();
        let lower_y = (middle_y + node_height / 3.).round();
        let sharp_x = middle_x - outgoing * (node_width / 4.);
        let sharper_x = middle_x - outgoing * (node_width / 2.);
        let blunt_x = middle_x + outgoing * (node_width / 2.);

        context.save();
        context.set_fill_style_str(constants::colors::REFERENCE);

        context.begin_path();
        context.move_to(blunt_x, upper_y);
        context.line_to(sharp_x, upper_y);
        context.line_to(sharper_x, middle_y);
        context.line_to(sharp_x, lower_y);
        context.line_to(blunt_x, lower_y);
        context.close_path();
        context.fill();
        context.stroke();

        context.restore();

        context
            .fill_text(
                &Self::reference_text(head_node_id, tail_node_id),
                middle_x + outgoing * (node_height / 12.),
                middle_y,
            )
            .unwrap();
    }

    pub fn reference_text(head_node_id: NodeId, tail_node_id: NodeId) -> String {
        format!("{}|{}", head_node_id, tail_node_id)
    }

    pub fn render_arrow_start(&self, view: &View, head_tile: Tile, successor_x_offset: u64) {
        let context = &self.main_context;
        let scheme = &view.camera.scheme;

        let tile_height = scheme.tile_size as f64;

        // draw the arrowshaft
        context.begin_path();
        let right_x = self.column_start(view, head_tile.x + successor_x_offset as i64) as f64;
        let tile_right_border_x =
            self.column_start(view, head_tile.x + 1) as f64 - scheme.node_half_margin();
        let tile_middle_y = head_tile.y as f64 * tile_height + tile_height / 2.;
        context.move_to(tile_right_border_x, tile_middle_y);
        context.line_to(right_x, tile_middle_y);
        context.stroke();
    }

    pub fn render_arrow_split(&self, view: &View, node_tile: Tile, x_offset: i64, split_len: u64) {
        let context = &self.main_context;
        let scheme = &view.camera.scheme;

        let tile_size = scheme.tile_size as f64;

        // draw the arrow split
        context.begin_path();
        let split_x = self.column_start(view, node_tile.x + x_offset) as f64;
        let split_upper_y = node_tile.y as f64 * tile_size + tile_size / 2.;
        let split_lower_y = split_upper_y + split_len as f64 * tile_size;
        context.move_to(split_x, split_upper_y);
        context.line_to(split_x, split_lower_y);
        context.stroke();
    }

    pub fn render_arrow_end(&self, view: &View, tail_tile: Tile) {
        let context = &self.main_context;
        let scheme = &view.camera.scheme;

        let tile_height = scheme.tile_size as f64;

        // draw the arrowshaft
        context.begin_path();
        let tile_left_x = self.column_start(view, tail_tile.x) as f64;
        let tile_left_border_x = tile_left_x + scheme.node_half_margin();
        let tile_middle_y = tail_tile.y as f64 * tile_height + tile_height / 2.;
        context.move_to(tile_left_x, tile_middle_y);
        context.line_to(tile_left_border_x, tile_middle_y);
        context.stroke();

        // draw the arrowhead
        let arrowhead_size = scheme.arrowhead_size;

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

    pub fn render_self_loop(&self, view: &View, node_tile: Tile) {
        let context = &self.main_context;
        let scheme = &view.camera.scheme;

        let (tile_height, node_height) = (scheme.tile_size as f64, scheme.node_size as f64);

        let tile_start_x = self.column_start(view, node_tile.x) as f64;
        let tile_width = self.column_width(view, node_tile.x) as f64;

        // draw the arrowshaft
        context.begin_path();
        let tile_right_x = tile_start_x + tile_width;
        let tile_middle_x = tile_start_x + tile_width / 2.;
        let tile_middle_y = node_tile.y as f64 * tile_height + tile_height / 2.;
        let tile_upper_y = node_tile.y as f64 * tile_height;
        let tile_upper_border_y = tile_upper_y + (tile_height - node_height) / 2.;
        context.move_to(tile_right_x, tile_middle_y);
        context.line_to(tile_right_x, tile_upper_y);
        context.line_to(tile_middle_x, tile_upper_y);
        context.line_to(tile_middle_x, tile_upper_border_y);
        context.stroke();

        // draw the arrowhead
        let arrowhead_size = scheme.arrowhead_size;

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
}

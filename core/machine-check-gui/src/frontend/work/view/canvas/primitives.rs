use machine_check_exec::NodeId;

use crate::frontend::{
    snapshot::Node,
    work::view::{constants, Tile},
};

use super::CanvasRenderer;

impl CanvasRenderer<'_> {
    pub fn render_node(&self, tile: Tile, node_id: NodeId, node: &Node) {
        let context = &self.main_context;

        let is_selected = if let Some(selected_node_id) = self.view.camera.selected_node_id {
            selected_node_id == node_id
        } else {
            false
        };

        let scheme = &self.view.camera.scheme;
        let (tile_size, node_size) = (scheme.tile_size as f64, scheme.node_size as f64);

        let node_start_x = tile.x as f64 * tile_size + (tile_size - node_size) / 2.;
        let node_start_y = tile.y as f64 * tile_size + (tile_size - node_size) / 2.;

        let radius = 4.;

        context.save();
        constants::setup_node_context(context, node, is_selected);

        context.begin_path();
        context
            .round_rect_with_f64(node_start_x, node_start_y, node_size, node_size, radius)
            .unwrap();
        context.fill();
        context.stroke();

        context.restore();

        context
            .fill_text(
                &node_id.to_string(),
                node_start_x + node_size / 2.,
                node_start_y + node_size / 2.,
            )
            .unwrap();
    }

    pub fn render_reference(
        &self,
        tile: Tile,
        head_node_id: NodeId,
        tail_node_id: NodeId,
        outgoing: bool,
    ) {
        let outgoing = if outgoing { 1. } else { -1. };
        let context = &self.main_context;

        let scheme = &self.view.camera.scheme;
        let (tile_size, node_size) = (scheme.tile_size as f64, scheme.node_size as f64);

        let middle_x = tile.x as f64 * tile_size + tile_size / 2.;
        let middle_y = tile.y as f64 * tile_size + tile_size / 2.;
        let upper_y = (middle_y - node_size / 3.).round();
        let lower_y = (middle_y + node_size / 3.).round();
        let sharp_x = middle_x - outgoing * (node_size / 4.);
        let sharper_x = middle_x - outgoing * (node_size / 2.);
        let blunt_x = middle_x + outgoing * (node_size / 2.);

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
                &format!("{}|{}", head_node_id, tail_node_id),
                middle_x + outgoing * (node_size / 12.),
                middle_y,
            )
            .unwrap();
    }

    pub fn render_arrow_start(&self, head_tile: Tile, successor_x_offset: u64) {
        let context = &self.main_context;

        let scheme = &self.view.camera.scheme;
        let (tile_size, node_size) = (scheme.tile_size as f64, scheme.node_size as f64);

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

    pub fn render_arrow_split(&self, node_tile: Tile, x_offset: i64, split_len: u64) {
        let context = &self.main_context;

        let scheme = &self.view.camera.scheme;
        let tile_size = scheme.tile_size as f64;

        // draw the arrow split
        context.begin_path();
        let split_x = node_tile.x as f64 * tile_size + tile_size * x_offset as f64;
        let split_upper_y = node_tile.y as f64 * tile_size + tile_size / 2.;
        let split_lower_y = split_upper_y + split_len as f64 * tile_size;
        context.move_to(split_x, split_upper_y);
        context.line_to(split_x, split_lower_y);
        context.stroke();
    }

    pub fn render_arrow_end(&self, tail_tile: Tile) {
        let context = &self.main_context;

        let scheme = &self.view.camera.scheme;
        let (tile_size, node_size) = (scheme.tile_size as f64, scheme.node_size as f64);

        // draw the arrowshaft
        context.begin_path();
        let tile_left_x = tail_tile.x as f64 * tile_size;
        let tile_left_border_x = tile_left_x + (tile_size - node_size) / 2.;
        let tile_middle_y = tail_tile.y as f64 * tile_size + tile_size / 2.;
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

    pub fn render_self_loop(&self, node_tile: Tile) {
        let context = &self.main_context;

        let scheme = &self.view.camera.scheme;
        let (tile_size, node_size) = (scheme.tile_size as f64, scheme.node_size as f64);

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
        let scheme = &self.view.camera.scheme;
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

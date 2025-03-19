use machine_check_common::ThreeValued;
use web_sys::CanvasRenderingContext2d;

pub const RAW_TILE_SIZE: f64 = 46.;
pub const RAW_NODE_SIZE: f64 = 30.;
pub const RAW_ARROWHEAD_SIZE: f64 = 4.;
pub const RAW_FONT_SIZE: f64 = 12.;
pub const RAW_FONT_MARGIN: f64 = 4.;

pub mod colors {
    pub const UNKNOWN: &str = "#CCCCCC"; // grey
    pub const TRUE: &str = "#4CBF50"; // green
    pub const FALSE: &str = "#CC2222"; // red
    pub const NOT_APPLICABLE: &str = "lightblue"; // light blue
    pub const REFERENCE: &str = "#F5F5DC"; // light yellow
    pub const ARROWHEAD: &str = "#000"; // black
}

pub const NODE_LINE_WIDTH_SELECTED: f64 = 3.;
pub const NODE_LINE_WIDTH_UNSELECTED: f64 = 1.;

pub fn labelling_color(value: ThreeValued) -> &'static str {
    match value {
        ThreeValued::Unknown => colors::UNKNOWN,
        ThreeValued::True => colors::TRUE,
        ThreeValued::False => colors::FALSE,
    }
}

pub fn setup_node_context(
    context: &CanvasRenderingContext2d,
    labelling: Option<ThreeValued>,
    is_selected: bool,
) {
    let node_color = match labelling {
        None => colors::NOT_APPLICABLE,
        Some(labelling) => labelling_color(labelling),
    };
    context.set_fill_style_str(node_color);
    context.set_line_width(if is_selected {
        NODE_LINE_WIDTH_SELECTED
    } else {
        NODE_LINE_WIDTH_UNSELECTED
    });
}

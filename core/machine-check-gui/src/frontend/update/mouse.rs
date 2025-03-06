use wasm_bindgen::JsCast;

use crate::frontend::{update::view::TileType, util::PixelPoint, MouseEvent};

use super::{view::View, PointOfView};

pub fn on_mouse(
    view: &View,
    point_of_view: &mut PointOfView,
    mouse: MouseEvent,
    event: web_sys::Event,
) -> bool {
    event.prevent_default();
    let event: web_sys::MouseEvent = event.dyn_into().expect("Mouse event should be MouseEvent");
    let device_pixel_ratio = web_sys::window().unwrap().device_pixel_ratio();
    let mouse_coords = PixelPoint {
        x: (event.offset_x() as f64 * device_pixel_ratio).round() as i64,
        y: (event.offset_y() as f64 * device_pixel_ratio).round() as i64,
    };

    const MAIN_BUTTON: i16 = 0;
    const MIDDLE_BUTTON: i16 = 1;

    match mouse {
        MouseEvent::Click => {
            if event.button() == MAIN_BUTTON {
                let absolute_coords = point_of_view.view_offset() + mouse_coords;

                let tile = super::render::get_tile_from_px(absolute_coords);
                let mut selected_node_id = None;
                if let Some(tile) = tile {
                    if let Some(TileType::Node(node)) = view.tiling.get_by_left(&tile) {
                        selected_node_id = Some(*node);
                    }
                }
                point_of_view.selected_node_id = selected_node_id;
                true
            } else {
                false
            }
        }
        MouseEvent::ContextMenu => {
            // do nothing for now
            false
        }
        MouseEvent::Down => {
            if event.button() == MIDDLE_BUTTON {
                point_of_view.mouse_current_coords = None;
                point_of_view.mouse_down_coords = Some(mouse_coords);
                true
            } else {
                false
            }
        }
        MouseEvent::Move => {
            point_of_view.mouse_current_coords = Some(mouse_coords);
            true
        }
        MouseEvent::Up | MouseEvent::Out => {
            if event.button() == MIDDLE_BUTTON {
                if let Some(mouse_down_coords) = point_of_view.mouse_down_coords.take() {
                    let offset = mouse_coords - mouse_down_coords;
                    point_of_view.view_offset -= offset;
                };
                true
            } else {
                false
            }
        }
    }
}

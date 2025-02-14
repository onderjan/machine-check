use wasm_bindgen::JsCast;

use crate::frontend::{update::view::TileType, MouseEvent};

use super::{view::View, PointOfView};

pub fn on_mouse(
    view: &View,
    point_of_view: &mut PointOfView,
    mouse: MouseEvent,
    event: web_sys::Event,
) -> bool {
    event.prevent_default();
    let event: web_sys::MouseEvent = event.dyn_into().expect("Mouse event should be MouseEvent");
    let mouse_coords_px = (event.offset_x(), event.offset_y());

    const MAIN_BUTTON: i16 = 0;
    const MIDDLE_BUTTON: i16 = 1;

    match mouse {
        MouseEvent::Click => {
            if event.button() == MAIN_BUTTON {
                let mut absolute_px = point_of_view.translation();
                absolute_px.0 += mouse_coords_px.0 as f64;
                absolute_px.1 += mouse_coords_px.1 as f64;

                let tile = super::render::get_tile_from_px(absolute_px.0, absolute_px.1);
                let mut selected_node_id = None;
                if let Some(tile) = tile {
                    if let Some(TileType::Node(node)) = view.tiling.get_by_left(&tile) {
                        selected_node_id = Some(node.clone());
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
                point_of_view.mouse_current_px = None;
                point_of_view.mouse_down_px = Some(mouse_coords_px);
                true
            } else {
                false
            }
        }
        MouseEvent::Move => {
            point_of_view.mouse_current_px = Some(mouse_coords_px);
            true
        }
        MouseEvent::Up | MouseEvent::Out => {
            if event.button() == MIDDLE_BUTTON {
                if let Some(mouse_down_px) = point_of_view.mouse_down_px.take() {
                    let offset = (
                        mouse_coords_px.0.wrapping_sub(mouse_down_px.0),
                        mouse_coords_px.1.wrapping_sub(mouse_down_px.1),
                    );
                    point_of_view.translation_px.0 += offset.0 as f64;
                    point_of_view.translation_px.1 += offset.1 as f64;
                };
                true
            } else {
                false
            }
        }
    }
}

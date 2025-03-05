use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Array;

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
    let device_pixel_ratio = web_sys::window().unwrap().device_pixel_ratio();
    let mouse_coords_px = (
        event.offset_x() as f64 * device_pixel_ratio,
        event.offset_y() as f64 * device_pixel_ratio,
    );

    const MAIN_BUTTON: i16 = 0;
    const MIDDLE_BUTTON: i16 = 1;

    match mouse {
        MouseEvent::Click => {
            if event.button() == MAIN_BUTTON {
                let mut absolute_px = (mouse_coords_px.0 as f64, mouse_coords_px.1 as f64);

                let cons = Array::new_with_length(1);
                cons.set(
                    0,
                    JsValue::from_str(&format!(
                        "Translation: {:?}, Click: {:?}, Page: {:?}",
                        point_of_view.view_offset(),
                        mouse_coords_px,
                        (event.page_x(), event.page_y())
                    )),
                );
                web_sys::console::log(&cons);

                absolute_px.0 += point_of_view.view_offset().0;
                absolute_px.1 += point_of_view.view_offset().1;

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
                        mouse_coords_px.0 - mouse_down_px.0,
                        mouse_coords_px.1 - mouse_down_px.1,
                    );
                    point_of_view.offset_px.0 -= offset.0 as f64;
                    point_of_view.offset_px.1 -= offset.1 as f64;
                };
                true
            } else {
                false
            }
        }
    }
}

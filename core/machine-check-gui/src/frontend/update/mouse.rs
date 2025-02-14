use wasm_bindgen::JsCast;

use crate::frontend::MouseEvent;

use super::PointOfView;

pub fn on_mouse(point_of_view: &mut PointOfView, mouse: MouseEvent, event: web_sys::Event) -> bool {
    event.prevent_default();
    let event: web_sys::MouseEvent = event.dyn_into().expect("Mouse event should be MouseEvent");
    let mouse_coords_px = (event.offset_x(), event.offset_y());

    const MIDDLE_BUTTON: i16 = 1;

    match mouse {
        MouseEvent::Click => {
            // TODO
            false
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

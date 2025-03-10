use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::frontend::{
    get_element_by_id,
    util::PixelPoint,
    window,
    work::view::{TileType, View},
};

#[derive(Clone, Copy, Debug)]
pub enum MouseEvent {
    Click,
    ContextMenu,
    Down,
    Move,
    Up,
    Out,
}

pub fn on_mouse(view: &mut View, mouse: MouseEvent, event: web_sys::Event) -> bool {
    event.prevent_default();
    let event: web_sys::MouseEvent = event.dyn_into().expect("Mouse event should be MouseEvent");
    let device_pixel_ratio = window().device_pixel_ratio();
    let mouse_coords = PixelPoint {
        x: (event.offset_x() as f64 * device_pixel_ratio).round() as i64,
        y: (event.offset_y() as f64 * device_pixel_ratio).round() as i64,
    };

    /// Typically the left button.
    const MAIN_BUTTON: i16 = 0;
    /// Typically the middle button.
    const DRAG_BUTTON: i16 = 1;

    match mouse {
        MouseEvent::Click => {
            // focus the main area
            let main_area: HtmlElement = get_element_by_id("main_area").dyn_into().unwrap();
            main_area.focus().unwrap();

            if event.button() == MAIN_BUTTON {
                // select a tile
                let tile = view.camera.viewport_px_tile(mouse_coords);
                let mut selected_node_id = None;
                if let Some(tile) = tile {
                    if let Some(TileType::Node(node)) = view.tiling.get_by_left(&tile) {
                        selected_node_id = Some(*node);
                    }
                }
                view.camera.selected_node_id = selected_node_id;

                // redraw
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
            if event.button() == DRAG_BUTTON {
                // start dragging the camera
                view.camera.mouse_current_coords = Some(mouse_coords);
                view.camera.mouse_down_coords = Some(mouse_coords);
            }

            // redraw
            true
        }
        MouseEvent::Move => {
            // update current mouse coords for camera drag
            view.camera.mouse_current_coords = Some(mouse_coords);

            // redraw if dragging
            view.camera.mouse_down_coords.is_some()
        }
        MouseEvent::Up | MouseEvent::Out => {
            if event.button() == DRAG_BUTTON {
                // finish dragging the camera
                if let Some(mouse_down_coords) = view.camera.mouse_down_coords.take() {
                    let offset = mouse_coords - mouse_down_coords;
                    view.camera.view_offset -= offset;

                    // redraw
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

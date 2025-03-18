use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::frontend::{
    client::{lock_view, render},
    util::{
        web_idl::{get_element_by_id, setup_selector_listener, window},
        PixelPoint,
    },
    view::TileType,
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

pub fn init() {
    for (event_type, event_name) in [
        (MouseEvent::Click, "click"),
        (MouseEvent::ContextMenu, "contextmenu"),
        (MouseEvent::Down, "mousedown"),
        (MouseEvent::Move, "mousemove"),
        (MouseEvent::Up, "mouseup"),
        (MouseEvent::Out, "mouseout"),
    ] {
        setup_selector_listener(
            "#main_area",
            event_name,
            Box::new(move |e| {
                wasm_bindgen_futures::spawn_local(on_mouse(event_type, e));
            }),
        );
    }
}

pub async fn on_mouse(mouse: MouseEvent, event: web_sys::Event) {
    // since this is on the canvas, we will always handle it
    event.prevent_default();
    let event: web_sys::MouseEvent = event.dyn_into().expect("Mouse event should be MouseEvent");
    let device_pixel_ratio = window().device_pixel_ratio();
    let mouse_coords = PixelPoint {
        x: (event.offset_x() as f64 * device_pixel_ratio).round() as i64,
        y: (event.offset_y() as f64 * device_pixel_ratio).round() as i64,
    };

    // lock the view
    let mut view_guard = lock_view();
    let view = view_guard.as_mut();

    match mouse {
        MouseEvent::Click => {
            // focus the main area
            let main_area: HtmlElement = get_element_by_id("main_area").dyn_into().unwrap();
            main_area.focus().unwrap();

            if event.button() == MAIN_BUTTON {
                // select a tile
                let tile = view.viewport_point_to_tile(mouse_coords, false);
                let mut selected_node_id = None;
                if let Some(TileType::Node(node)) = view.tiling.get_by_left(&tile) {
                    selected_node_id = Some(*node);
                }
                view.camera.selected_node_id = selected_node_id;

                // redraw

                render(view);
            }
        }
        MouseEvent::ContextMenu => {
            // do nothing for now
        }
        MouseEvent::Down => {
            if is_drag(&event) {
                // start dragging the camera
                view.camera.mouse_current_coords = Some(mouse_coords);
                view.camera.mouse_down_coords = Some(mouse_coords);
                // redraw
                render(view);
            }
        }
        MouseEvent::Move => {
            // update current mouse coords for camera drag
            view.camera.mouse_current_coords = Some(mouse_coords);

            // redraw if dragging
            if view.camera.mouse_down_coords.is_some() {
                render(view);
            }
        }
        MouseEvent::Up | MouseEvent::Out => {
            if matches!(mouse, MouseEvent::Out) || is_drag(&event) {
                // finish dragging the camera
                if let Some(mouse_down_coords) = view.camera.mouse_down_coords.take() {
                    let offset = mouse_coords - mouse_down_coords;
                    view.camera.view_offset -= offset;

                    // redraw
                    render(view);
                }
            }
        }
    }
}

/// Typically the left button.
const MAIN_BUTTON: i16 = 0;
/// Typically the middle button.
const AUX_BUTTON: i16 = 1;
/// Typically the right button.
const CONTEXT_BUTTON: i16 = 2;

fn is_drag(event: &web_sys::MouseEvent) -> bool {
    let button = event.button();
    button == AUX_BUTTON || button == CONTEXT_BUTTON
}

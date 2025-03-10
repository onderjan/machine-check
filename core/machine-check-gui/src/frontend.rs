#[macro_export]
macro_rules! console_log {
    ($a: expr) => {
        let a = $a;
        let cons = ::web_sys::js_sys::Array::new_with_length(1);
        cons.set(0, ::wasm_bindgen::JsValue::from_str(a));
        ::web_sys::console::log(&cons);
    };
}

pub mod interaction;
pub mod snapshot;

mod util;
mod work;

use wasm_bindgen::prelude::*;
use web_sys::Event;
use work::input::{keyboard::KeyboardEvent, mouse::MouseEvent};

use util::web_idl::{
    create_element, document, get_element_by_id, setup_element_listener, setup_interval,
    setup_selector_listener, setup_window_listener, window,
};

#[wasm_bindgen]
pub async fn exec() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    setup_listeners();

    work::init().await;
}

pub async fn resize() {
    // force complete re-render
    work::render(true);
}

async fn on_keyboard(keyboard: KeyboardEvent, event: Event) {
    work::on_keyboard(keyboard, event);
}

async fn on_mouse(mouse: MouseEvent, event: Event) {
    work::on_mouse(mouse, event);
}

fn setup_listeners() {
    setup_window_listener(
        "resize",
        Box::new(|_e| {
            wasm_bindgen_futures::spawn_local(resize());
        }),
    );

    for (event_type, event_name) in [
        (MouseEvent::Click, "click"),
        (MouseEvent::ContextMenu, "contextmenu"),
        (MouseEvent::Down, "mousedown"),
        (MouseEvent::Move, "mousemove"),
        (MouseEvent::Up, "mouseup"),
        (MouseEvent::Out, "mouseout"),
    ] {
        setup_selector_listener(
            "#main_canvas",
            event_name,
            Box::new(move |e| {
                wasm_bindgen_futures::spawn_local(on_mouse(event_type, e));
            }),
        );
    }

    for (event_type, event_name) in [
        (KeyboardEvent::Down, "keydown"),
        (KeyboardEvent::Up, "keyup"),
    ] {
        setup_selector_listener(
            "#main_area",
            event_name,
            Box::new(move |e| {
                wasm_bindgen_futures::spawn_local(on_keyboard(event_type, e));
            }),
        );
        setup_selector_listener(
            "#main_canvas",
            event_name,
            Box::new(move |e| {
                wasm_bindgen_futures::spawn_local(on_keyboard(event_type, e));
            }),
        );
    }

    setup_interval(
        Box::new(move |_| {
            wasm_bindgen_futures::spawn_local(work::tick());
        }),
        10,
    );
}

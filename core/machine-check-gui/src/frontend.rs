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
use web_sys::{Document, Element, Event, Window};
use work::input::{keyboard::KeyboardEvent, mouse::MouseEvent};

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

fn setup_selector_listener(selector: &str, event_name: &str, func: Box<dyn FnMut(web_sys::Event)>) {
    let element = document()
        .query_selector(selector)
        .expect("Selector should succeed")
        .expect("Selector should select some element");

    setup_element_listener(&element, event_name, func);
}

fn setup_window_listener(ty: &str, func: Box<dyn FnMut(web_sys::Event)>) {
    let closure = Closure::wrap(func);

    window()
        .add_event_listener_with_callback(ty, closure.as_ref().dyn_ref().unwrap())
        .expect("Adding a listener should succeed");
    // the closure must be explicitely forgotten so it remains accessible
    closure.forget();
}

fn setup_element_listener(
    element: &Element,
    event_name: &str,
    func: Box<dyn FnMut(web_sys::Event)>,
) {
    let closure = Closure::wrap(func);
    element
        .add_event_listener_with_callback(event_name, closure.as_ref().dyn_ref().unwrap())
        .expect("Adding a listener should succeed");
    // the closure must be explicitely forgotten so it remains accessible
    closure.forget();
}

fn setup_interval(func: Box<dyn FnMut(web_sys::Event)>, interval: i32) {
    let closure = Closure::wrap(func);
    let handler = closure.as_ref().dyn_ref().unwrap();
    window()
        .set_interval_with_callback_and_timeout_and_arguments_0(handler, interval)
        .unwrap();
    // the closure must be explicitely forgotten so it remains accessible
    closure.forget();
}

fn window() -> Window {
    web_sys::window().expect("HTML Window should exist")
}

fn document() -> Document {
    window().document().expect("HTML document should exist")
}

fn get_element_by_id(element_id: &str) -> Element {
    let element = document().get_element_by_id(element_id);
    match element {
        Some(element) => element,
        None => panic!("Element '{}' should exist", element_id),
    }
}

fn create_element(local_name: &str) -> Element {
    let element = document().create_element(local_name);
    match element {
        Ok(element) => element,
        Err(err) => panic!("Element '{}' could not be created: {:?}", local_name, err),
    }
}

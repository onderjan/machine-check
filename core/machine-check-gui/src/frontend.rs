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
use web_sys::{Element, Event, HtmlInputElement};
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

pub async fn reset() {
    work::reset().await;
}

pub async fn step() {
    let input: HtmlInputElement = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("max_refinements")
        .expect("The number of steps element should exist")
        .dyn_into()
        .expect("The number of steps element should be an input");

    let max_refinements = (input.value_as_number() as u64).max(1);

    work::step(Some(max_refinements)).await;
}

pub async fn run() {
    work::step(None).await;
}

async fn on_keyboard(keyboard: KeyboardEvent, event: Event) {
    work::on_keyboard(keyboard, event);
}

async fn on_mouse(mouse: MouseEvent, event: Event) {
    work::on_mouse(mouse, event);
}

fn setup_listeners() {
    setup_selector_listener(
        "#reset",
        "click",
        Box::new(|_e| {
            wasm_bindgen_futures::spawn_local(reset());
        }),
    );

    setup_selector_listener(
        "#step",
        "click",
        Box::new(|_e| {
            wasm_bindgen_futures::spawn_local(step());
        }),
    );

    setup_selector_listener(
        "#run",
        "click",
        Box::new(|_e| {
            wasm_bindgen_futures::spawn_local(run());
        }),
    );

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
}

fn setup_selector_listener(selector: &str, event_name: &str, func: Box<dyn FnMut(web_sys::Event)>) {
    let window = web_sys::window().expect("HTML Window should exist");
    let document = window.document().expect("HTML document should exist");
    let element = document
        .query_selector(selector)
        .expect("Selector should succeed")
        .expect("Selector should select some element");

    setup_element_listener(&element, event_name, func);
}

fn setup_window_listener(ty: &str, func: Box<dyn FnMut(web_sys::Event)>) {
    let closure = Closure::wrap(func);

    let window = web_sys::window().expect("HTML Window should exist");
    window
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

pub mod content;
mod update;

use wasm_bindgen::prelude::*;
use web_sys::Event;

#[wasm_bindgen]
pub async fn exec() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    setup_listeners();

    update::display(true).await;
}

pub async fn resize() {
    update::display(true).await;
}

pub async fn step_verification() {
    update::update(update::Action::Step, false).await;
}

#[derive(Clone, Copy, Debug)]
enum MouseEvent {
    Click,
    ContextMenu,
    Down,
    Move,
    Up,
    Out,
}

async fn on_mouse(mouse: MouseEvent, event: Event) {
    update::on_mouse(mouse, event).await;
}

fn setup_listeners() {
    setup_element_listener(
        "#step_verification",
        "click",
        Box::new(|_e| {
            wasm_bindgen_futures::spawn_local(step_verification());
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
        setup_element_listener(
            "#main_canvas",
            event_name,
            Box::new(move |e| {
                wasm_bindgen_futures::spawn_local(on_mouse(event_type, e));
            }),
        );
    }
}

fn setup_element_listener(selector: &str, ty: &str, func: Box<dyn FnMut(web_sys::Event)>) {
    let closure = Closure::wrap(func);

    let window = web_sys::window().expect("HTML Window should exist");
    let document = window.document().expect("HTML document should exist");
    document
        .query_selector(selector)
        .expect("Selector should succeed")
        .expect("Selector should select")
        .add_event_listener_with_callback(ty, closure.as_ref().unchecked_ref())
        .expect("Adding a listener should succeed");
    // the closure must be explicitely forgotten so it remains accessible
    closure.forget();
}

fn setup_window_listener(ty: &str, func: Box<dyn FnMut(web_sys::Event)>) {
    let closure = Closure::wrap(func);

    let window = web_sys::window().expect("HTML Window should exist");
    window
        .add_event_listener_with_callback(ty, closure.as_ref().unchecked_ref())
        .expect("Adding a listener should succeed");
    // the closure must be explicitely forgotten so it remains accessible
    closure.forget();
}

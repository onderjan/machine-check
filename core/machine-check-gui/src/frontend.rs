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

use interaction::{Request, StepSettings};
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlInputElement};

#[wasm_bindgen]
pub async fn exec() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    setup_listeners();

    work::command(Request::GetContent, true).await;
}

pub async fn resize() {
    // force complete re-render
    work::render(true);
}

pub async fn reset() {
    work::command(Request::Reset, false).await;
}

pub async fn step() {
    let input: HtmlInputElement = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("num_steps")
        .expect("The number of steps element should exist")
        .dyn_into()
        .expect("The number of steps element should be an input");

    let num_steps = (input.value_as_number() as u64).max(1);

    work::command(
        Request::Step(StepSettings {
            num_steps: Some(num_steps),
        }),
        false,
    )
    .await;
}

pub async fn run() {
    work::command(Request::Step(StepSettings { num_steps: None }), false).await;
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
    work::on_mouse(mouse, event);
}

fn setup_listeners() {
    setup_element_listener(
        "#reset",
        "click",
        Box::new(|_e| {
            wasm_bindgen_futures::spawn_local(reset());
        }),
    );

    setup_element_listener(
        "#step",
        "click",
        Box::new(|_e| {
            wasm_bindgen_futures::spawn_local(step());
        }),
    );

    setup_element_listener(
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

pub mod content;
mod local;
mod update;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn exec() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    setup_listeners();

    update::render(true).await;
}

pub async fn resize() {
    let cons = web_sys::js_sys::Array::new_with_length(1);
    cons.set(0, JsValue::from_str("resize"));
    web_sys::console::log(&cons);
    update::render(true).await;
}

pub async fn step_verification() {
    update::update(update::Action::Step, false).await;
}

fn setup_listeners() {
    setup_element_listener("#step_verification", "click", |_e| {
        wasm_bindgen_futures::spawn_local(step_verification());
    });

    setup_window_listener("resize", |_e| {
        wasm_bindgen_futures::spawn_local(resize());
    });
}

fn setup_element_listener(selector: &str, ty: &str, func: fn(web_sys::Event)) {
    let closure = Closure::wrap(Box::new(func) as Box<dyn FnMut(_)>);

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

fn setup_window_listener(ty: &str, func: fn(web_sys::Event)) {
    let closure = Closure::wrap(Box::new(func) as Box<dyn FnMut(_)>);

    let window = web_sys::window().expect("HTML Window should exist");
    window
        .add_event_listener_with_callback(ty, closure.as_ref().unchecked_ref())
        .expect("Adding a listener should succeed");
    // the closure must be explicitely forgotten so it remains accessible
    closure.forget();
}

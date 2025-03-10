use wasm_bindgen::{prelude::Closure, JsCast};

use crate::frontend::util::web_idl::get_element_by_id;

mod render;

pub use render::render;

use super::lock_view;

pub fn setup() {
    render::setup();
}

pub fn init() {
    let func: Box<dyn FnMut(web_sys::Event)> = Box::new(|_| {
        wasm_bindgen_futures::spawn_local(on_resize());
    });
    let closure = Closure::wrap(func);
    let callback = closure.as_ref().dyn_ref().unwrap();

    let resize_observer = web_sys::ResizeObserver::new(callback)
        .expect("Canvas resize observer should be constructable");

    // the closure must be explicitely forgotten so it remains accessible
    closure.forget();

    // observe the main area instead of the canvas: the canvas is absolutely positioned,
    // while the main area is a normal element that can be resized
    let main_area = get_element_by_id("main_area");
    resize_observer.observe(&main_area);
}

async fn on_resize() {
    // force a complete setup and re-render
    setup();
    let view_guard = lock_view();
    if let Some(view) = view_guard.as_ref() {
        render(view);
    }
}

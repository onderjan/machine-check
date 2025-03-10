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

fn setup_listeners() {
    setup_window_listener(
        "resize",
        Box::new(|_| {
            wasm_bindgen_futures::spawn_local(resize());
        }),
    );

    setup_interval(
        Box::new(move |_| {
            wasm_bindgen_futures::spawn_local(work::tick());
        }),
        10,
    );
}

/// Logs to the frontend developer console.
#[macro_export]
macro_rules! console_log {
    ($($arg:tt)*) => {
        let a = ::std::format!($($arg)+);
        let cons = ::web_sys::js_sys::Array::new_with_length(1);
        cons.set(0, ::wasm_bindgen::JsValue::from_str(&a));
        ::web_sys::console::log(&cons);
    };
}

mod client;
mod tiling;
mod util;
mod view;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn exec() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    client::init().await;
}

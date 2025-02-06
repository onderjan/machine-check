mod fetch;
mod local;
pub mod view;

use wasm_bindgen::prelude::*;
use web_sys::{js_sys::Array, CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

struct Local {
    _main_canvas: HtmlCanvasElement,
    main_context: CanvasRenderingContext2d,
}

impl Local {
    fn new() -> Local {
        let window = web_sys::window().expect("HTML Window should exist");
        let document = window.document().expect("HTML document should exist");
        let main_canvas = document
            .get_element_by_id("main_canvas")
            .expect("Main canvas should exist");
        let main_canvas: HtmlCanvasElement = main_canvas
            .dyn_into()
            .expect("Main canvas should be a Canvas element");
        let main_context: CanvasRenderingContext2d = main_canvas
            .get_context("2d")
            .expect("Main canvas 2D context should be obtainable without an error")
            .expect("Main canvas should have a 2D context")
            .dyn_into()
            .expect("Main canvas 2D rendering context should be castable");
        Local {
            _main_canvas: main_canvas,
            main_context,
        }
    }
}

thread_local! {
    static LOCAL: Local = Local::new();
}
#[wasm_bindgen]
pub async fn exec() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    LOCAL.with(|local| local.main_context.fill_rect(0., 0., 20., 20.));
    let result = fetch::fetch().await;

    alert(&format!("{:?}", result));
    let cons = Array::new_with_length(1);
    cons.set(0, JsValue::from_str(&format!("{:?}", result)));

    web_sys::console::log(&cons);

    //main_context.fill_rect(0., 0., 20., 20.);
    // TODO: this is just a placeholder into which the current Javascript GUI implementation will be migrated
    //a();
}

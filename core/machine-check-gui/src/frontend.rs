use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

fn a() {
    alert("Hello, machine-check-gui-wasm!");
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let val = document.create_element("p").unwrap();
    val.set_text_content(Some("Hello from Rust!"));

    body.append_child(&val).unwrap();
}

#[wasm_bindgen]
pub fn greet() {
    // TODO: this is just a placeholder into which the current Javascript GUI implementation will be migrated
    a();
}

#[allow(dead_code)]
pub fn b() {}

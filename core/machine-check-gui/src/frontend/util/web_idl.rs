use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{CanvasRenderingContext2d, Document, Element, HtmlCanvasElement, Window};

pub fn setup_selector_listener(
    selector: &str,
    event_name: &str,
    func: Box<dyn FnMut(web_sys::Event)>,
) {
    let element = document()
        .query_selector(selector)
        .expect("Selector should succeed")
        .expect("Selector should select some element");

    setup_element_listener(&element, event_name, func);
}

pub fn setup_element_listener(
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

pub fn setup_interval(func: Box<dyn FnMut(web_sys::Event)>, interval: i32) {
    let closure = Closure::wrap(func);
    let handler = closure.as_ref().dyn_ref().unwrap();
    window()
        .set_interval_with_callback_and_timeout_and_arguments_0(handler, interval)
        .unwrap();
    // the closure must be explicitely forgotten so it remains accessible
    closure.forget();
}

pub fn window() -> Window {
    web_sys::window().expect("HTML Window should exist")
}

pub fn document() -> Document {
    window().document().expect("HTML document should exist")
}

pub fn get_element_by_id(element_id: &str) -> Element {
    let element = document().get_element_by_id(element_id);
    match element {
        Some(element) => element,
        None => panic!("Element '{}' should exist", element_id),
    }
}

pub fn create_element(local_name: &str) -> Element {
    let element = document().create_element(local_name);
    match element {
        Ok(element) => element,
        Err(err) => panic!("Element '{}' could not be created: {:?}", local_name, err),
    }
}

pub fn main_canvas_with_context() -> (HtmlCanvasElement, CanvasRenderingContext2d) {
    let main_canvas = get_element_by_id("main_canvas");
    let main_canvas: HtmlCanvasElement = main_canvas
        .dyn_into()
        .expect("Main canvas should be a Canvas element");
    let main_context: CanvasRenderingContext2d = main_canvas
        .get_context("2d")
        .expect("Main canvas 2D context should be obtainable without an error")
        .expect("Main canvas should have a 2D context")
        .dyn_into()
        .expect("Main canvas 2D rendering context should be castable");
    (main_canvas, main_context)
}

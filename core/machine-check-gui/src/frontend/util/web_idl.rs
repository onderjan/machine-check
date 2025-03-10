use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Document, Element, Window};

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

use wasm_bindgen::JsCast;

use crate::frontend::{
    util::web_idl::setup_selector_listener,
    view::NavigationTarget,
    work::{render, VIEW},
};

#[derive(Clone, Copy, Debug)]
pub enum KeyboardEvent {
    Down,
    Up,
}

mod codes {
    pub const HOME: &str = "Home";
    pub const ARROW_UP: &str = "ArrowUp";
    pub const ARROW_LEFT: &str = "ArrowLeft";
    pub const ARROW_RIGHT: &str = "ArrowRight";
    pub const ARROW_DOWN: &str = "ArrowDown";
}

pub fn init() {
    for (event_type, event_name) in [
        (KeyboardEvent::Down, "keydown"),
        (KeyboardEvent::Up, "keyup"),
    ] {
        setup_selector_listener(
            "#main_area",
            event_name,
            Box::new(move |e| {
                wasm_bindgen_futures::spawn_local(on_keyboard(event_type, e));
            }),
        );
        setup_selector_listener(
            "#main_canvas",
            event_name,
            Box::new(move |e| {
                wasm_bindgen_futures::spawn_local(on_keyboard(event_type, e));
            }),
        );
    }
}

async fn on_keyboard(keyboard: KeyboardEvent, event: web_sys::Event) {
    let mut view_guard = VIEW.lock().expect("View should not be poisoned");
    let Some(view) = view_guard.as_mut() else {
        return;
    };
    console_log!(&format!("Keyboard event received: {:?}", event));

    let event: web_sys::KeyboardEvent = event
        .dyn_into()
        .expect("Keyboard event should be KeyboardEvent");

    let navigation_target = match event.code().as_str() {
        codes::HOME => NavigationTarget::Root,
        codes::ARROW_UP => NavigationTarget::Up,
        codes::ARROW_DOWN => NavigationTarget::Down,
        codes::ARROW_LEFT => NavigationTarget::Left,
        codes::ARROW_RIGHT => NavigationTarget::Right,
        _ => return,
    };

    // prevent default action only for the handled events
    // to prevent keyboard traps
    event.prevent_default();

    match keyboard {
        KeyboardEvent::Down => {
            view.navigate(navigation_target);
            // redraw
            render(view);
        }
        KeyboardEvent::Up => {
            // nothing for now
        }
    }
}

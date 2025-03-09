use wasm_bindgen::JsCast;

use crate::frontend::work::view::{NavigationTarget, View};

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

pub fn on_keyboard(view: &mut View, keyboard: KeyboardEvent, event: web_sys::Event) -> bool {
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
        _ => return false,
    };

    // prevent default action only for the handled events
    // to prevent keyboard traps
    event.prevent_default();

    match keyboard {
        KeyboardEvent::Down => {
            view.navigate(navigation_target);
            // redraw
            true
        }
        KeyboardEvent::Up => {
            // nothing for now
            false
        }
    }
}

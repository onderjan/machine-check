use std::sync::{LazyLock, Mutex};

use machine_check_exec::Proposition;
use view::{camera::Camera, PropertiesView, View};

use super::interaction::Request;

mod control;
pub mod input;
mod view;

pub async fn command(request: Request, force: bool) {
    let new_snapshot = control::command(request).await;

    let mut view = VIEW.lock().expect("View should not be poisoned");

    let new_view = if let Some(view) = view.take() {
        View::new(new_snapshot, view.camera, view.properties)
    } else {
        let properties = PropertiesView::new(vec![Proposition::inherent()]);
        View::new(new_snapshot, Camera::new(), properties)
    };
    new_view.render(force);
    view.replace(new_view);
}

pub fn render(force: bool) {
    let view_guard = VIEW.lock().expect("View should not be poisoned");
    let view = view_guard.as_ref();
    if let Some(view) = view {
        view.render(force);
    }
}

pub fn on_keyboard(keyboard: super::KeyboardEvent, event: web_sys::Event) {
    let mut view_guard = VIEW.lock().expect("View should not be poisoned");
    let view = view_guard.as_mut();
    if let Some(view) = view {
        if input::keyboard::on_keyboard(view, keyboard, event) {
            view.render(false);
        }
    }
}

pub fn on_mouse(mouse: super::MouseEvent, event: web_sys::Event) {
    let mut view_guard = VIEW.lock().expect("View should not be poisoned");
    let view = view_guard.as_mut();
    if let Some(view) = view {
        if input::mouse::on_mouse(view, mouse, event) {
            view.render(false);
        }
    }
}

static VIEW: LazyLock<Mutex<Option<View>>> = LazyLock::new(|| Mutex::new(None));

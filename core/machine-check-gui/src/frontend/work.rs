use std::cell::RefCell;

use view::{camera::Camera, View};

use super::interaction::Request;

mod control;
mod input;
mod view;

pub async fn command(request: Request, force: bool) {
    let new_snapshot = control::command(request).await;

    VIEW.with_borrow_mut(|view| {
        let new_view = if let Some(view) = view.take() {
            View::new(new_snapshot, view.camera)
        } else {
            View::new(new_snapshot, Camera::new())
        };
        new_view.render(force);
        view.replace(new_view);
    });
}

pub fn render(force: bool) {
    VIEW.with_borrow_mut(|view| {
        if let Some(view) = view {
            view.render(force);
        }
    });
}

pub fn on_mouse(mouse: super::MouseEvent, event: web_sys::Event) {
    VIEW.with_borrow_mut(|view| {
        if let Some(view) = view {
            if input::on_mouse(view, mouse, event) {
                view.render(false);
            }
        }
    });
}

thread_local! {
    static VIEW: RefCell<Option<View>> = const { RefCell::new(None) };
}

use std::sync::{LazyLock, Mutex, MutexGuard};

use view::{camera::Camera, View};

use super::interaction::{Request, StepSettings};

mod control;
pub mod input;
mod view;

pub async fn init() {
    command(Request::GetContent, true).await;
    control::init();
}

pub async fn step(max_refinements: Option<u64>) {
    let selected_property = {
        let view_guard = VIEW.lock().expect("View should not be poisoned");

        let Some(view) = view_guard.as_ref() else {
            return;
        };

        // select the property to use for stepping
        // use the root property, not the subproperty, as we are interested
        // in whether the root property holds or not
        let Some(selected_property) = view.selected_root_property() else {
            // if no property is selected, just quietly return
            return;
        };

        selected_property.property.clone()
    };

    command(
        Request::Step(StepSettings {
            max_refinements,
            selected_property,
        }),
        false,
    )
    .await;
}

pub async fn reset() {
    command(Request::Reset, false).await;
}

pub fn render(force: bool) {
    let view_guard = VIEW.lock().expect("View should not be poisoned");
    if let Some(view) = view_guard.as_ref() {
        view.render(force);
    }
}

pub fn on_keyboard(keyboard: super::KeyboardEvent, event: web_sys::Event) {
    let mut view_guard = VIEW.lock().expect("View should not be poisoned");
    if let Some(view) = view_guard.as_mut() {
        if input::keyboard::on_keyboard(view, keyboard, event) {
            view.render(false);
        }
    }
}

pub fn on_mouse(mouse: super::MouseEvent, event: web_sys::Event) {
    let mut view_guard = VIEW.lock().expect("View should not be poisoned");
    if let Some(view) = view_guard.as_mut() {
        if input::mouse::on_mouse(view, mouse, event) {
            view.render(false);
        }
    }
}
async fn command(request: Request, force: bool) {
    let new_snapshot = control::command(request).await;
    let mut view_guard = VIEW.lock().expect("View should not be poisoned");

    let new_view = if let Some(view) = view_guard.take() {
        View::new(new_snapshot, view.camera)
    } else {
        View::new(new_snapshot, Camera::new())
    };
    new_view.render(force);
    view_guard.replace(new_view);
}

static VIEW: LazyLock<Mutex<Option<View>>> = LazyLock::new(|| Mutex::new(None));

fn lock_view() -> MutexGuard<'static, Option<View>> {
    VIEW.lock().expect("View should not be poisoned")
}

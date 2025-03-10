use super::{
    interaction::{BackendStatus, Request},
    util::web_idl::get_element_by_id,
    view::{camera::Camera, View},
};

mod canvas;
mod control;
mod input;
mod text;
mod tick;

pub async fn init() {
    canvas::init();
    issue_command(Request::GetContent).await;
    input::init();
    control::init();
    tick::init();
}

async fn issue_command(request: Request) {
    let is_query = matches!(request, Request::Query);
    let mut response = control::call_backend(request).await;
    if is_query {
        {
            let mut view_guard = lock_view();
            let Some(view) = view_guard.as_mut() else {
                return;
            };
            if matches!(view.backend_status, BackendStatus::Waiting)
                || !matches!(response.backend_status, BackendStatus::Waiting)
            {
                return;
            }
            view.backend_status = BackendStatus::Waiting;
        }
        response = control::call_backend(Request::GetContent).await;
    }

    let mut view_guard = lock_view();

    let status_str = match response.backend_status {
        BackendStatus::Waiting => "Waiting",
        BackendStatus::Running => "Running",
    };

    let status_element = get_element_by_id("verification_status");
    status_element.set_text_content(Some(status_str));

    // update the view with the snapshot and backend status
    if let Some(snapshot) = response.snapshot {
        let new_view = if let Some(view) = view_guard.take() {
            View::new(snapshot, response.backend_status, view.camera)
        } else {
            View::new(snapshot, response.backend_status, Camera::new())
        };

        render(&new_view);
        view_guard.replace(new_view);
    } else if let Some(view) = view_guard.as_mut() {
        view.backend_status = response.backend_status;
    }
}

fn render(view: &View) {
    canvas::render(view);
    text::display(view);
}

// put the view singleton in its own scope so it cannot be manipulated otherwise
mod view_singleton {
    use std::sync::{LazyLock, Mutex, MutexGuard};

    use crate::frontend::view::View;

    static VIEW: LazyLock<Mutex<Option<View>>> = LazyLock::new(|| Mutex::new(None));

    pub fn lock_view() -> MutexGuard<'static, Option<View>> {
        VIEW.lock().expect("View should not be poisoned")
    }
}

use view_singleton::lock_view;

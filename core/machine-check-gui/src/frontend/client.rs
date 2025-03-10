use super::{
    util::web_idl::get_element_by_id,
    view::{camera::Camera, View},
};
use crate::shared::{BackendStatus, Request};

mod canvas;
mod control;
mod input;
mod text;
mod tick;

pub async fn init() {
    // get the initial content first to provide the initial view
    let response = control::call_backend(Request::GetContent).await;
    let Some(snapshot) = response.snapshot else {
        panic!("Initial content should have a snapshot");
    };
    let initial_view = View::new(snapshot, response.backend_status, Camera::new());
    view_singleton::provide_initial_view(initial_view);

    canvas::init();
    input::init();
    control::init();
    tick::init();

    // make sure rendering occurs
    let view_guard = lock_view();
    render(view_guard.as_ref());
}

async fn issue_command(request: Request) {
    let is_query = matches!(request, Request::Query);
    let mut response = control::call_backend(request).await;
    if is_query {
        {
            let mut view_guard = lock_view();
            let view = view_guard.as_mut();
            if matches!(view.backend_status, BackendStatus::Waiting)
                || !matches!(response.backend_status, BackendStatus::Waiting)
            {
                return;
            }
            view.backend_status = BackendStatus::Waiting;
        }
        response = control::call_backend(Request::GetContent).await;
    }

    // update the view with the snapshot and backend status and render
    let mut view_guard = lock_view();
    let view = view_guard.as_mut();
    view.backend_status = response.backend_status;
    if let Some(snapshot) = response.snapshot {
        view.apply_snapshot(snapshot);
        render(view);
    } else {
        render_backend_status(view);
    }
}

fn render(view: &View) {
    render_backend_status(view);
    canvas::render(view);
    text::display(view);
}

fn render_backend_status(view: &View) {
    let status_str = match view.backend_status {
        BackendStatus::Waiting => "Waiting",
        BackendStatus::Running => "Running",
    };
    let status_element = get_element_by_id("verification_status");
    status_element.set_text_content(Some(status_str));
}

// put the view singleton in its own scope so it cannot be manipulated otherwise
mod view_singleton {
    use std::sync::{LazyLock, Mutex, MutexGuard};

    use crate::frontend::view::View;

    static VIEW: LazyLock<Mutex<Option<View>>> = LazyLock::new(|| Mutex::new(None));

    pub struct ViewGuard {
        guard: MutexGuard<'static, Option<View>>,
    }

    impl ViewGuard {
        pub fn as_ref(&self) -> &View {
            self.guard
                .as_ref()
                .expect("View should be initially provided")
        }

        pub fn as_mut(&mut self) -> &mut View {
            self.guard
                .as_mut()
                .expect("View should be initially provided")
        }
    }

    pub fn lock_view() -> ViewGuard {
        ViewGuard {
            guard: VIEW.lock().expect("View should not be poisoned"),
        }
    }

    pub(super) fn provide_initial_view(view: View) {
        VIEW.lock()
            .expect("View should not be poisoned")
            .replace(view);
    }
}

use view_singleton::lock_view;

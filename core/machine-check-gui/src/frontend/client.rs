use super::{util::web_idl::get_element_by_id, view::View};
use crate::shared::{BackendInfo, BackendStatus, Request};

mod canvas;
mod control;
mod input;
mod text;
mod tick;

pub async fn init() {
    // get the initial content first to provide the initial view
    let response = control::call_backend(Request::InitialContent).await;
    let Some(snapshot) = response.snapshot else {
        panic!("Initial content should have a snapshot");
    };
    let initial_view = View::new(snapshot, response.info);
    view_singleton::provide_initial_view(initial_view);

    canvas::init();
    input::init();
    control::init();
    tick::init();

    // make sure rendering occurs
    let mut view_guard = lock_view();
    render(view_guard.as_mut());
}

async fn issue_command(request: Request) {
    let is_query = matches!(request, Request::Query);
    let mut response = control::call_backend(request).await;
    if is_query {
        {
            let mut view_guard = lock_view();
            let view = view_guard.as_mut();

            if *view.backend_info() == response.info {
                // no change, do nothing
                return;
            }

            if view.backend_info().status.is_waiting() || !response.info.status.is_waiting() {
                // the query should not result in content get
                // just update and display the info
                view.update_backend_info(response.info);
                display_backend_info(view.backend_info());
                return;
            }
            view.update_backend_info(response.info);
        }
        response = control::call_backend(Request::GetContent).await;
    }

    // update the view with the snapshot and backend status and render
    let mut view_guard = lock_view();
    let view = view_guard.as_mut();
    view.update_backend_info(response.info);
    if let Some(snapshot) = response.snapshot {
        view.apply_snapshot(snapshot);
        render(view);
    } else {
        display_backend_info(view.backend_info());
    }
}

fn render(view: &mut View) {
    display_backend_info(view.backend_info());
    canvas::render(view);
    text::display(view);
}

fn display_backend_info(backend_info: &BackendInfo) {
    let backend_status = &backend_info.status;
    let (status_class, status_str) = match backend_status {
        BackendStatus::Waiting => ("waiting", "Waiting"),
        BackendStatus::Running => ("running", "Running"),
        BackendStatus::Cancelling => ("cancelling", "Cancelling"),
    };
    let info_middle_element = get_element_by_id("info");
    info_middle_element
        .class_list()
        .remove_3("waiting", "running", "cancelling")
        .unwrap();
    info_middle_element
        .class_list()
        .add_1(status_class)
        .unwrap();

    let status_element = get_element_by_id("verification_status");
    status_element.set_text_content(Some(status_str));
    control::display_backend_status(backend_status);

    let space_info_element = get_element_by_id("space_info");

    let info = &backend_info.space_info;
    space_info_element.set_text_content(Some(&format!(
        "{} refinements, {} states, {} transitions",
        info.num_refinements, info.num_states, info.num_transitions
    )));
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

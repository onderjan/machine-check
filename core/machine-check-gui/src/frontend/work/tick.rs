use crate::frontend::{
    interaction::{BackendStatus, Request},
    util::web_idl::setup_interval,
};

use super::{issue_command, lock_view};

pub fn init() {
    setup_interval(
        Box::new(move |_| {
            wasm_bindgen_futures::spawn_local(tick());
        }),
        10,
    );
}

async fn tick() {
    // if the backend is running, query to see when it finished
    let backend_running = {
        let mut backend_running = false;
        let view_guard = lock_view();
        if let Some(view) = view_guard.as_ref() {
            backend_running = matches!(view.backend_status, BackendStatus::Running);
        }
        backend_running
    };

    if backend_running {
        issue_command(Request::Query).await;
    }
}

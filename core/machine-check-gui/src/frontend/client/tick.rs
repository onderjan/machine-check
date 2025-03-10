use crate::{
    frontend::util::web_idl::setup_interval,
    shared::{BackendStatus, Request},
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
        let view_guard = lock_view();
        matches!(view_guard.as_ref().backend_status, BackendStatus::Running)
    };

    if backend_running {
        issue_command(Request::Query).await;
    }
}

use crate::{frontend::util::web_idl::setup_interval, shared::Request};

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
    // if the backend is not waiting, query to see when it finishes
    let backend_waiting = {
        let view_guard = lock_view();
        view_guard.as_ref().backend_status.is_waiting()
    };

    if !backend_waiting {
        issue_command(Request::Query).await;
    }
}

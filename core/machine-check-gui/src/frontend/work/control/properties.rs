use machine_check_exec::{PreparedProperty, Proposition};

use crate::frontend::{
    interaction::Request,
    util::web_idl::{get_element_by_id, setup_element_listener, window},
    work::{self, lock_view},
};

pub fn init() {
    let new_property_element = get_element_by_id("new_property");
    setup_element_listener(
        &new_property_element,
        "click",
        Box::new(move |_| {
            wasm_bindgen_futures::spawn_local(on_new_property_click());
        }),
    );

    let new_property_element = get_element_by_id("delete_property");
    setup_element_listener(
        &new_property_element,
        "click",
        Box::new(move |_| {
            wasm_bindgen_futures::spawn_local(on_delete_property_click());
        }),
    );
}

async fn on_new_property_click() {
    let window = window();
    let property = window
        .prompt_with_message("Enter the new property")
        .unwrap();
    let Some(property) = property else {
        return;
    };

    let property = match Proposition::parse(&property) {
        Ok(ok) => PreparedProperty::new(ok),
        Err(err) => {
            window
                .alert_with_message(&format!("Error parsing property: {}", err))
                .unwrap();
            return;
        }
    };

    work::issue_command(Request::AddProperty(property)).await;
}

async fn on_delete_property_click() {
    let property_index = {
        let view_guard = lock_view();
        let view = view_guard.as_ref();
        let Some(property_subindex) = view.camera.selected_subproperty else {
            return;
        };
        view.snapshot.subindex_to_root_index(property_subindex)
    };

    // TODO: disallow removing the inherent property more elegantly
    if property_index.0 == 0 {
        return;
    }

    work::issue_command(Request::RemoveProperty(property_index)).await;
}

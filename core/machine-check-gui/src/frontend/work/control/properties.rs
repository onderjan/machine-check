use machine_check_exec::{PreparedProperty, Proposition};

use crate::frontend::{
    interaction::Request,
    setup_element_listener,
    work::{self, lock_view},
};

pub fn init() {
    let window = web_sys::window().expect("HTML Window should exist");
    let document = window.document().expect("HTML document should exist");
    let new_property_element = document
        .get_element_by_id("new_property")
        .expect("New property element should exist");

    setup_element_listener(
        &new_property_element,
        "click",
        Box::new(move |_| {
            wasm_bindgen_futures::spawn_local(on_new_property_click());
        }),
    );

    let new_property_element = document
        .get_element_by_id("delete_property")
        .expect("Delete property element should exist");
    setup_element_listener(
        &new_property_element,
        "click",
        Box::new(move |_| {
            wasm_bindgen_futures::spawn_local(on_delete_property_click());
        }),
    );
    console_log!("New property element has been set up");
}

async fn on_new_property_click() {
    let window = web_sys::window().expect("HTML Window should exist");
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

    work::command(Request::AddProperty(property), false).await;
}

async fn on_delete_property_click() {
    let property_index = {
        let view_guard = lock_view();
        let Some(view) = view_guard.as_ref() else {
            return;
        };
        let Some(property_subindex) = view.camera.selected_subproperty else {
            return;
        };
        view.snapshot.subindex_to_root_index(property_subindex)
    };

    // TODO: disallow removing the inherent property more elegantly
    if property_index.0 == 0 {
        return;
    }

    work::command(Request::RemoveProperty(property_index), false).await;
}

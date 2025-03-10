use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use crate::frontend::{
    interaction::{Request, StepSettings},
    util::web_idl::{get_element_by_id, setup_selector_listener},
    work::{issue_command, view_singleton::lock_view},
};

pub fn init() {
    setup_selector_listener(
        "#reset",
        "click",
        Box::new(|_e| {
            wasm_bindgen_futures::spawn_local(on_reset_click());
        }),
    );

    setup_selector_listener(
        "#step",
        "click",
        Box::new(|_e| {
            wasm_bindgen_futures::spawn_local(on_step_click());
        }),
    );

    setup_selector_listener(
        "#run",
        "click",
        Box::new(|_e| {
            wasm_bindgen_futures::spawn_local(on_run_click());
        }),
    );
}

pub async fn on_reset_click() {
    issue_command(Request::Reset).await;
}

pub async fn on_step_click() {
    let input: HtmlInputElement = get_element_by_id("max_refinements")
        .dyn_into()
        .expect("The number of steps element should be an input");

    let max_refinements = (input.value_as_number() as u64).max(1);

    issue_step(Some(max_refinements)).await;
}

pub async fn on_run_click() {
    issue_step(None).await;
}

pub async fn issue_step(max_refinements: Option<u64>) {
    let selected_property = {
        let view_guard = lock_view();

        // select the property to use for stepping
        // use the root property, not the subproperty, as we are interested
        // in whether the root property holds or not
        let Some(selected_property) = view_guard.as_ref().selected_root_property() else {
            // if no property is selected, just quietly return
            return;
        };

        selected_property.property.clone()
    };

    issue_command(Request::Step(StepSettings {
        max_refinements,
        selected_property,
    }))
    .await;
}

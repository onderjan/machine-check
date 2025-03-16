use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, HtmlInputElement};

use crate::frontend::util::web_idl::document;
use crate::shared::{Request, StepSettings};
use crate::{
    frontend::{
        client::{issue_command, view_singleton::lock_view},
        util::web_idl::{get_element_by_id, setup_selector_listener},
    },
    shared::BackendStatus,
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

pub fn display_backend_status(backend_status: &BackendStatus) {
    let reset_element: HtmlButtonElement = get_element_by_id("reset").dyn_into().unwrap();
    let step_element: HtmlButtonElement = get_element_by_id("step").dyn_into().unwrap();
    let run_element: HtmlButtonElement = get_element_by_id("run").dyn_into().unwrap();

    let mut step_active = false;
    let mut run_active = false;
    let document = document();
    if let Some(active_element) = document.active_element() {
        if active_element == **step_element {
            step_active = true;
        } else if active_element == **run_element {
            run_active = true;
        }
    }

    const DATA_ACTIVE: &str = "data-active";

    match backend_status {
        BackendStatus::Cancelling => {
            reset_element.set_disabled(true);
            step_element.set_disabled(true);
            run_element.set_disabled(true);

            if run_active && run_element.get_attribute(DATA_ACTIVE).is_none() {
                run_element.set_attribute(DATA_ACTIVE, "run").unwrap();
            }
        }
        BackendStatus::Waiting => {
            reset_element.set_disabled(false);
            step_element.set_disabled(false);
            run_element.set_disabled(false);

            match run_element.get_attribute(DATA_ACTIVE).as_deref() {
                Some("step") => {
                    step_element.focus().unwrap();
                }
                Some("run") => {
                    run_element.focus().unwrap();
                }
                _ => {}
            };
            run_element.remove_attribute(DATA_ACTIVE).unwrap();
        }
        BackendStatus::Running => {
            reset_element.set_disabled(true);
            step_element.set_disabled(true);
            run_element.set_disabled(false);

            if step_active {
                run_element.set_attribute(DATA_ACTIVE, "step").unwrap();
                // refocus on the run element, which now acts as cancellation
                run_element.focus().unwrap();
            }
        }
    }

    if backend_status.is_waiting() {
        // play symbol
        run_element.set_inner_text("\u{23F5}");
    } else {
        // pause symbol
        run_element.set_inner_text("\u{23F8}");
    }
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
    // depending on whether we are waiting, either run or cancel
    let waiting = {
        let view_guard = lock_view();
        view_guard.as_ref().backend_info.status.is_waiting()
    };

    if waiting {
        issue_step(None).await;
    } else {
        issue_command(Request::Cancel).await;
    }
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

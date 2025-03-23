use machine_check_exec::Conclusion;
use wasm_bindgen::JsCast;
use web_sys::{Element, Event, HtmlElement};

use crate::frontend::{
    client::{lock_view, render},
    util::web_idl::{create_element, document, get_element_by_id, setup_element_listener},
    view::View,
};
use crate::shared::snapshot::{PropertySnapshot, SubpropertyIndex};

pub fn display(view: &View) {
    PropertiesDisplayer::new(view).display();
}

struct PropertiesDisplayer<'a> {
    view: &'a View,
    properties_element: Element,
}

impl PropertiesDisplayer<'_> {
    fn new(view: &View) -> PropertiesDisplayer {
        let properties_element = get_element_by_id("properties").dyn_into().unwrap();
        PropertiesDisplayer {
            view,
            properties_element,
        }
    }

    fn display(&self) {
        // determine if some radio button was focused
        let mut was_focused = false;
        if let Some(active_element) = document().active_element() {
            if active_element.class_list().contains("property-radio") {
                was_focused = true;
            }
        }

        // remove all children
        self.properties_element.set_inner_html("");

        let mut id_index = 0;
        for property in self.view.snapshot().root_properties_iter() {
            Self::display_property(
                property,
                &self.properties_element,
                self.view.selected_subproperty_index(),
                &mut id_index,
                was_focused,
                false,
            );
        }
    }

    fn display_property(
        property_snapshot: &PropertySnapshot,
        parent_element: &Element,
        selected_subproperty: Option<SubpropertyIndex>,
        id_index: &mut usize,
        was_focused: bool,
        is_subproperty: bool,
    ) {
        let outer_div = create_element("div");
        outer_div.class_list().add_1("property_outer").unwrap();

        let radio_input = create_element("input");
        let radio_input: HtmlElement = radio_input.dyn_into().unwrap();
        radio_input.set_attribute("type", "radio").unwrap();
        radio_input.set_attribute("name", "property_group").unwrap();
        radio_input
            .set_attribute("data-index", &id_index.to_string())
            .unwrap();
        let radio_input_id = &format!("property_radio_{}", id_index);
        radio_input.set_id(radio_input_id);
        radio_input.class_list().add_1("property-radio").unwrap();

        let radio_label = create_element("label");

        radio_label.set_attribute("for", radio_input_id).unwrap();
        radio_label.set_text_content(Some(&property_snapshot.property.to_string()));

        if !is_subproperty {
            let conclusion_span = create_element("span");
            let (conclusion_class, conclusion_str, title_text) = match &property_snapshot.conclusion
            {
                Ok(conclusion) => match conclusion {
                    Conclusion::Known(true) => {
                        ("conclusion-true", "\u{2714}", String::from("Holds"))
                    }
                    Conclusion::Known(false) => (
                        "conclusion-false",
                        "\u{274C}",
                        String::from("Does not hold"),
                    ),
                    Conclusion::Unknown(_culprit) => {
                        ("conclusion-unknown", "\u{2754}", String::from("Unknown"))
                    }
                    Conclusion::NotCheckable => (
                        "conclusion-not-checkable",
                        "\u{2754}",
                        String::from("Unknown (the state space is currently not checkable)"),
                    ),
                },
                Err(err) => ("conclusion-error", "\u{1F6D1}", format!("Error: {}", err)),
            };
            conclusion_span
                .class_list()
                .add_2("conclusion", conclusion_class)
                .unwrap();
            conclusion_span.set_attribute("title", &title_text).unwrap();

            conclusion_span.set_text_content(Some(conclusion_str));
            radio_label.append_child(&conclusion_span).unwrap();
        }

        outer_div.append_child(&radio_input).unwrap();
        outer_div.append_child(&radio_label).unwrap();

        let property_ul = create_element("div");

        outer_div.append_child(&property_ul).unwrap();

        parent_element.append_child(&outer_div).unwrap();

        if let Some(selected_subproperty) = selected_subproperty {
            if selected_subproperty.0 == *id_index {
                radio_input.set_attribute("checked", "true").unwrap();
                // if a radio button was focused, focus on the currently checked
                console_log!("Checking radio button");
                if was_focused {
                    console_log!("Focusing radio button");
                    radio_input.focus().unwrap();
                }
            }
        }

        setup_element_listener(
            &radio_input,
            "change",
            Box::new(move |e| {
                wasm_bindgen_futures::spawn_local(on_radio_change(e));
            }),
        );

        *id_index += 1;

        for child in &property_snapshot.children {
            Self::display_property(
                child,
                &property_ul,
                selected_subproperty,
                id_index,
                was_focused,
                true,
            );
        }
    }
}

async fn on_radio_change(event: Event) {
    let mut view_guard = lock_view();
    let view = view_guard.as_mut();

    let element: Element = event.current_target().unwrap().dyn_into().unwrap();

    let index: usize = element
        .get_attribute("data-index")
        .unwrap()
        .parse()
        .unwrap();

    if let Some(current_selected_subproperty_index) = view.selected_subproperty_index() {
        if current_selected_subproperty_index.0 == index {
            // already selected, do nothing
            return;
        }
    }

    // change and redraw
    view.select_subproperty_index(Some(SubpropertyIndex(index)));
    render(view);
}

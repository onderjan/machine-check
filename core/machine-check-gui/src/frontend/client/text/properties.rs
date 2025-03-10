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
        for property in self.view.snapshot.root_properties_iter() {
            Self::display_property(
                property,
                &self.properties_element,
                self.view.camera.selected_subproperty,
                &mut id_index,
                was_focused,
            );
        }
    }

    fn display_property(
        property_snapshot: &PropertySnapshot,
        parent_element: &Element,
        selected_subproperty: Option<SubpropertyIndex>,
        id_index: &mut usize,
        was_focused: bool,
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

    if let Some(current_selected_subproperty) = view.camera.selected_subproperty {
        if current_selected_subproperty.0 == index {
            // already selected, do nothing
            return;
        }
    }

    // change and redraw
    view.camera.selected_subproperty = Some(SubpropertyIndex(index));
    render(view);
}

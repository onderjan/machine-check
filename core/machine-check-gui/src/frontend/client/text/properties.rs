use machine_check_common::{
    check::{Conclusion, KnownConclusion},
    ParamValuation,
};
use wasm_bindgen::JsCast;
use web_sys::{Element, Event, HtmlElement};

use crate::frontend::{
    client::{lock_view, render},
    util::web_idl::{create_element, document, get_element_by_id, setup_element_listener},
    view::View,
};
use crate::shared::snapshot::{SubpropertyIndex, SubpropertySnapshot};

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

        // TODO: do not force the inherent property to be the first
        let inherent_property = self
            .view
            .snapshot()
            .root_properties_iter()
            .next()
            .expect("The snapshot should have the inherent property");

        let inherent_result: ParamValuation = inherent_property
            .conclusion
            .as_ref()
            .map(|conclusion| match conclusion {
                Conclusion::Known(KnownConclusion::False) => ParamValuation::False,
                Conclusion::Known(KnownConclusion::True) => ParamValuation::True,
                Conclusion::Known(KnownConclusion::Dependent) => ParamValuation::Dependent,
                _ => ParamValuation::Unknown,
            })
            .unwrap_or(ParamValuation::Unknown);

        let panic_message = self.view.snapshot().panic_message.as_ref();

        let mut id_index = 0;
        for property in self.view.snapshot().root_properties_iter() {
            let is_inherent = id_index == 0;
            Self::display_property(
                property,
                &self.properties_element,
                self.view.selected_subproperty_index(),
                &mut id_index,
                was_focused,
                is_inherent,
                inherent_result,
                panic_message,
                false,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn display_property(
        property_snapshot: &SubpropertySnapshot,
        parent_element: &Element,
        selected_subproperty: Option<SubpropertyIndex>,
        id_index: &mut usize,
        was_focused: bool,
        is_inherent: bool,
        inherent_result: ParamValuation,
        panic_message: Option<&String>,
        is_subproperty: bool,
    ) {
        let parent_element = if let Some(property_str) = property_snapshot.subproperty.display_str()
        {
            let outer_div = create_element("div");
            outer_div.class_list().add_1("property-outer").unwrap();

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

            // TODO: remove the inherent property text kludge
            let property_text = if is_inherent {
                if is_subproperty {
                    String::from("No panic in the state")
                } else {
                    String::from("Inherent property")
                }
            } else {
                property_str.to_string()
            };

            radio_label.set_text_content(Some(&property_text));

            if !is_subproperty {
                let property_icons = create_element("span");
                property_icons.class_list().add_1("property-icons").unwrap();

                if !is_inherent {
                    // display a warning that the property value may be / is meaningless
                    // if the inherent property has not been proven
                    let inherent_warning_text = match inherent_result {
                        ParamValuation::True => None,
                        ParamValuation::False => Some(concat!(
                            "The inherent property does not hold.\n",
                            "This verification result is meaningless."
                        )),
                        ParamValuation::Dependent => Some(concat!(
                            "The inherent property holds or does not depending on parameters.\n",
                            "This verification result is meaningless."
                        )),
                        ParamValuation::Unknown => Some(concat!(
                            "The inherent property has not been proven to hold yet.\n",
                            "If it does not hold, this verification result is meaningless."
                        )),
                    };

                    if let Some(inherent_warning_text) = inherent_warning_text {
                        let inherent_warning = create_element("span");

                        inherent_warning
                            .set_attribute("title", inherent_warning_text)
                            .unwrap();
                        inherent_warning.set_text_content(Some("\u{26A0}\u{FE0F}"));

                        property_icons.append_child(&inherent_warning).unwrap();
                    }
                }

                let conclusion_span = create_element("span");
                let (conclusion_class, conclusion_str, title_text) = match &property_snapshot
                    .conclusion
                {
                    Ok(conclusion) => match conclusion {
                        Conclusion::Known(KnownConclusion::True) => {
                            ("conclusion-true", "\u{2714}\u{FE0F}", String::from("Holds"))
                        }
                        Conclusion::Known(KnownConclusion::False) => {
                            ("conclusion-false", "\u{274C}", {
                                let mut conclusion_string = String::from("Does not hold");
                                if is_inherent {
                                    if let Some(panic_message) = panic_message {
                                        conclusion_string = format!(
                                            "Does not hold, panic message: '{}'",
                                            panic_message
                                        );
                                    }
                                }
                                conclusion_string
                            })
                        }
                        Conclusion::Known(KnownConclusion::Dependent) => {
                            ("conclusion-dependent", "dependent", String::from("Unknown"))
                        }
                        Conclusion::Unknown(_culprit) => (
                            "conclusion-unknown",
                            "\u{2754}\u{FE0F}",
                            String::from("Unknown"),
                        ),
                        Conclusion::NotCheckable => (
                            "conclusion-not-checkable",
                            "\u{2754}",
                            String::from("Unknown (the state space is currently not checkable)"),
                        ),
                    },
                    Err(err) => (
                        "conclusion-error",
                        "\u{1F6D1}\u{FE0F}",
                        format!("Error: {}", err),
                    ),
                };
                conclusion_span
                    .class_list()
                    .add_2("conclusion", conclusion_class)
                    .unwrap();
                conclusion_span.set_attribute("title", &title_text).unwrap();
                conclusion_span.set_text_content(Some(conclusion_str));
                property_icons.append_child(&conclusion_span).unwrap();

                radio_label.append_child(&property_icons).unwrap();
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
            property_ul
        } else {
            parent_element.clone()
        };
        *id_index += 1;

        for child in &property_snapshot.children {
            Self::display_property(
                child,
                &parent_element,
                selected_subproperty,
                id_index,
                was_focused,
                is_inherent,
                inherent_result,
                panic_message,
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

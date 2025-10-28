use icondata::Icon;
use machine_check_common::{ExecError, ParamValuation};
use wasm_bindgen::JsCast;
use web_sys::{Element, Event, HtmlElement};

use crate::{
    frontend::{
        client::{lock_view, render},
        util::{
            constants,
            web_idl::{create_element, document, get_element_by_id, setup_element_listener},
        },
        view::View,
    },
    shared::snapshot::PropertyIndex,
};

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
        let inherent_property = self.view.snapshot().property_snapshot(PropertyIndex(0));

        let inherent_result: ParamValuation = inherent_property
            .conclusion
            .clone()
            .unwrap_or(ParamValuation::Unknown);

        let mut is_inherent = true;
        for property_index in 0..self.view.snapshot().properties.len() {
            let property_wrap = create_element("div");
            property_wrap.class_list().add_1("property-wrap").unwrap();

            self.display_subproperty(
                PropertyIndex(property_index),
                0,
                &property_wrap,
                &property_wrap,
                was_focused,
                is_inherent,
                inherent_result,
            );

            self.properties_element
                .append_child(&property_wrap)
                .unwrap();
            is_inherent = false;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn display_subproperty(
        &self,
        property_index: PropertyIndex,
        subproperty_index: usize,
        property_wrap: &Element,
        parent_element: &Element,
        was_focused: bool,
        is_inherent: bool,
        inherent_result: ParamValuation,
    ) {
        let selected_subproperty = self.view.selected_subproperty_index();
        let panic_message = self.view.snapshot().panic_message.as_ref();
        let property_snapshot = self.view.snapshot().property_snapshot(property_index);

        let is_subproperty = subproperty_index != 0;
        let subproperty = &property_snapshot.property.subproperties[subproperty_index];

        let parent_element = if let Some(display_str) = subproperty.display_str() {
            let outer_div = create_element("div");
            outer_div.class_list().add_1("property-outer").unwrap();

            let radio_input = create_element("input");
            let radio_input: HtmlElement = radio_input.dyn_into().unwrap();
            radio_input.set_attribute("type", "radio").unwrap();
            radio_input.set_attribute("name", "property_group").unwrap();
            radio_input
                .set_attribute("data-property", &property_index.0.to_string())
                .unwrap();
            radio_input
                .set_attribute("data-subproperty", &subproperty_index.to_string())
                .unwrap();
            let radio_input_id =
                &format!("property_radio_{}_{}", property_index.0, subproperty_index);
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
                display_str.to_string()
            };

            radio_label.set_text_content(Some(&property_text));

            outer_div.append_child(&radio_input).unwrap();
            outer_div.append_child(&radio_label).unwrap();

            if !is_subproperty {
                let property_icons = create_element("div");
                property_icons.class_list().add_1("property-icons").unwrap();
                property_wrap.append_child(&property_icons).unwrap();

                let conclusion_span = create_element("div");
                property_icons.append_child(&conclusion_span).unwrap();
                let (conclusion_class, conclusion_string, title_text) = conclusion_icon(
                    property_snapshot.conclusion.as_ref(),
                    is_inherent,
                    panic_message.map(|m| m.as_str()),
                );
                conclusion_span
                    .class_list()
                    .add_2("conclusion", conclusion_class)
                    .unwrap();
                conclusion_span.set_attribute("title", &title_text).unwrap();
                conclusion_span.set_inner_html(&conclusion_string);

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
                        let inherent_warning = create_element("div");

                        inherent_warning.class_list().add_1("conclusion").unwrap();
                        inherent_warning
                            .set_attribute("title", inherent_warning_text)
                            .unwrap();
                        inherent_warning
                            .set_inner_html(&icon_data(icondata::BsExclamationTriangle, "#000"));

                        property_icons.append_child(&inherent_warning).unwrap();
                    }
                }
            }

            let property_ul = create_element("div");

            outer_div.append_child(&property_ul).unwrap();

            parent_element.append_child(&outer_div).unwrap();

            if let Some(selected_subproperty) = selected_subproperty {
                if selected_subproperty == (property_index, subproperty_index) {
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

        for child_index in subproperty.children() {
            self.display_subproperty(
                property_index,
                *child_index,
                property_wrap,
                &parent_element,
                was_focused,
                is_inherent,
                inherent_result,
            );
        }
    }
}

fn conclusion_icon(
    valuation: Result<&ParamValuation, &ExecError>,
    is_inherent: bool,
    panic_message: Option<&str>,
) -> (&'static str, String, String) {
    match valuation {
        Ok(valuation) => match valuation {
            ParamValuation::True => (
                "conclusion-true",
                icon_data(icondata::TbSquareCheckFilled, constants::colors::TRUE),
                String::from("Holds"),
            ),
            ParamValuation::False => (
                "conclusion-false",
                icon_data(icondata::TbSquareXFilled, constants::colors::FALSE),
                {
                    let mut conclusion_string = String::from("Does not hold");
                    if is_inherent {
                        if let Some(panic_message) = panic_message {
                            conclusion_string =
                                format!("Does not hold, panic message: '{}'", panic_message);
                        }
                    }
                    conclusion_string
                },
            ),
            ParamValuation::Dependent => (
                "conclusion-dependent",
                icon_data(
                    icondata::TbSquareAsteriskFilled,
                    constants::colors::DEPENDENT,
                ),
                String::from("Dependent on parameters"),
            ),
            ParamValuation::Unknown => (
                "conclusion-unknown",
                icon_data(icondata::BsQuestionSquareFill, "#555"),
                String::from("Unknown"),
            ),
        },
        Err(err) => (
            "conclusion-error",
            icon_data(icondata::BsExclamationSquareFill, "#00c"),
            format!("Error: {}", err),
        ),
    }
}

fn icon_data(icon: Icon, color: &'static str) -> String {
    let view_box = if let Some(view_box) = icon.view_box {
        format!("viewBox=\"{}\"", view_box)
    } else {
        String::new()
    };

    format!("<svg {} fill={}>{}</svg>", view_box, color, icon.data)
}

async fn on_radio_change(event: Event) {
    let mut view_guard = lock_view();
    let view = view_guard.as_mut();

    let element: Element = event.current_target().unwrap().dyn_into().unwrap();

    let property_index = PropertyIndex(
        element
            .get_attribute("data-property")
            .unwrap()
            .parse()
            .unwrap(),
    );

    let subproperty_index: usize = element
        .get_attribute("data-subproperty")
        .unwrap()
        .parse()
        .unwrap();

    if let Some(selected) = view.selected_subproperty_index() {
        if selected == (property_index, subproperty_index) {
            // already selected, do nothing
            return;
        }
    }

    // change and redraw
    view.select_subproperty_index(property_index, subproperty_index);
    render(view);
}

use wasm_bindgen::JsCast;
use web_sys::Element;

use crate::frontend::work::view::{PropertyView, View};

pub fn display(view: &View) {
    PropertiesDisplayer::new(view).display();
}

struct PropertiesDisplayer<'a> {
    view: &'a View,
    properties_element: Element,
}

impl PropertiesDisplayer<'_> {
    fn new(view: &View) -> PropertiesDisplayer {
        let window = web_sys::window().expect("HTML Window should exist");
        let document = window.document().expect("HTML document should exist");
        let properties_element = document
            .get_element_by_id("properties")
            .expect("Properties element should exist");
        let properties_element = properties_element.dyn_into().unwrap();
        PropertiesDisplayer {
            view,
            properties_element,
        }
    }

    fn display(&self) {
        // remove all children
        self.properties_element.set_inner_html("");

        for property in self.view.properties.vec.iter() {
            Self::display_property(property, &self.properties_element);
        }
    }

    fn display_property(property: &PropertyView, parent_element: &Element) {
        let property_li = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("li")
            .unwrap();

        property_li.set_text_content(Some(&property.prop.to_string()));

        let property_ul = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("ul")
            .unwrap();

        property_li.append_child(&property_ul).unwrap();

        for child in &property.children {
            Self::display_property(child, &property_ul);
        }

        parent_element.append_child(&property_li).unwrap();
    }
}

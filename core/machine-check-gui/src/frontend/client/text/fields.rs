use machine_check_common::NodeId;
use mck::abstr::{AbstractDisplay, ArrayDisplay};

use wasm_bindgen::JsCast;
use web_sys::{HtmlTableCellElement, HtmlTableElement, HtmlTableRowElement};

use crate::{frontend::util::web_idl::get_element_by_id, shared::snapshot::Node};

use super::View;

pub fn display(view: &View) {
    FieldDisplayer::new(view).display();
}

const STANDARD_FIELD_CLASSES: &[&str; 0] = &[];
const STANDARD_VALUE_CLASSES: &[&str; 1] = &["monospace"];

struct FieldDisplayer<'a> {
    view: &'a View,
    state_fields: HtmlTableElement,
}

impl FieldDisplayer<'_> {
    fn new(view: &View) -> FieldDisplayer {
        let state_fields = get_element_by_id("state_fields");
        let state_fields = state_fields.dyn_into().unwrap();
        FieldDisplayer { view, state_fields }
    }

    fn display(&self) {
        // remove all children
        self.state_fields.set_inner_html("");

        let mut selected = false;

        if let Some(node_id) = self.view.selected_node_id() {
            let node = self.view.snapshot().state_space.nodes.get(&node_id);
            if let Some(node) = node {
                selected = true;
                self.display_node_fields(node_id, node);
            }
        }

        if !selected {
            self.add_auxiliary_row("No node selected");
        }
    }

    fn display_node_fields(&self, node_id: NodeId, node: &Node) {
        self.add_field_row("id", &node_id.to_string(), &["bold", "italic"], &[]);

        let panic_value = match &node.panic_state {
            Some(panic_state) => {
                panic_state.expect_struct()[1].to_string()
                /*let bitvec = panic_state.expect_struct()[1].expect_bitvector();

                match bitvec.concrete_value() {
                    Some(value) => {
                        if value.is_nonzero() {
                            "\"1\""
                        } else {
                            "\"0\""
                        }
                    }
                    None => "\"X\"",
                }
                .to_string()*/
            }
            None => String::from("(none)"),
        };

        self.add_field_row("panic", &panic_value, &["bold", "italic"], &["bold"]);

        let Some(panic_state) = &node.panic_state else {
            return;
        };

        let field_types = &self.view.snapshot().state_info.fields;
        let field_values = panic_state.expect_struct()[0].expect_struct();
        assert_eq!(field_types.len(), field_values.len());

        for (field_ident, value) in field_types.keys().zip(field_values) {
            match value {
                AbstractDisplay::Array(array) => {
                    self.display_array_field(field_ident.name(), array);
                }
                AbstractDisplay::Boolean(boolean) => {
                    self.add_field_row(
                        field_ident.name(),
                        &boolean.to_string(),
                        STANDARD_FIELD_CLASSES,
                        STANDARD_VALUE_CLASSES,
                    );
                }
                AbstractDisplay::Bitvector(_) | AbstractDisplay::Struct(_) => {
                    self.add_field_row(
                        field_ident.name(),
                        &value.to_string(),
                        STANDARD_FIELD_CLASSES,
                        STANDARD_VALUE_CLASSES,
                    );
                }
            };
        }
    }

    fn display_array_field(&self, field_name: &str, array: &ArrayDisplay) {
        for ((start_index, end_index), element) in &array.0 {
            let field_part_name = if start_index == end_index {
                format!("{}[{}]", field_name, start_index)
            } else {
                format!("{}[{}..={}]", field_name, start_index, end_index)
            };

            self.add_field_row(
                &field_part_name,
                &element.to_string(),
                STANDARD_FIELD_CLASSES,
                STANDARD_VALUE_CLASSES,
            );
        }
    }

    fn add_field_row(
        &self,
        field: &str,
        value: &str,
        field_classes: &[&str],
        value_classes: &[&str],
    ) {
        let row: HtmlTableRowElement = self.state_fields.insert_row().unwrap().dyn_into().unwrap();
        let field_element = row.insert_cell().unwrap();
        field_element.set_inner_text(field);
        let value_element = row.insert_cell().unwrap();
        value_element.set_inner_text(value);

        for field_class in field_classes {
            field_element.class_list().add_1(field_class).unwrap();
        }

        for value_class in value_classes {
            value_element.class_list().add_1(value_class).unwrap();
        }
    }

    fn add_auxiliary_row(&self, text: &str) {
        let row: HtmlTableRowElement = self.state_fields.insert_row().unwrap().dyn_into().unwrap();
        row.class_list().add_1("dynamic").unwrap();
        let field_cell: HtmlTableCellElement = row.insert_cell().unwrap().dyn_into().unwrap();
        field_cell.set_inner_text(text);
        field_cell.set_col_span(2);
    }
}

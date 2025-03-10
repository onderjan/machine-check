use std::collections::BTreeMap;

use machine_check_exec::NodeId;
use mck::abstr::{ArrayField, ArrayFieldBitvector, BitvectorField, Field};
use wasm_bindgen::JsCast;
use web_sys::{HtmlTableCellElement, HtmlTableElement, HtmlTableRowElement};

use crate::frontend::{get_element_by_id, snapshot::Node};

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

        if let Some(node_id) = self.view.camera.selected_node_id {
            let node = self.view.snapshot.state_space.nodes.get(&node_id);
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

        let panic_value = match &node.panic {
            Some(tvb) => tvb.to_string(),
            None => String::from("(none)"),
        };

        self.add_field_row("panic", &panic_value, &["bold", "italic"], &["bold"]);

        for (field_name, value) in node.fields.iter() {
            match value {
                Field::Bitvector(bitvector) => {
                    self.add_field_row(
                        field_name,
                        &bitvector.to_string(),
                        STANDARD_FIELD_CLASSES,
                        STANDARD_VALUE_CLASSES,
                    );
                }
                Field::Array(array) => {
                    self.display_array_field(field_name, array);
                }
            };
        }
    }

    fn display_array_field(&self, field_name: &str, array: &ArrayField) {
        // the light array contains the values only for the leftmost indices in the runs of same elements
        // we want to create the names of slices where all elements correspond to the same value
        // i.e. field_name[i..=j] where there is a multi-element run and field_name[i] where there is
        // a single-element run, i.e. i == j
        let inner: BTreeMap<u64, ArrayFieldBitvector> = BTreeMap::from_iter(
            array
                .inner
                .iter()
                .map(|(index, bitvector)| (*index, *bitvector)),
        );

        // we need to be able to look at the successive two elements, so we use peeking
        let mut iter = inner.into_iter().peekable();
        while let Some((start_index, element)) = iter.next() {
            let peek = iter.peek();
            let end_index = if let Some(peek) = peek {
                // we have a next index, the end index of this run is just before it
                peek.0 - 1
            } else if array.bit_length == u64::BITS {
                // there is no next index; since the array is the maximum amount of bits wide,
                // we must explicitly use the maximum value since the right shift would overflow
                u64::MAX
            } else {
                // there is no next index; we can compute the end index here by (2^N)-1
                // without overflow
                (1u64 << array.bit_length) - 1
            };

            // format the field part name accordingly
            let field_part_name = if start_index == end_index {
                format!("{}[{}]", field_name, start_index)
            } else {
                format!("{}[{}..={}]", field_name, start_index, end_index)
            };

            // setup the value for printing
            let value = BitvectorField {
                bit_width: array.bit_width,
                zeros: element.zeros,
                ones: element.ones,
            };
            self.add_field_row(
                &field_part_name,
                &value.to_string(),
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

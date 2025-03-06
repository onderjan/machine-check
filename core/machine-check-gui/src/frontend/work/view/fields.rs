use std::collections::BTreeMap;

use machine_check_exec::NodeId;
use mck::abstr::{ArrayFieldBitvector, BitvectorField, Field};
use wasm_bindgen::JsCast;
use web_sys::{HtmlTableCellElement, HtmlTableElement, HtmlTableRowElement};

use crate::frontend::snapshot::Node;

use super::View;

pub fn display(view: &View) {
    LOCAL.with(|local| {
        // remove all children
        local.state_fields.set_inner_html("");

        let mut selected = false;

        if let Some(node_id) = view.camera.selected_node_id {
            let node = view.snapshot.state_space.nodes.get(&node_id);
            if let Some(node) = node {
                selected = true;
                display_node_fields(local, node_id, node);
            }
        }

        if !selected {
            add_auxiliary_row(local, "No node selected");
        }
    });
}

fn display_node_fields(local: &Local, node_id: NodeId, node: &Node) {
    add_field_row(local, "id", &node_id.to_string(), &["bold", "italic"], &[]);

    let panic_value = match &node.panic {
        Some(tvb) => tvb.to_string(),
        None => String::from("(none)"),
    };

    add_field_row(local, "panic", &panic_value, &["bold", "italic"], &["bold"]);

    let standard_field_classes = &[];
    let standard_value_classes = &["monospace"];

    for (field, value) in node.fields.iter() {
        match value {
            Field::Bitvector(bitvector) => {
                let value = format!("{}", bitvector);
                add_field_row(
                    local,
                    field,
                    &value,
                    standard_field_classes,
                    standard_value_classes,
                );
            }
            Field::Array(array) => {
                let inner: BTreeMap<u64, ArrayFieldBitvector> = BTreeMap::from_iter(
                    array
                        .inner
                        .iter()
                        .map(|(key, value)| (key.parse::<u64>().unwrap(), *value)),
                );

                let mut iter = inner.into_iter().peekable();
                while let Some((start_index, element)) = iter.next() {
                    let peek = iter.peek();
                    let end_index = if let Some(peek) = peek {
                        peek.0 - 1
                    } else if array.bit_length == u64::BITS {
                        u64::MAX
                    } else {
                        (1u64 << array.bit_length) - 1
                    };

                    let field = if start_index != end_index {
                        format!("{}[{}..={}]", field, start_index, end_index)
                    } else {
                        format!("{}[{}]", field, start_index)
                    };
                    let value = format!(
                        "{}",
                        BitvectorField {
                            bit_width: array.bit_width,
                            zeros: element.zeros,
                            ones: element.ones
                        }
                    );
                    add_field_row(
                        local,
                        &field,
                        &value,
                        standard_field_classes,
                        standard_value_classes,
                    );
                }
            }
        };
    }
}

fn add_field_row(
    local: &Local,
    field: &str,
    value: &str,
    field_classes: &[&str],
    value_classes: &[&str],
) {
    let row: HtmlTableRowElement = local.state_fields.insert_row().unwrap().dyn_into().unwrap();
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

fn add_auxiliary_row(local: &Local, text: &str) {
    let row: HtmlTableRowElement = local.state_fields.insert_row().unwrap().dyn_into().unwrap();
    row.class_list().add_1("dynamic").unwrap();
    let field_cell: HtmlTableCellElement = row.insert_cell().unwrap().dyn_into().unwrap();
    field_cell.set_inner_text(text);
    field_cell.set_col_span(2);
}

struct Local {
    state_fields: HtmlTableElement,
}
impl Local {
    fn new() -> Local {
        let window = web_sys::window().expect("HTML Window should exist");
        let document = window.document().expect("HTML document should exist");
        let state_fields = document
            .get_element_by_id("state_fields")
            .expect("State fields should exist");
        let state_fields = state_fields.dyn_into().unwrap();

        Local { state_fields }
    }
}

thread_local! {
    static LOCAL: Local = Local::new();
}

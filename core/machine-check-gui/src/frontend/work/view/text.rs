use super::View;

mod fields;
mod properties;

pub fn display(view: &View) {
    fields::display(view);
    properties::display(view);
}

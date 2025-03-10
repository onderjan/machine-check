use super::View;

mod fields;
mod log;
mod properties;

pub fn display(view: &View) {
    log::display(view);
    fields::display(view);
    properties::display(view);
}

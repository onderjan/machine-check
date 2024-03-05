use proc_macro2::Span;
use syn::{visit_mut::VisitMut, Item};

mod process;
mod visit_mut;

use crate::{ErrorType, MachineError};

pub fn normalize_constructs(items: &mut [Item]) -> Result<(), MachineError> {
    let mut visitor = Visitor { result: Ok(()) };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }

    visitor.result
}

struct Visitor {
    result: Result<(), MachineError>,
}

impl Visitor {
    fn push_error(&mut self, msg: String, span: Span) {
        if self.result.is_ok() {
            self.result = Err(MachineError::new(
                ErrorType::UnsupportedConstruct(msg),
                span,
            ));
        }
    }
}

use proc_macro2::Span;
use syn::{visit::Visit, Item};

mod visit_mut;

use super::error::{DescriptionError, DescriptionErrorType};

pub fn normalize_constructs(items: &mut [Item]) -> Result<(), DescriptionError> {
    let mut visitor = Visitor { result: Ok(()) };
    for item in items.iter_mut() {
        visitor.visit_item(item);
    }

    visitor.result
}

struct Visitor {
    result: Result<(), DescriptionError>,
}

impl Visitor {
    fn push_error(&mut self, msg: &'static str, span: Span) {
        if self.result.is_ok() {
            self.result = Err(DescriptionError::new(
                DescriptionErrorType::UnsupportedConstruct(msg),
                span,
            ));
        }
    }
}

mod expr;
mod func;
mod path;
mod stmt;

mod property;

fn error(msg: String, span: crate::wir::WSpan) -> crate::Error {
    crate::Error {
        ty: crate::ErrorType::IIRConversionError(msg),
        span,
    }
}

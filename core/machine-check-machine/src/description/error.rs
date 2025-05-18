use proc_macro2::Span;

use crate::{support::error_list::ErrorList, ErrorType, MachineError};

#[derive(thiserror::Error, Debug, Clone)]
pub(super) enum DescriptionErrorType {
    #[error("{0}")]
    MacroError(String),
    #[error("{0}")]
    MacroParseError(syn::Error),
    #[error("{0} not supported")]
    UnsupportedConstruct(&'static str),
    #[error("{0}")]
    IllegalConstruct(String),
    #[error("machine-check: Could not infer variable type")]
    InferenceFailure,
    #[error("{0}")]
    TypeConversionError(&'static str),
}

impl From<Error> for MachineError {
    fn from(error: Error) -> Self {
        MachineError {
            ty: ErrorType::DescriptionError(format!("{}", error)),
            span: error.span,
        }
    }
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("{ty}")]
pub(super) struct Error {
    pub ty: DescriptionErrorType,
    pub span: Span,
}

impl Error {
    pub fn new(ty: DescriptionErrorType, span: Span) -> Self {
        Self { ty, span }
    }

    pub fn unsupported_construct(msg: &'static str, span: Span) -> Self {
        Self::new(DescriptionErrorType::UnsupportedConstruct(msg), span)
    }
}

pub(super) type Errors = ErrorList<Error>;

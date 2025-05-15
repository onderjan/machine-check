use proc_macro2::Span;

use crate::{support::errors::Errors, ErrorType, MachineError};

#[derive(thiserror::Error, Debug, Clone)]
pub enum DescriptionErrorType {
    #[error("Unknown macro")]
    UnknownMacro,
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

impl From<DescriptionError> for MachineError {
    fn from(error: DescriptionError) -> Self {
        MachineError {
            ty: ErrorType::DescriptionError(format!("{}", error)),
            span: error.span,
        }
    }
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("{span:?}: {ty}")]
pub struct DescriptionError {
    pub ty: DescriptionErrorType,
    pub span: Span,
}

impl DescriptionError {
    pub fn new(ty: DescriptionErrorType, span: Span) -> Self {
        Self { ty, span }
    }
}

pub type DescriptionErrors = Errors<DescriptionError>;

use proc_macro2::Span;

use crate::BackwardError;

#[derive(thiserror::Error, Debug, Clone)]
pub enum ErrorType {
    #[error("machine-check: Cannot parse module without content")]
    ModuleWithoutContent,

    #[error("machine-check error: Unknown macro")]
    UnknownMacro,
    #[error("{0}")]
    MacroError(String),
    #[error("{0}")]
    MacroParseError(syn::Error),
    #[error("machine-check: {0}")]
    UnsupportedConstruct(String),
    #[error("machine-check: {0}")]
    IllegalConstruct(String),
    #[error("machine-check: Could not infer variable type")]
    InferenceFailure,
    #[error("machine-check (concrete conversion): {0}")]
    ConcreteConversionError(String),
    #[error("machine-check (forward conversion): {0}")]
    ForwardConversionError(String),
    #[error("machine-check (backward conversion): {0}")]
    BackwardConversionError(String),

    #[error("machine-check: {0}")]
    DescriptionError(String),

    #[error("machine-check internal error (SSA translation): {0}")]
    SsaInternal(String),
    #[error("machine-check internal error (forward translation): {0}")]
    ForwardInternal(String),
    #[error("machine-check internal error (backward translation): {0}")]
    BackwardInternal(String),
    #[error("machine-check internal error (rules): {0}")]
    RulesInternal(String),
}

impl From<BackwardError> for MachineError {
    fn from(error: BackwardError) -> Self {
        MachineError {
            ty: ErrorType::BackwardConversionError(format!("{}", error)),
            span: error.span,
        }
    }
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("{span:?}: {ty}")]
pub struct MachineError {
    pub ty: ErrorType,
    pub span: Span,
}

impl MachineError {
    pub fn new(ty: ErrorType, span: Span) -> Self {
        Self { ty, span }
    }
}

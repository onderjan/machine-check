use crate::wir::WSpan;

#[derive(thiserror::Error, Debug, Clone)]
pub enum ErrorType {
    #[error("machine-check: {0}")]
    ExpressionParseError(String),
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
    #[error("machine-check: {0}")]
    IIRConversionError(String),

    #[error("machine-check: {0}")]
    DescriptionError(String),
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("{ty}")]
pub struct Error {
    pub(crate) ty: ErrorType,
    pub(crate) span: WSpan,
}

impl Error {
    pub fn new(ty: ErrorType, span: WSpan) -> Self {
        Self { ty, span }
    }

    pub fn into_syn_error(self) -> syn::Error {
        syn::Error::new_spanned(self.span.syn_delimiters(), self.ty.to_string())
    }

    pub fn span(&self) -> WSpan {
        self.span
    }
}

mod conversion;
mod description;
mod from_syn;
mod property;

use std::collections::HashMap;

use machine_check_common::iir::ISubpropertyInfo;
use quote::ToTokens;
use syn::Item;

use crate::{
    support::error_list::ErrorList,
    wir::{WBasicType, WDescription, WIdent, WSpan, YConverted},
};

pub fn create_description(
    items: Vec<Item>,
) -> Result<(WDescription<YConverted>, Vec<String>), crate::Errors> {
    description::create_from_syn(items).map_err(Errors::convert_inner)
}

pub fn create_property_description(
    expr: syn::Expr,
    global_ident_types: &HashMap<WIdent, WBasicType>,
) -> Result<(WDescription<YConverted>, Vec<String>, Vec<ISubpropertyInfo>), crate::Errors> {
    property::create_from_syn(expr, global_ident_types).map_err(Errors::convert_inner)
}

#[derive(thiserror::Error, Debug, Clone)]
pub(super) enum ErrorType {
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
    CallConversionError(&'static str),
}

impl From<Error> for crate::Error {
    fn from(error: Error) -> crate::Error {
        crate::Error {
            ty: crate::ErrorType::DescriptionError(format!("{}", error)),
            span: error.span,
        }
    }
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("{ty}")]
pub(super) struct Error {
    pub ty: ErrorType,
    pub span: WSpan,
}

impl Error {
    pub fn new(ty: ErrorType, span: WSpan) -> Self {
        Self { ty, span }
    }

    pub fn unsupported_construct(msg: &'static str, span: WSpan) -> Self {
        Self::new(ErrorType::UnsupportedConstruct(msg), span)
    }

    pub fn unsupported_syn_construct(msg: &'static str, to_tokens: &impl ToTokens) -> Self {
        Self::unsupported_construct(msg, WSpan::from_syn(to_tokens))
    }
}

pub(super) type Errors = ErrorList<Error>;

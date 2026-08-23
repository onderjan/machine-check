mod conversion;
mod description;
mod from_syn;
mod property;

use indexmap::IndexMap;
use machine_check_common::PropertyMacros;
use quote::ToTokens;
use syn::Item;

pub use from_syn::{fold_partial_path, fold_type};

use crate::{
    context::{WContextBuilder, WLowContext},
    util::error_list::ErrorList,
    wir::{WIdent, WProperty, WSpan, WTypeId},
};

pub fn create_description(items: Vec<Item>) -> Result<WLowContext, crate::Errors> {
    description::description_from_syn(items).map_err(Errors::convert_inner)
}

pub fn create_property<D>(
    ctx: WContextBuilder,
    expr: syn::Expr,
    globals: &IndexMap<WIdent, WTypeId>,
    property_macros: &PropertyMacros<D>,
) -> Result<WProperty, crate::Errors> {
    property::create_from_syn(ctx, expr, globals, property_macros).map_err(Errors::convert_inner)
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
    #[error("Undefined variable '{0}'")]
    UndefinedVariable(String),
    #[error("Could not infer variable type")]
    InferenceFailure,
    #[error("{0}")]
    CallConversionError(&'static str),
    #[error("Unknown call function '{0}'")]
    UnknownCallFunction(String),
    #[error("Type cannot be called")]
    NotCallable,
    #[error("Expected {0} arguments, got {1}")]
    WrongNumberOfArguments(usize, usize),
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

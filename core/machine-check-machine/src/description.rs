mod attribute_disallower;
mod convert_indexing;
mod convert_to_ssa;
mod convert_total;
mod convert_types;
mod expand_macros;
mod from_syn;
mod infer_types;
mod resolve_use;

use quote::ToTokens;
use syn::Item;

use crate::{
    support::error_list::ErrorList,
    wir::{WDescription, WSpan, YConverted},
};

pub fn create_description(
    items: Vec<Item>,
) -> Result<(WDescription<YConverted>, Vec<String>), crate::Errors> {
    create_description_inner(items).map_err(Errors::convert_inner)
}

fn create_description_inner(
    mut items: Vec<Item>,
) -> Result<(WDescription<YConverted>, Vec<String>), Errors> {
    let mut macro_expander = expand_macros::MacroExpander::new();
    loop {
        resolve_use::resolve_use(&mut items)?;
        if !macro_expander.expand_macros(&mut items)? {
            break;
        }
    }

    resolve_use::remove_use(&mut items)?;

    /*println!(
        "Original syn string:\n{}",
        quote::ToTokens::into_token_stream(syn::File {
            shebang: None,
            attrs: Vec::new(),
            items: items.clone()
        })
    );
    println!("---");
    */

    let w_description = from_syn::from_syn(items.clone().into_iter())?;
    let w_description = convert_indexing::convert_indexing(w_description);
    let (w_description, panic_messages) = convert_total::convert_total(w_description);
    let w_description = convert_to_ssa::convert_to_ssa(w_description)?;
    let w_description = infer_types::infer_types(w_description)?;
    let w_description = convert_types::convert_types(w_description)?;

    /*println!(
        "Compared syn string:\n{}",
        quote::ToTokens::into_token_stream(w_description.clone().into_syn())
    );
    println!("---");*/

    //let items: Vec<Item> = w_description.into_syn().items;

    Ok((w_description, panic_messages))
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

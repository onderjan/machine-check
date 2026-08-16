use machine_check_common::iir::path::ISpan;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::hash::Hash;
use syn::Token;

pub trait WSpanned {
    fn wir_span(&self) -> WSpan;
}

/// WIR span structure for nice error spans.
///
/// The syn `Span` structure currently cannot produce spans for more than
/// one token in stable Rust due to some limitations. This structure
/// is similar to proc_macro2 `DelimSpan`, containing the first and last span.
///
/// This allows for error spans that span more than one token even on stable Rust.
#[derive(Clone, Copy, Debug)]
pub struct WSpan {
    first: Span,
    last: Span,
}

impl Hash for WSpan {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        // do nothing here, the spans are just for information
    }
}

impl PartialEq for WSpan {
    fn eq(&self, other: &Self) -> bool {
        // always return true, the spans are just for information
        true
    }
}

impl Eq for WSpan {}

impl WSpan {
    pub(crate) fn from_syn(to_tokens: &impl ToTokens) -> Self {
        let mut iter = to_tokens.into_token_stream().into_iter();
        let first = match iter.next() {
            Some(first) => first.span(),
            None => Span::call_site(),
        };
        let last = match iter.last() {
            Some(last) => last.span(),
            None => first,
        };

        Self { first, last }
    }

    pub(crate) fn from_span(span: Span) -> Self {
        WSpan {
            first: span,
            last: span,
        }
    }

    pub(crate) fn into_iir(self) -> ISpan {
        // TODO: convert to IIR span
        ISpan::Unspecified
    }

    pub(crate) fn from_delimiters(first: Span, last: Span) -> Self {
        WSpan { first, last }
    }

    pub(crate) fn call_site() -> Self {
        Self::from_span(Span::call_site())
    }

    pub(crate) fn first(&self) -> Span {
        self.first
    }

    pub(crate) fn syn_delimiters(&self) -> TokenStream {
        let first = Token![_](self.first).into_token_stream();
        let last = Token![_](self.last).into_token_stream();
        TokenStream::from_iter([first, last])
    }
}

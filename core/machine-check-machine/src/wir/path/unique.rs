use proc_macro2::Span;

use crate::wir::{WIdent, WSpan, WSpanned, WTotalPath, WTotalPathSegment};
use std::fmt::Debug;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WUniquePath {
    pub leading_colon: Option<WSpan>,
    pub segments: Vec<WIdent>,
}

impl WUniquePath {
    pub fn into_total(self) -> WTotalPath {
        WTotalPath {
            leading_colon: self.leading_colon,
            segments: self
                .segments
                .into_iter()
                .map(|ident| WTotalPathSegment {
                    ident,
                    generics: None,
                })
                .collect(),
        }
    }
}

impl Debug for WUniquePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.leading_colon.is_some() {
            f.write_str("::")?;
        }

        let mut first = true;
        for ident in &self.segments {
            if first {
                first = false;
            } else {
                f.write_str("::")?;
            }
            Debug::fmt(&ident, f)?;
        }
        Ok(())
    }
}

impl WSpanned for WUniquePath {
    fn wir_span(&self) -> WSpan {
        let first = if let Some(leading_colon) = self.leading_colon {
            leading_colon.first()
        } else {
            self.segments
                .first()
                .map(|first| first.span())
                .unwrap_or(Span::call_site())
        };
        WSpan::from_delimiters(
            first,
            self.segments
                .last()
                .map(|last| last.span())
                .unwrap_or(Span::call_site()),
        )
    }
}

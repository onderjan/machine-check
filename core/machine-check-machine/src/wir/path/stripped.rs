use crate::wir::{WIdent, WSpan};
use std::fmt::Debug;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WStrippedPath {
    pub leading_colon: Option<WSpan>,
    pub segments: Vec<WIdent>,
}

impl WStrippedPath {
    /*pub fn into_total(self) -> WTotalPath {
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
    }*/

    pub fn from_ident(ident: WIdent) -> Self {
        Self {
            leading_colon: None,
            segments: vec![ident],
        }
    }

    pub fn span(&self) -> WSpan {
        if let Some(leading_colon) = self.leading_colon {
            leading_colon
        } else {
            self.segments
                .first()
                .map(|first| first.span())
                .unwrap_or(WSpan::call_site())
        }
    }
}

impl Debug for WStrippedPath {
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

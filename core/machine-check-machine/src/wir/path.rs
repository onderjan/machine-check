use proc_macro2::Span;
use std::fmt::Debug;
use std::hash::Hash;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, GenericArgument, Path, PathArguments,
    PathSegment, Token, Type,
};

use crate::wir::{ident::WIdent, WSpan, WTypeId};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WPathGenerics {
    pub turbofish: Option<WSpan>,
    pub arguments: Vec<WTypeId>,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WPathSegment {
    pub ident: WIdent,
    pub generics: Option<WPathGenerics>,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WPath {
    pub leading_colon: Option<WSpan>,
    pub segments: Vec<WPathSegment>,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WStrippedPath {
    pub leading_colon: Option<WSpan>,
    pub segments: Vec<WIdent>,
}

impl Debug for WPathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.ident, f)?;
        if let Some(generics) = &self.generics {
            if generics.turbofish.is_some() {
                write!(f, "::")?;
            }
            write!(f, "<")?;
            let mut first = true;
            for arg in &generics.arguments {
                if first {
                    first = false;
                } else {
                    write!(f, ",")?;
                }
                Debug::fmt(&arg, f)?;
            }

            write!(f, ">")?;
        }
        Ok(())
    }
}

impl WPath {
    pub fn from_ident(ident: WIdent) -> Self {
        WPath {
            leading_colon: None,
            segments: vec![WPathSegment {
                ident,
                generics: None,
            }],
        }
    }

    pub fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Path {
        let leading_span = if let Some(leading_colon) = self.leading_colon {
            leading_colon.first()
        } else {
            Span::call_site()
        };
        Path {
            leading_colon: if self.leading_colon.is_some() {
                Some(Token![::](leading_span))
            } else {
                None
            },

            segments: Punctuated::from_iter(self.segments.into_iter().map(|segment| {
                let arguments = match segment.generics {
                    Some(generics) => {
                        let span = segment.ident.span();
                        let colon2_token = if generics.turbofish.is_some() {
                            Some(Token![::](span.first()))
                        } else {
                            None
                        };
                        let args = Punctuated::from_iter(
                            generics
                                .arguments
                                .into_iter()
                                .map(|arg| GenericArgument::Type(type_fn(arg))),
                        );
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token,
                            lt_token: Token![<](span.first()),
                            args,
                            gt_token: Token![>](span.first()),
                        })
                    }
                    None => PathArguments::None,
                };
                PathSegment {
                    ident: segment.ident.into(),
                    arguments,
                }
            })),
        }
    }

    pub fn resolve_self(self, self_path: &WPath) -> Self {
        if self.leading_colon.is_some() {
            return self;
        }

        let Some(first_segment) = self.segments.first() else {
            return self;
        };

        if first_segment.ident.name() == "Self" && first_segment.generics.is_none() {
            //let span = first_segment.ident.span();
            let mut result = self_path.clone();
            //result.set_span(span);
            result.segments.extend(self.segments.into_iter().skip(1));

            result
        } else {
            self
        }
    }

    pub fn without_generics(self) -> WStrippedPath {
        WStrippedPath {
            leading_colon: self.leading_colon,
            segments: self
                .segments
                .into_iter()
                .map(|segment| segment.ident)
                .collect(),
        }
    }

    pub fn span(&self) -> WSpan {
        if let Some(leading_colon) = self.leading_colon {
            leading_colon
        } else {
            self.segments
                .first()
                .map(|first| first.ident.span())
                .unwrap_or(WSpan::call_site())
        }
    }

    pub fn get_ident(&self) -> Option<&WIdent> {
        if self.leading_colon.is_none()
            && self.segments.len() == 1
            && self.segments[0].generics.is_none()
        {
            Some(&self.segments[0].ident)
        } else {
            None
        }
    }

    /// Returns true if the path is absolute and the segment idents start with the given strings.
    ///
    /// Does not take generics into account.
    pub fn starts_with_absolute(&self, segments: &[&str]) -> bool {
        if self.leading_colon.is_none() {
            return false;
        }
        if self.segments.len() < segments.len() {
            return false;
        }
        for (self_segment, other_segment) in self.segments.iter().zip(segments.iter()) {
            if self_segment.ident.name() != *other_segment {
                return false;
            }
        }
        true
    }

    /// Returns true if the path is absolute and the segment idents match the given strings.
    ///
    /// Does not take generics into account.
    pub fn matches_absolute(&self, segments: &[&str]) -> bool {
        if self.leading_colon.is_none() {
            return false;
        }
        if self.segments.len() != segments.len() {
            return false;
        }
        for (self_segment, other_segment) in self.segments.iter().zip(segments.iter()) {
            if self_segment.ident.name() != *other_segment {
                return false;
            }
        }
        true
    }
}

impl Debug for WPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut skip_leading = self.leading_colon.is_none();
        for segment in &self.segments {
            if skip_leading {
                skip_leading = false;
            } else {
                write!(f, "::")?;
            }
            Debug::fmt(&segment, f)?;
        }
        Ok(())
    }
}

impl WStrippedPath {
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

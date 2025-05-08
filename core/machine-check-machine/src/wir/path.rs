use proc_macro2::Span;
use std::hash::Hash;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, Expr, ExprLit, GenericArgument, Ident,
    Lit, LitInt, Path, PathArguments, PathSegment, Token,
};

use super::{IntoSyn, WSimpleType};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WPath {
    pub leading_colon: bool,
    pub segments: Vec<WPathSegment>,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WPathSegment {
    pub ident: WIdent,
    pub generics: Option<WGenerics>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WGenerics {
    pub leading_colon: bool,
    pub inner: Vec<WGeneric>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WGeneric {
    Type(WSimpleType),
    Const(u32),
}

impl WPath {
    /// Returns true if the path is absolute and the segment idents match the given strings.
    ///
    /// Does not take generics into account.
    pub fn matches_absolute(&self, segments: &[&str]) -> bool {
        if !self.leading_colon {
            return false;
        }
        if self.segments.len() != segments.len() {
            return false;
        }
        for (self_segment, other_segment) in self.segments.iter().zip(segments.iter()) {
            if self_segment.ident.name != *other_segment {
                return false;
            }
        }
        true
    }

    /// Returns true if the path is relative and the segment idents match the given strings.
    ///
    /// Does not take generics into account.
    pub fn matches_relative(&self, segments: &[&str]) -> bool {
        if self.leading_colon {
            return false;
        }
        if self.segments.len() != segments.len() {
            return false;
        }
        for (self_segment, other_segment) in self.segments.iter().zip(segments.iter()) {
            if self_segment.ident.name != *other_segment {
                return false;
            }
        }
        true
    }

    /// Creates a new absolute path from the given segment names with the given span.
    ///
    /// There are no generics in the path after creation.
    pub fn new_absolute(segments: &[&str], span: Span) -> WPath {
        WPath {
            leading_colon: true,
            segments: segments
                .iter()
                .map(|name| WPathSegment {
                    ident: WIdent {
                        name: String::from(*name),
                        span,
                    },
                    generics: None,
                })
                .collect(),
        }
    }

    pub fn from_ident(ident: WIdent) -> Self {
        WPath {
            leading_colon: false,
            segments: vec![WPathSegment {
                ident,
                generics: None,
            }],
        }
    }

    pub fn span(&self) -> Span {
        // TODO: correct span
        if let Some(last_segment) = self.segments.last() {
            last_segment.ident.span
        } else {
            Span::call_site()
        }
    }
}

impl From<WPath> for Path {
    fn from(path: WPath) -> Self {
        let span = Span::call_site();
        Path {
            leading_colon: if path.leading_colon {
                Some(Token![::](span))
            } else {
                None
            },

            segments: Punctuated::from_iter(path.segments.into_iter().map(|segment| {
                let arguments = match segment.generics {
                    Some(generics) => {
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: if generics.leading_colon {
                                Some(Token![::](span))
                            } else {
                                None
                            },
                            lt_token: Token![<](span),
                            args: Punctuated::from_iter(generics.inner.into_iter().map(
                                |generic| match generic {
                                    WGeneric::Type(ty) => GenericArgument::Type(ty.into_syn()),
                                    WGeneric::Const(value) => {
                                        GenericArgument::Const(Expr::Lit(ExprLit {
                                            attrs: Vec::new(),
                                            lit: Lit::Int(LitInt::new(&value.to_string(), span)),
                                        }))
                                    }
                                },
                            )),
                            gt_token: Token![>](span),
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
}

#[derive(Clone, Debug)]
pub struct WIdent {
    pub name: String,
    pub span: Span,
}

impl PartialEq for WIdent {
    fn eq(&self, other: &Self) -> bool {
        // do not consider span for equality
        self.name == other.name
    }
}

impl Eq for WIdent {}

impl Hash for WIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // do not consider span for the hash
        // this is fine as it just means two idents
        // with different spans will hash to the same value
        self.name.hash(state);
    }
}

impl WIdent {
    pub fn into_path(self) -> WPath {
        WPath::from_ident(self)
    }
}

impl From<WIdent> for Ident {
    fn from(ident: WIdent) -> Self {
        Ident::new(&ident.name, ident.span)
    }
}

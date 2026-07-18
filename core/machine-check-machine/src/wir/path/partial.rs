use proc_macro2::Span;
use std::fmt::Debug;
use std::hash::Hash;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, Expr, ExprLit, GenericArgument, Lit,
    LitInt, Path, PathArguments, PathSegment, Token, Type, TypeInfer,
};

use crate::wir::{
    ident::WIdent, WPartialType, WPath, WPathArgument, WPathGenerics, WPathSegment, WSpan,
};

#[derive(Clone, Hash)]
pub enum WPartialArgument {
    Type(WPartialType),
    Uint(u32, WSpan),
    Infer(WSpan),
}

impl From<WPartialArgument> for GenericArgument {
    fn from(value: WPartialArgument) -> Self {
        match value {
            WPartialArgument::Type(ty) => GenericArgument::Type(ty.into()),
            WPartialArgument::Uint(value, span) => GenericArgument::Const(Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit: Lit::Int(LitInt::new(&value.to_string(), span.first())),
            })),
            WPartialArgument::Infer(span) => GenericArgument::Type(Type::Infer(TypeInfer {
                underscore_token: Token![_](span.first()),
            })),
        }
    }
}

impl WPartialArgument {
    pub fn into_total(self) -> Result<WPathArgument, ()> {
        match self {
            WPartialArgument::Type(ty) => Ok(WPathArgument::Type(ty.into_total()?)),
            WPartialArgument::Uint(num, span) => Ok(WPathArgument::Uint(num, span)),
            WPartialArgument::Infer(_) => Err(()),
        }
    }
}

#[derive(Clone, Hash)]
pub struct WPartialGenerics {
    pub turbofish: Option<WSpan>,
    pub arguments: Vec<WPartialArgument>,
}

impl WPartialGenerics {
    pub fn into_total(self) -> Result<WPathGenerics, ()> {
        let mut arguments = Vec::new();
        for arg in self.arguments {
            arguments.push(arg.into_total()?);
        }

        Ok(WPathGenerics {
            turbofish: self.turbofish,
            arguments,
        })
    }
}

#[derive(Clone, Hash)]
pub struct WPartialSegment {
    pub ident: WIdent,
    pub generics: Option<WPartialGenerics>,
}

#[derive(Clone, Hash)]
pub struct WPartialPath {
    pub leading_colon: Option<WSpan>,
    pub segments: Vec<WPartialSegment>,
}

impl Debug for WPartialArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type(ty) => Debug::fmt(&ty, f),
            Self::Uint(num, _span) => write!(f, "{}", num),
            Self::Infer(_span) => write!(f, "_"),
        }
    }
}

impl Debug for WPartialSegment {
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

impl From<WPartialPath> for Path {
    fn from(path: WPartialPath) -> Self {
        let leading_span = if let Some(leading_colon) = path.leading_colon {
            leading_colon.first()
        } else {
            Span::call_site()
        };
        Path {
            leading_colon: if path.leading_colon.is_some() {
                Some(Token![::](leading_span))
            } else {
                None
            },

            segments: Punctuated::from_iter(path.segments.into_iter().map(|segment| {
                let arguments = match segment.generics {
                    Some(generics) => {
                        let span = segment.ident.span();
                        let colon2_token = if generics.turbofish.is_some() {
                            Some(Token![::](span))
                        } else {
                            None
                        };
                        let args = Punctuated::from_iter(
                            generics
                                .arguments
                                .into_iter()
                                .map(Into::<GenericArgument>::into),
                        );
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token,
                            lt_token: Token![<](span),
                            args,
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

impl Debug for WPartialPath {
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

impl WPartialPath {
    pub fn into_total(self) -> Result<WPath, ()> {
        let mut segments = Vec::new();
        for segment in self.segments {
            let generics = if let Some(generics) = segment.generics {
                Some(generics.into_total()?)
            } else {
                None
            };

            segments.push(WPathSegment {
                ident: segment.ident,
                generics,
            });
        }
        Ok(WPath {
            leading_colon: self.leading_colon,
            segments,
        })
    }

    pub fn wir_span(&self) -> WSpan {
        let first = if let Some(leading_colon) = self.leading_colon {
            leading_colon.first()
        } else {
            self.segments
                .first()
                .map(|first| first.ident.span())
                .unwrap_or(Span::call_site())
        };
        WSpan::from_delimiters(
            first,
            self.segments
                .last()
                .map(|last| last.ident.span())
                .unwrap_or(Span::call_site()),
        )
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

use proc_macro2::Span;
use std::fmt::Debug;
use std::hash::Hash;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, Expr, ExprLit, GenericArgument, Lit,
    LitInt, Path, PathArguments, PathSegment, Token, Type, TypeInfer,
};

use crate::wir::{
    ident::WIdent, WPartialType, WSpan, WStrippedPath, WTotalPath, WTotalPathArgument,
    WTotalPathGenerics, WTotalPathSegment,
};

#[derive(Clone, Hash)]
pub enum WPartialPathArgument {
    Type(WPartialType),
    Uint(u32, WSpan),
    Infer(WSpan),
}

#[derive(Clone, Hash)]
pub struct WPartialPathSegment {
    pub ident: WIdent,
    pub generics: Option<WPartialPathGenerics>,
}

#[derive(Clone, Hash)]
pub struct WPartialPathGenerics {
    pub turbofish: Option<WSpan>,
    pub arguments: Vec<WPartialPathArgument>,
}

#[derive(Clone, Hash)]
pub struct WPartialPath {
    pub leading_colon: Option<WSpan>,
    pub segments: Vec<WPartialPathSegment>,
}

impl WPartialPathArgument {
    pub fn set_span(&mut self, new_span: WSpan) {
        match self {
            WPartialPathArgument::Type(ty) => {
                ty.set_span(new_span);
            }
            WPartialPathArgument::Uint(_value, span) => *span = new_span,
            WPartialPathArgument::Infer(span) => *span = new_span,
        }
    }

    pub fn into_total(self) -> Result<WTotalPathArgument, ()> {
        match self {
            WPartialPathArgument::Type(ty) => Ok(WTotalPathArgument::Type(ty.try_into_total()?)),
            WPartialPathArgument::Uint(num, span) => Ok(WTotalPathArgument::Uint(num, span)),
            WPartialPathArgument::Infer(_) => Err(()),
        }
    }

    fn into_syn(self) -> GenericArgument {
        match self {
            WPartialPathArgument::Type(ty) => GenericArgument::Type(ty.into_syn()),
            WPartialPathArgument::Uint(value, span) => GenericArgument::Const(Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit: Lit::Int(LitInt::new(&value.to_string(), span.first())),
            })),
            WPartialPathArgument::Infer(span) => GenericArgument::Type(Type::Infer(TypeInfer {
                underscore_token: Token![_](span.first()),
            })),
        }
    }
}

impl Debug for WPartialPathArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type(ty) => Debug::fmt(&ty, f),
            Self::Uint(num, _span) => write!(f, "{}", num),
            Self::Infer(_span) => write!(f, "_"),
        }
    }
}

impl WPartialPathGenerics {
    pub fn into_total(self) -> Result<WTotalPathGenerics, ()> {
        let mut arguments = Vec::new();
        for arg in self.arguments {
            arguments.push(arg.into_total()?);
        }

        Ok(WTotalPathGenerics {
            turbofish: self.turbofish,
            arguments,
        })
    }

    pub fn set_span(&mut self, span: WSpan) {
        self.turbofish.map(|_| span);
        for arg in &mut self.arguments {
            arg.set_span(span);
        }
    }
}

impl Debug for WPartialPathSegment {
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

impl WPartialPath {
    pub fn into_syn(self) -> Path {
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
                                .map(WPartialPathArgument::into_syn),
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

    pub fn resolve_self(self, self_path: &WTotalPath) -> Self {
        if self.leading_colon.is_some() {
            return self;
        }

        let Some(first_segment) = self.segments.first() else {
            return self;
        };

        if first_segment.ident.name() == "Self" && first_segment.generics.is_none() {
            let span = first_segment.ident.span();
            let mut result = self_path.clone().into_partial();
            result.set_span(span);
            result.segments.extend(self.segments.into_iter().skip(1));

            result
        } else {
            self
        }
    }

    pub fn set_span(&mut self, span: WSpan) {
        self.leading_colon.map(|_| span);
        for segment in &mut self.segments {
            segment.ident.set_span(span);
            if let Some(generics) = &mut segment.generics {
                generics.set_span(span);
            }
        }
    }

    pub fn try_into_total(self) -> Result<WTotalPath, ()> {
        let mut segments = Vec::new();
        for segment in self.segments {
            let generics = if let Some(generics) = segment.generics {
                Some(generics.into_total()?)
            } else {
                None
            };

            segments.push(WTotalPathSegment {
                ident: segment.ident,
                generics,
            });
        }
        Ok(WTotalPath {
            leading_colon: self.leading_colon,
            segments,
        })
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

use proc_macro2::Span;
use std::fmt::Debug;
use std::hash::Hash;
use syn::{
    punctuated::Punctuated, Expr, ExprLit, GenericArgument, Lit, LitInt, Path, PathArguments,
    PathSegment, Token,
};

use crate::wir::{
    ident::WIdent, WPartialPath, WPartialPathArgument, WPartialPathGenerics, WPartialPathSegment,
    WSpan, WSpanned, WTotalType, WUniquePath,
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum WTotalPathArgument {
    Type(WTotalType),
    Uint(u32, WSpan),
}

impl From<WTotalPathArgument> for GenericArgument {
    fn from(value: WTotalPathArgument) -> Self {
        match value {
            WTotalPathArgument::Type(ty) => GenericArgument::Type(ty.into_syn()),
            WTotalPathArgument::Uint(value, span) => GenericArgument::Const(Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit: Lit::Int(LitInt::new(&value.to_string(), span.first())),
            })),
        }
    }
}

impl WTotalPathArgument {
    pub fn into_partial(self) -> WPartialPathArgument {
        match self {
            WTotalPathArgument::Type(ty) => WPartialPathArgument::Type(ty.into_partial()),
            WTotalPathArgument::Uint(value, span) => WPartialPathArgument::Uint(value, span),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WTotalPathGenerics {
    pub turbofish: Option<WSpan>,
    pub arguments: Vec<WTotalPathArgument>,
}

impl WTotalPathGenerics {
    pub fn into_partial(self) -> WPartialPathGenerics {
        WPartialPathGenerics {
            turbofish: self.turbofish,
            arguments: self
                .arguments
                .into_iter()
                .map(|arg| arg.into_partial())
                .collect(),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WTotalPathSegment {
    pub ident: WIdent,
    pub generics: Option<WTotalPathGenerics>,
}

impl WTotalPathSegment {
    pub fn into_partial(self) -> WPartialPathSegment {
        WPartialPathSegment {
            ident: self.ident,
            generics: self.generics.map(|generics| generics.into_partial()),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WTotalPath {
    pub leading_colon: Option<WSpan>,
    pub segments: Vec<WTotalPathSegment>,
}

impl WTotalPath {
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

    /// Returns true if the path is relative and the segment idents match the given strings.
    ///
    /// Does not take generics into account.
    pub fn matches_relative(&self, segments: &[&str]) -> bool {
        if self.leading_colon.is_some() {
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

    pub fn from_ident(ident: WIdent) -> Self {
        WTotalPath {
            leading_colon: None,
            segments: vec![WTotalPathSegment {
                ident,
                generics: None,
            }],
        }
    }

    pub fn span(&self) -> Span {
        // TODO: correct span
        if let Some(last_segment) = self.segments.last() {
            last_segment.ident.span()
        } else {
            Span::call_site()
        }
    }

    pub fn segments_strs(&self) -> impl Iterator<Item = &str> {
        self.segments.iter().map(|segment| segment.ident.name())
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

    pub fn into_partial(self) -> WPartialPath {
        WPartialPath {
            leading_colon: self.leading_colon,
            segments: self
                .segments
                .into_iter()
                .map(|segment| segment.into_partial())
                .collect(),
        }
    }

    pub fn without_generics(self) -> WUniquePath {
        WUniquePath {
            leading_colon: self.leading_colon,
            segments: self
                .segments
                .into_iter()
                .map(|segment| segment.ident)
                .collect(),
        }
    }
}

impl WSpanned for WTotalPath {
    fn wir_span(&self) -> WSpan {
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
}

impl From<WTotalPath> for Path {
    fn from(path: WTotalPath) -> Self {
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

            segments: Punctuated::from_iter(path.segments.into_iter().map(|segment| PathSegment {
                ident: segment.ident.into(),
                arguments: PathArguments::None,
            })),
        }
    }
}

impl Debug for WTotalPathArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type(ty) => Debug::fmt(&ty, f),
            Self::Uint(num, _span) => write!(f, "{}", num),
        }
    }
}

impl Debug for WTotalPathSegment {
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

impl Debug for WTotalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.leading_colon.is_some() {
            f.write_str("::")?;
        }

        let mut first = true;
        for segment in &self.segments {
            if first {
                first = false;
            } else {
                f.write_str("::")?;
            }
            Debug::fmt(&segment, f)?;
        }
        Ok(())
    }
}

use std::fmt::{Debug, Write};
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, Expr, ExprLit, GenericArgument, Lit,
    LitInt, Path, PathArguments, PathSegment, Token, Type, TypeInfer, TypePath, TypeReference,
};

use crate::wir::{WIdent, WSpan, WStrippedPath, WTotalType, WTypeId};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WTypePathSegment {
    pub ident: WIdent,
    pub generics: Option<Vec<WTypeId>>,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WTypePath {
    pub leading_colon: Option<WSpan>,
    pub segments: Vec<WTypePathSegment>,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum WPartialType {
    Path(WTypePath),
    Reference(WTypeId, WSpan),
    Infer(WSpan),
    Number(u32, WSpan),
}

impl WTypePathSegment {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> PathSegment {
        let ident = self.ident.to_syn();
        let span = ident.span();
        let arguments = if let Some(arguments) = self.generics {
            let args = Punctuated::from_iter(
                arguments
                    .iter()
                    .map(|arg| GenericArgument::Type(type_fn(arg.clone()))),
            );
            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Token![<](span),
                args,
                gt_token: Token![>](span),
            })
        } else {
            PathArguments::None
        };

        PathSegment { ident, arguments }
    }
}

impl WTypePath {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> TypePath {
        let leading_colon = self.leading_colon.map(|c| Token![::](c.first()));
        let segments = Punctuated::from_iter(
            self.segments
                .into_iter()
                .map(|segment| segment.into_typed_syn(type_fn)),
        );

        let path = Path {
            leading_colon,
            segments,
        };

        TypePath { qself: None, path }
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
}

impl WPartialType {
    pub fn span(&self) -> WSpan {
        match self {
            WPartialType::Path(path) => {
                if let Some(last) = path.segments.last() {
                    last.ident.span()
                } else {
                    WSpan::call_site()
                }
            }
            WPartialType::Reference(_inner, span) => *span,
            WPartialType::Infer(span) => *span,
            WPartialType::Number(_num, span) => *span,
        }
    }

    /*pub fn is_fully_inferred(&self) -> bool {
        match self {
            WPartialType::Path(path) => {
                for segment in &path.segments {
                    if let Some(generics) = &segment.generics {
                        for argument in &generics.arguments {
                            if let WPartialPathArgument::Infer(_) = argument {
                                return false;
                            }
                        }
                    }
                }
                true
            }
            WPartialType::Infer(_) => false,
            WPartialType::Reference(inner) => inner.is_fully_inferred(),
        }
    }*/

    pub fn try_into_total(self) -> Result<WTotalType, ()> {
        match self {
            WPartialType::Path(path) => Ok(WTotalType::Path(path)),
            WPartialType::Reference(inner, span) => Ok(WTotalType::Reference(inner, span)),
            WPartialType::Infer(_span) => Err(()),
            WPartialType::Number(num, span) => Ok(WTotalType::Number(num, span)),
        }
    }

    pub fn into_typed_syn_argument(self, type_fn: &impl Fn(WTypeId) -> Type) -> GenericArgument {
        match self {
            WPartialType::Path(type_path) => {
                let type_path = type_path.into_typed_syn(type_fn);
                GenericArgument::Type(Type::Path(type_path))
            }
            WPartialType::Reference(ty, span) => {
                let elem = Box::new(type_fn(ty));
                GenericArgument::Type(Type::Reference(TypeReference {
                    and_token: Token![&](span.first()),
                    lifetime: None,
                    mutability: None,
                    elem,
                }))
            }
            WPartialType::Infer(span) => GenericArgument::Type(Type::Infer(TypeInfer {
                underscore_token: Token![_](span.first()),
            })),
            WPartialType::Number(num, span) => GenericArgument::Const(Expr::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Int(LitInt::new(&num.to_string(), span.first())),
            })),
        }
    }

    pub fn into_typed_syn_type(self, type_fn: &impl Fn(WTypeId) -> Type) -> Type {
        match self.into_typed_syn_argument(type_fn) {
            GenericArgument::Type(ty) => ty,
            _ => panic!("Cannot convert sort into syn type"),
        }
    }
}

impl Debug for WTypePathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.ident, f)?;
        if let Some(generics) = &self.generics {
            f.write_char('<')?;
            let mut first = true;
            for ty in generics {
                if first {
                    first = false;
                } else {
                    f.write_char(',')?;
                }
                Debug::fmt(&ty, f)?;
            }
            f.write_char('>')?;
        }
        Ok(())
    }
}

impl Debug for WTypePath {
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

impl Debug for WPartialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => Debug::fmt(&path, f),
            Self::Reference(inner, _span) => {
                write!(f, "&")?;
                Debug::fmt(inner, f)
            }
            Self::Infer(_span) => write!(f, "_"),
            Self::Number(num, _span) => Debug::fmt(num, f),
        }
    }
}

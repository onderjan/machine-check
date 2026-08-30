use std::fmt::{Debug, Write};
use syn::Type;

use crate::wir::{WIdent, WSpan, WStrippedPath};

use super::IntoTypedSyn;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WTypeId(usize);

impl WTypeId {
    pub fn from_index(index: usize) -> WTypeId {
        Self(index)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

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
pub enum WType {
    Path(WTypePath),
    Reference(WTypeId, WSpan),
    Number(u32, WSpan),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum WInferenceType {
    Inferred(WType),
    Infer(WSpan),
}

impl WTypePath {
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

impl WType {
    pub fn span(&self) -> WSpan {
        match self {
            WType::Path(path) => {
                if let Some(last) = path.segments.last() {
                    last.ident.span()
                } else {
                    WSpan::call_site()
                }
            }
            WType::Reference(_inner, span) => *span,
            WType::Number(_num, span) => *span,
        }
    }
}

impl WInferenceType {
    pub fn span(&self) -> WSpan {
        match self {
            WInferenceType::Inferred(ty) => ty.span(),
            WInferenceType::Infer(span) => *span,
        }
    }

    pub fn try_into_total(self) -> Result<WType, ()> {
        match self {
            WInferenceType::Inferred(ty) => Ok(ty),
            WInferenceType::Infer(_span) => Err(()),
        }
    }
}

impl IntoTypedSyn<Type> for WTypeId {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Type {
        type_fn(self)
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

impl Debug for WType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => Debug::fmt(&path, f),
            Self::Reference(type_id, _span) => {
                write!(f, "&")?;
                Debug::fmt(&type_id, f)
            }
            WType::Number(num, _span) => Debug::fmt(&num, f),
        }
    }
}

impl Debug for WInferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inferred(ty) => Debug::fmt(&ty, f),
            Self::Infer(_span) => write!(f, "_"),
        }
    }
}

impl Debug for WTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

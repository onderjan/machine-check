use proc_macro2::Span;
use std::hash::Hash;
use syn::{punctuated::Punctuated, Expr, ExprPath, Ident, Path, PathArguments, PathSegment, Token};

use super::IntoSyn;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WPath {
    pub leading_colon: bool,
    pub segments: Vec<WPathSegment>,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WPathSegment {
    pub ident: WIdent,
}

impl WPath {
    /// Returns true if the path is absolute and the segment idents start with the given strings.
    ///
    /// Does not take generics into account.
    pub fn starts_with_absolute(&self, segments: &[&str]) -> bool {
        if !self.leading_colon {
            return false;
        }
        if self.segments.len() < segments.len() {
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

    pub fn from_ident(ident: WIdent) -> Self {
        WPath {
            leading_colon: false,
            segments: vec![WPathSegment { ident }],
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

    pub fn segments_strs(&self) -> impl Iterator<Item = &str> {
        self.segments
            .iter()
            .map(|segment| segment.ident.name.as_str())
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

            segments: Punctuated::from_iter(path.segments.into_iter().map(|segment| PathSegment {
                ident: segment.ident.into(),
                arguments: PathArguments::None,
            })),
        }
    }
}

#[derive(Clone, Debug)]
pub struct WIdent {
    name: String,
    span: Span,
}

impl WIdent {
    pub fn new(name: String, span: Span) -> Self {
        Self { name, span }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn from_syn_ident(ident: Ident) -> Self {
        Self {
            name: ident.to_string(),
            span: ident.span(),
        }
    }

    pub fn into_path(self) -> WPath {
        WPath::from_ident(self)
    }

    pub fn mck_prefixed(&self, prefix: &str) -> WIdent {
        let orig_ident_str = self.name();
        // make sure everything is prefixed by __mck_ only once at the start
        let stripped_ident_str = orig_ident_str
            .strip_prefix("__mck_")
            .unwrap_or(orig_ident_str);

        WIdent::new(
            format!("__mck_{}_{}", prefix, stripped_ident_str),
            self.span(),
        )
    }
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

impl PartialOrd for WIdent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WIdent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // do not consider span for comparison
        self.name.cmp(&other.name)
    }
}

impl From<WIdent> for Ident {
    fn from(ident: WIdent) -> Self {
        Ident::new(&ident.name, ident.span)
    }
}

impl IntoSyn<Expr> for WIdent {
    fn into_syn(self) -> Expr {
        Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: Path {
                leading_colon: None,
                segments: Punctuated::from_iter(vec![PathSegment {
                    ident: self.into(),
                    arguments: PathArguments::None,
                }]),
            },
        })
    }
}

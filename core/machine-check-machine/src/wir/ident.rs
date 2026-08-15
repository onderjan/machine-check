use machine_check_common::iir::path::IIdent;
use proc_macro2::Span;
use std::fmt::Debug;
use std::hash::Hash;
use syn::{punctuated::Punctuated, Expr, ExprPath, Ident, Path, PathArguments, PathSegment, Type};

use crate::wir::{IntoSyn, WPath, WSpan, WSpanned, WTypeId};

#[derive(Clone)]
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

    pub fn set_span(&mut self, span: Span) {
        self.span = span;
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

    pub fn to_syn_ident(&self) -> Ident {
        Ident::new(&self.name, self.span)
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

    pub fn into_iir(self) -> IIdent {
        IIdent::new(self.name, WSpan::from_span(self.span).into_iir())
    }
}

impl Debug for WIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // just print the name
        f.write_str(&self.name)
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
    fn into_syn(self, _type_fn: &impl Fn(WTypeId) -> Type) -> Expr {
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

impl WSpanned for WIdent {
    fn wir_span(&self) -> super::WSpan {
        WSpan::from_span(self.span)
    }
}

use proc_macro2::Span;
use syn::Lit;

use super::{WIdent, WPath};

#[derive(Clone, Debug, Hash)]
pub enum WExpr {
    Move(WIdent),
    Call(WExprCall),
    Field(WExprField),
    Struct(WExprStruct),
    Reference(WExprReference),
    Lit(Lit),
}

#[derive(Clone, Debug, Hash)]
pub struct WExprCall {
    pub fn_path: WPath,
    pub args: Vec<WCallArg>,
}

impl WExprCall {
    pub fn span(&self) -> Span {
        // TODO: correct span
        self.fn_path.span()
    }
}

#[derive(Clone, Debug, Hash)]
pub enum WCallArg {
    Ident(WIdent),
    Literal(Lit),
}

#[derive(Clone, Debug, Hash)]
pub struct WExprField {
    pub base: WIdent,
    pub inner: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WExprStruct {
    pub type_path: WPath,
    pub fields: Vec<(WIdent, WIdent)>,
}

#[derive(Clone, Debug, Hash)]
pub enum WExprReference {
    Ident(WIdent),
    Field(WExprField),
}

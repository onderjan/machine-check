use proc_macro2::Span;
use std::hash::Hash;
use syn::Lit;
mod from_syn;
mod to_syn;

#[derive(Clone, Debug, Hash)]
pub struct WDescription {
    pub items: Vec<WItem>,
}

#[derive(Clone, Debug, Hash)]
pub enum WItem {
    Struct(WItemStruct),
    Impl(WItemImpl),
}

#[derive(Clone, Debug, Hash)]
pub struct WItemStruct {
    pub visibility: WVisibility,
    pub derives: Vec<WPath>,
    pub ident: WIdent,
    pub fields: Vec<WField>,
}

#[derive(Clone, Debug, Hash)]
pub enum WVisibility {
    Public,
    Inherited,
}

#[derive(Clone, Debug, Hash)]
pub struct WField {
    pub ident: WIdent,
    pub ty: WSimpleType,
}

#[derive(Clone, Debug, Hash)]
pub struct WItemImpl {
    pub self_ty: WPath,
    pub trait_: Option<WPath>,
    pub items: Vec<WImplItem>,
}

#[derive(Clone, Debug, Hash)]
pub enum WImplItem {
    Fn(WImplItemFn),
    Type(WImplItemType),
}

#[derive(Clone, Debug, Hash)]
pub struct WImplItemFn {
    pub signature: WSignature,
    pub block: WBlock,
    // TODO: only allow idents in fn result
    pub result: Option<WExpr>,
}

#[derive(Clone, Debug, Hash)]
pub struct WSignature {
    pub ident: WIdent,
    pub inputs: Vec<WFnArg>,
    pub output: WSimpleType,
}

#[derive(Clone, Debug, Hash)]
pub struct WFnArg {
    pub ident: WIdent,
    pub ty: WType,
}

#[derive(Clone, Debug, Hash)]
pub struct WImplItemType {
    pub left_ident: WIdent,
    pub right_path: WPath,
}

#[derive(Clone, Debug, Hash)]
pub struct WBlock {
    pub locals: Vec<WLocal>,
    pub stmts: Vec<WStmt>,
}

#[derive(Clone, Debug, Hash)]
pub enum WStmt {
    Assign(WStmtAssign),
    If(WStmtIf),
}

#[derive(Clone, Debug, Hash)]
pub struct WStmtAssign {
    pub left_ident: WIdent,
    pub right_expr: WExpr,
}

#[derive(Clone, Debug, Hash)]
pub struct WStmtIf {
    pub condition: WExpr,
    pub then_block: WBlock,
    pub else_block: WBlock,
}

#[derive(Clone, Debug, Hash)]
pub struct WLocal {
    pub ident: WIdent,
    pub original: WIdent,
    pub ty: Option<WType>,
}

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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WSimpleType {
    Bitvector(u32),
    BitvectorArray(WTypeArray),
    Unsigned(u32),
    Signed(u32),
    Boolean,
    Path(WPath),
}

impl WSimpleType {
    pub fn into_type(self) -> WType {
        WType {
            reference: WReference::None,
            inner: self,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WType {
    pub reference: WReference,
    pub inner: WSimpleType,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WReference {
    Mutable,
    Immutable,
    None,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WTypeArray {
    pub index_width: u32,
    pub element_width: u32,
}

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

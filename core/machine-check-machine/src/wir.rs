use proc_macro2::Span;
use syn::Lit;
mod from_syn;
mod to_syn;

#[derive(Clone, Debug)]
pub struct WDescription {
    items: Vec<WItem>,
}

#[derive(Clone, Debug)]
pub enum WItem {
    Struct(WItemStruct),
    Impl(WItemImpl),
}

#[derive(Clone, Debug)]
pub struct WItemStruct {
    visibility: WVisibility,
    derives: Vec<WPath>,
    ident: WIdent,
    fields: Vec<WField>,
}

#[derive(Clone, Debug)]
pub enum WVisibility {
    Public,
    Inherited,
}

#[derive(Clone, Debug)]
pub struct WField {
    ident: WIdent,
    ty: WSimpleType,
}

#[derive(Clone, Debug)]
pub struct WItemImpl {
    self_ty: WPath,
    trait_: Option<WPath>,
    items: Vec<WImplItem>,
}

#[derive(Clone, Debug)]
pub enum WImplItem {
    Fn(WImplItemFn),
    Type(WImplItemType),
}

#[derive(Clone, Debug)]
pub struct WImplItemFn {
    signature: WSignature,
    block: WBlock,
    // TODO: only allow idents in fn result
    result: Option<WExpr>,
}

#[derive(Clone, Debug)]
pub struct WSignature {
    ident: WIdent,
    inputs: Vec<WFnArg>,
    output: WSimpleType,
}

#[derive(Clone, Debug)]
pub struct WFnArg {
    ident: WIdent,
    ty: WType,
}

#[derive(Clone, Debug)]
pub struct WImplItemType {
    left_ident: WIdent,
    right_path: WPath,
}

#[derive(Clone, Debug)]
pub struct WBlock {
    locals: Vec<WLocal>,
    stmts: Vec<WStmt>,
}

#[derive(Clone, Debug)]
pub enum WStmt {
    Assign(WStmtAssign),
    If(WStmtIf),
}

#[derive(Clone, Debug)]
pub struct WStmtAssign {
    left_ident: WIdent,
    right_expr: WExpr,
}

#[derive(Clone, Debug)]
pub struct WStmtIf {
    condition: WExpr,
    then_block: WBlock,
    else_block: WBlock,
}

#[derive(Clone, Debug)]
pub struct WLocal {
    ident: WIdent,
    original: WIdent,
    ty: Option<WType>,
}

#[derive(Clone, Debug)]
pub enum WExpr {
    Move(WIdent),
    Call(WExprCall),
    Field(WExprField),
    Struct(WExprStruct),
    Reference(WExprReference),
    Lit(Lit),
}

#[derive(Clone, Debug)]
pub struct WExprCall {
    fn_path: WPath,
    args: Vec<WCallArg>,
}

#[derive(Clone, Debug)]
pub enum WCallArg {
    Ident(WIdent),
    Literal(Lit),
}

#[derive(Clone, Debug)]
pub struct WExprField {
    base: WIdent,
    inner: WIdent,
}

#[derive(Clone, Debug)]
pub struct WExprStruct {
    type_path: WPath,
    fields: Vec<(WIdent, WIdent)>,
}

#[derive(Clone, Debug)]
pub enum WExprReference {
    Ident(WIdent),
    Field(WExprField),
}

#[derive(Clone, Debug)]
pub enum WSimpleType {
    Bitvector(u32),
    BitvectorArray(WTypeArray),
    Unsigned(u32),
    Signed(u32),
    Path(WPath),
}

#[derive(Clone, Debug)]
pub struct WType {
    reference: WReference,
    inner: WSimpleType,
}

#[derive(Clone, Debug)]
pub enum WReference {
    Mutable,
    Immutable,
    None,
}

#[derive(Clone, Debug)]
pub struct WTypeArray {
    index_width: u32,
    element_width: u32,
}

#[derive(Clone, Debug)]
pub struct WPath {
    leading_colon: bool,
    segments: Vec<WPathSegment>,
}
#[derive(Clone, Debug)]
pub struct WPathSegment {
    ident: WIdent,
    generics: Option<WGenerics>,
}

#[derive(Clone, Debug)]
pub struct WGenerics {
    leading_colon: bool,
    inner: Vec<WGeneric>,
}

#[derive(Clone, Debug)]
pub enum WGeneric {
    Type(WSimpleType),
    Const(u32),
}

#[derive(Clone, Debug)]
pub struct WIdent {
    name: String,
    span: Span,
}

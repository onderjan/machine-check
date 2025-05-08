use super::{WBlock, WExpr, WIdent, WPath, WSimpleType, WType};

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

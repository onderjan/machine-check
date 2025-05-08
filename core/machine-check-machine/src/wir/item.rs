use super::{WIdent, WImplItem, WPath, WSimpleType};

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

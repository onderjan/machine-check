use std::{fmt::Debug, hash::Hash};
use syn::{Expr, File, Item, ItemImpl, Local, Stmt, Type};

mod expr;
mod impl_item;
mod item;
mod path;
mod stmt;
mod ty;

pub use expr::*;
pub use impl_item::*;
pub use item::*;
pub use path::*;
pub use stmt::*;
pub use ty::*;

#[derive(Clone, Debug, Hash)]
pub struct WDescription<Y: YStage> {
    pub structs: Vec<WItemStruct<<Y::AssignTypes as ZAssignTypes>::FundamentalType>>,
    pub impls: Vec<WItemImpl<Y>>,
}

pub trait IntoSyn<T> {
    fn into_syn(self) -> T;
}

impl<Y: YStage> IntoSyn<File> for WDescription<Y>
where
    WItemImpl<Y>: IntoSyn<ItemImpl>,
{
    fn into_syn(self) -> File {
        File {
            shebang: None,
            attrs: Vec::new(),
            items: self
                .structs
                .into_iter()
                .map(|item| Item::Struct(item.into_syn()))
                .chain(
                    self.impls
                        .into_iter()
                        .map(|item| Item::Impl(item.into_syn())),
                )
                .collect(),
        }
    }
}

pub trait ZAssignTypes {
    type Stmt: IntoSyn<Stmt> + Clone + Debug + Hash;
    type FundamentalType: IntoSyn<Type> + Clone + Debug + Hash;
    type AssignLeft: IntoSyn<Expr> + Clone + Debug + Hash;
    type AssignRight: IntoSyn<Expr> + Clone + Debug + Hash;
}

#[derive(Clone, Debug, Hash)]
pub struct ZTac;

impl ZAssignTypes for ZTac {
    type Stmt = WMacroableStmt<ZTac>;
    type FundamentalType = WBasicType;
    type AssignLeft = WIndexedIdent;
    type AssignRight = WIndexedExpr<WBasicType, WHighLevelCallFunc<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct ZNonindexed;

impl ZAssignTypes for ZNonindexed {
    type Stmt = WMacroableStmt<ZNonindexed>;
    type FundamentalType = WBasicType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WBasicType, WHighLevelCallFunc<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct ZTotal;

impl ZAssignTypes for ZTotal {
    type Stmt = WStmt<ZTotal>;
    type FundamentalType = WBasicType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WBasicType, WHighLevelCallFunc<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct ZSsa;

impl ZAssignTypes for ZSsa {
    type Stmt = WStmt<ZSsa>;
    type FundamentalType = WBasicType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WBasicType, WHighLevelCallFunc<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct ZConverted;

impl ZAssignTypes for ZConverted {
    type Stmt = WStmt<ZConverted>;
    type FundamentalType = WElementaryType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WElementaryType, WCallFunc<WElementaryType>>;
}

pub trait YStage {
    type AssignTypes: ZAssignTypes + Clone + Debug + Hash;
    type OutputType: IntoSyn<Type> + Clone + Debug + Hash;
    type FnResult: IntoSyn<Expr> + Clone + Debug + Hash;
    type Local: IntoSyn<Local> + Clone + Debug + Hash;
}

#[derive(Clone, Debug, Hash)]
pub struct YTac;

impl YStage for YTac {
    type AssignTypes = ZTac;
    type OutputType = WBasicType;
    type FnResult = WIdent;
    type Local = WTacLocal<WPartialGeneralType<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct YNonindexed;

impl YStage for YNonindexed {
    type AssignTypes = ZNonindexed;
    type OutputType = WBasicType;
    type FnResult = WIdent;
    type Local = WTacLocal<WPartialGeneralType<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct YTotal;

impl YStage for YTotal {
    type AssignTypes = ZTotal;
    type OutputType = WPanicResultType<WBasicType>;
    type FnResult = WPanicResult;
    type Local = WTacLocal<WPartialGeneralType<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct YSsa;

impl YStage for YSsa {
    type AssignTypes = ZSsa;
    type OutputType = WPanicResultType<WBasicType>;
    type FnResult = WPanicResult;
    type Local = WSsaLocal<WPartialGeneralType<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct YInferred;

impl YStage for YInferred {
    type AssignTypes = ZSsa;
    type OutputType = WPanicResultType<WBasicType>;
    type FnResult = WPanicResult;
    type Local = WSsaLocal<WGeneralType<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct YConverted;

impl YStage for YConverted {
    type AssignTypes = ZConverted;
    type OutputType = WPanicResultType<WElementaryType>;
    type FnResult = WPanicResult;
    type Local = WSsaLocal<WGeneralType<WElementaryType>>;
}

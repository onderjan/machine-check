use std::{fmt::Debug, hash::Hash};
use syn::{Expr, File, Item, ItemImpl, Local, Type};

mod expr;
mod impl_item;
mod item;
mod path;
mod stmt;
mod ty;

mod from_syn;

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
    type FundamentalType: IntoSyn<Type> + Clone + Debug + Hash;
    type AssignLeft: IntoSyn<Expr> + Clone + Debug + Hash;
    type AssignRight: IntoSyn<Expr> + Clone + Debug + Hash;
}

#[derive(Clone, Debug, Hash)]
pub struct ZTac;

impl ZAssignTypes for ZTac {
    type FundamentalType = WBasicType;
    type AssignLeft = WIndexedIdent;
    type AssignRight = WIndexedExpr<WBasicType>;
}

#[derive(Clone, Debug, Hash)]
pub struct ZSsa;

impl ZAssignTypes for ZSsa {
    type FundamentalType = WBasicType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WBasicType>;
}

#[derive(Clone, Debug, Hash)]
pub struct ZConverted;

impl ZAssignTypes for ZConverted {
    type FundamentalType = WElementaryType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WElementaryType>;
}

pub trait YStage {
    type AssignTypes: ZAssignTypes + Clone + Debug + Hash;
    type OutputType: IntoSyn<Type> + Clone + Debug + Hash;
    type OutputExpr: IntoSyn<Expr> + Clone + Debug + Hash;
    type Local: IntoSyn<Local> + Clone + Debug + Hash;
}

#[derive(Clone, Debug, Hash)]
pub struct YTac;

impl YStage for YTac {
    type AssignTypes = ZTac;
    type OutputType = WBasicType;
    type OutputExpr = WExpr<WBasicType>;
    type Local = WTacLocal<WPartialGeneralType<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct YNonindexed;

impl YStage for YNonindexed {
    type AssignTypes = ZSsa;
    type OutputType = WBasicType;
    type OutputExpr = WExpr<WBasicType>;
    type Local = WTacLocal<WPartialGeneralType<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct YTotal;

impl YStage for YTotal {
    type AssignTypes = ZSsa;
    type OutputType = WPanicResultType<WBasicType>;
    type OutputExpr = WPanicResultExpr<WBasicType>;
    type Local = WTacLocal<WPartialGeneralType<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct YSsa;

impl YStage for YSsa {
    type AssignTypes = ZSsa;
    type OutputType = WPanicResultType<WBasicType>;
    type OutputExpr = WPanicResultExpr<WBasicType>;
    type Local = WSsaLocal<WPartialGeneralType<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct YInferred;

impl YStage for YInferred {
    type AssignTypes = ZSsa;
    type OutputType = WPanicResultType<WBasicType>;
    type OutputExpr = WPanicResultExpr<WBasicType>;
    type Local = WSsaLocal<WGeneralType<WBasicType>>;
}

#[derive(Clone, Debug, Hash)]
pub struct YConverted;

impl YStage for YConverted {
    type AssignTypes = ZConverted;
    type OutputType = WPanicResultType<WElementaryType>;
    type OutputExpr = WPanicResultExpr<WElementaryType>;
    type Local = WSsaLocal<WGeneralType<WElementaryType>>;
}

use proc_macro2::Span;
use std::{fmt::Debug, hash::Hash};
use syn::{
    token::Brace, Expr, File, Generics, Ident, ImplItem, Item, ItemImpl, Local, Path, Stmt, Token,
    Type,
};

mod call;
mod expr;
mod impl_item;
mod item;
mod op;
mod path;
mod span;
mod stmt;
mod ty;

pub use call::*;
pub use expr::*;
pub use impl_item::*;
pub use item::*;
pub use op::*;
pub use path::*;
pub use span::*;
pub use stmt::*;
pub use ty::*;

use crate::util::{create_path_from_ident, create_type_path};

#[derive(Clone, Debug, Hash)]
pub struct WDescription<Y: YStage> {
    pub structs: Vec<WItemStruct<<Y::AssignTypes as ZAssignTypes>::FundamentalType>>,
    pub impls: Vec<WItemImpl<Y>>,
}

#[derive(Clone, Debug, Hash)]
pub struct WProperty<Y: YStage> {
    pub fns: Vec<WImplItemFn<Y>>,
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

impl<Y: YStage> IntoSyn<File> for WProperty<Y>
where
    WItemImpl<Y>: IntoSyn<ItemImpl>,
{
    fn into_syn(self) -> File {
        File {
            shebang: None,
            attrs: Vec::new(),
            items: vec![Item::Impl(ItemImpl {
                attrs: vec![],
                defaultness: None,
                unsafety: None,
                impl_token: Token![impl](Span::call_site()),
                generics: Generics::default(),
                trait_: None,
                self_ty: Box::new(create_type_path(create_path_from_ident(Ident::new(
                    "PropertyComputer",
                    Span::call_site(),
                )))),
                brace_token: Brace::default(),
                items: self
                    .fns
                    .into_iter()
                    .map(|fun| ImplItem::Fn(fun.into_syn()))
                    .collect(),
            })],
        }
    }
}

pub trait ZIfPolarity: IntoSyn<Path> + Clone + Debug + Hash {}

pub trait ZAssignTypes {
    type Stmt: IntoSyn<Stmt> + Clone + Debug + Hash;
    type FundamentalType: IntoSyn<Type> + Clone + Debug + Hash;
    type AssignLeft: IntoSyn<Expr> + Clone + Debug + Hash;
    type AssignRight: IntoSyn<Expr> + Clone + Debug + Hash;
    type IfPolarity: ZIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct WNoIfPolarity;

impl IntoSyn<Path> for WNoIfPolarity {
    fn into_syn(self) -> Path {
        syn_path::path!(::mck::forward::Test::into_bool)
    }
}

impl ZIfPolarity for WNoIfPolarity {}

#[derive(Clone, Debug, Hash)]
pub struct ZTac;

impl ZAssignTypes for ZTac {
    type Stmt = WMacroableStmt<ZTac>;
    type FundamentalType = WBasicType;
    type AssignLeft = WIndexedIdent;
    type AssignRight = WIndexedExpr<WExprHighCall>;
    type IfPolarity = WNoIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct ZNonindexed;

impl ZAssignTypes for ZNonindexed {
    type Stmt = WMacroableStmt<ZNonindexed>;
    type FundamentalType = WBasicType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprHighCall>;
    type IfPolarity = WNoIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct ZTotal;

impl ZAssignTypes for ZTotal {
    type Stmt = WStmt<ZTotal>;
    type FundamentalType = WBasicType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprHighCall>;
    type IfPolarity = WNoIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct ZSsa;

impl ZAssignTypes for ZSsa {
    type Stmt = WStmt<ZSsa>;
    type FundamentalType = WBasicType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprHighCall>;
    type IfPolarity = WNoIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct ZConverted;

impl ZAssignTypes for ZConverted {
    type Stmt = WStmt<ZConverted>;
    type FundamentalType = WElementaryType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprCall>;
    type IfPolarity = WNoIfPolarity;
}

pub trait YStage {
    type AssignTypes: ZAssignTypes + Clone + Debug + Hash;
    type InputType: IntoSyn<Type> + Clone + Debug + Hash;
    type OutputType: IntoSyn<Type> + Clone + Debug + Hash;
    type FnResult: IntoSyn<Expr> + Clone + Debug + Hash;
    type Local: IntoSyn<Local> + Clone + Debug + Hash;
    type ItemImplTrait: IntoSyn<Path> + Clone + Debug + Hash;
}

#[derive(Clone, Debug, Hash)]
pub struct YTac;

impl YStage for YTac {
    type AssignTypes = ZTac;
    type InputType = WType<WBasicType>;
    type OutputType = WBasicType;
    type FnResult = WIdent;
    type Local = WTacLocal<WPartialGeneralType<WBasicType>>;
    type ItemImplTrait = WItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct YNonindexed;

impl YStage for YNonindexed {
    type AssignTypes = ZNonindexed;
    type InputType = WType<WBasicType>;
    type OutputType = WBasicType;
    type FnResult = WIdent;
    type Local = WTacLocal<WPartialGeneralType<WBasicType>>;
    type ItemImplTrait = WItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct YTotal;

impl YStage for YTotal {
    type AssignTypes = ZTotal;
    type InputType = WType<WBasicType>;
    type OutputType = WPanicResultType<WBasicType>;
    type FnResult = WPanicResult;
    type Local = WTacLocal<WPartialGeneralType<WBasicType>>;
    type ItemImplTrait = WItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct YSsa;

impl YStage for YSsa {
    type AssignTypes = ZSsa;
    type InputType = WType<WBasicType>;
    type OutputType = WPanicResultType<WBasicType>;
    type FnResult = WPanicResult;
    type Local = WSsaLocal<WPartialGeneralType<WBasicType>>;
    type ItemImplTrait = WItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct YInferred;

impl YStage for YInferred {
    type AssignTypes = ZSsa;
    type InputType = WType<WBasicType>;
    type OutputType = WPanicResultType<WBasicType>;
    type FnResult = WPanicResult;
    type Local = WSsaLocal<WGeneralType<WBasicType>>;
    type ItemImplTrait = WItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct YConverted;

impl YStage for YConverted {
    type AssignTypes = ZConverted;
    type InputType = WType<WElementaryType>;
    type OutputType = WPanicResultType<WElementaryType>;
    type FnResult = WPanicResult;
    type Local = WSsaLocal<WGeneralType<WElementaryType>>;
    type ItemImplTrait = WItemImplTrait;
}

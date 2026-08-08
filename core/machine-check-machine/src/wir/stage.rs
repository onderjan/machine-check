use std::{fmt::Debug, hash::Hash};

use syn::{Expr, Local, Path, Stmt, Type};

use crate::wir::{
    IntoSyn, WExpr, WExprHighCall, WExprLowCall, WIdent, WIndexedExpr, WIndexedIdent,
    WItemImplTrait, WMacroableStmt, WSsaLocal, WStmt, WTacLocal, WTypeId,
};

pub trait YStage {
    type AssignTypes: ZAssignTypes + Clone + Debug + Hash;
    type FnResult: IntoSyn<Expr> + Clone + Debug + Hash;
    type Local: IntoSyn<Local> + Clone + Debug + Hash;
    type ItemImplTrait: IntoSyn<Path> + Clone + Debug + Hash;
}

#[derive(Clone, Debug, Hash)]
pub struct YTac;

impl YStage for YTac {
    type AssignTypes = ZTac;
    type FnResult = WIdent;
    type Local = WTacLocal;
    type ItemImplTrait = WItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct YTotal;

impl YStage for YTotal {
    type AssignTypes = ZTotal;
    type FnResult = WIdent;
    type Local = WTacLocal;
    type ItemImplTrait = WItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct YLowered;

impl YStage for YLowered {
    type AssignTypes = ZLowered;
    type FnResult = WIdent;
    type Local = WTacLocal;
    type ItemImplTrait = WItemImplTrait;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct YSsa;

impl YStage for YSsa {
    type AssignTypes = ZSsa;
    type FnResult = WIdent;
    type Local = WSsaLocal;
    type ItemImplTrait = WItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct ZTac;

impl ZAssignTypes for ZTac {
    type Stmt = WMacroableStmt<ZTac>;
    type AssignLeft = WIndexedIdent;
    type AssignRight = WIndexedExpr<WExprHighCall>;
    type IfPolarity = WNoIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct ZTotal;

impl ZAssignTypes for ZTotal {
    type Stmt = WStmt<ZTotal>;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprHighCall>;
    type IfPolarity = WNoIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct ZLowered;

impl ZAssignTypes for ZLowered {
    type Stmt = WStmt<ZLowered>;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprLowCall>;
    type IfPolarity = WNoIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct ZSsa;

impl ZAssignTypes for ZSsa {
    type Stmt = WStmt<ZSsa>;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprLowCall>;
    type IfPolarity = WNoIfPolarity;
}

pub trait ZIfPolarity: IntoSyn<Path> + Clone + Debug + Hash {}

pub trait ZAssignTypes {
    type Stmt: IntoSyn<Stmt> + Clone + Debug + Hash;
    type AssignLeft: IntoSyn<Expr> + Clone + Debug + Hash;
    type AssignRight: IntoSyn<Expr> + Clone + Debug + Hash;
    type IfPolarity: ZIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct WNoIfPolarity;

impl IntoSyn<Path> for WNoIfPolarity {
    fn into_syn(self, _type_fn: &impl Fn(WTypeId) -> Type) -> Path {
        syn_path::path!(::mck::forward::Test::into_bool)
    }
}

impl ZIfPolarity for WNoIfPolarity {}

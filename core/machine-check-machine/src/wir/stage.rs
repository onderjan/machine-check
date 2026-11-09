use std::{fmt::Debug, hash::Hash};

use syn::{Block, Expr, Local, Path, Stmt, Type};

use crate::wir::{
    IntoTypedSyn, WExpr, WExprHighCall, WExprLowCall, WIdent, WIndexedExpr, WIndexedIdent,
    WItemFnBody, WItemImplTrait, WMacroableStmt, WSsaLocal, WStmt, WSynBlock, WTacLocal, WTypeId,
};

pub trait YStage: Clone + PartialEq + Eq {
    type Local: IntoTypedSyn<Local> + Clone + Debug + Hash;
    type ItemImplTrait: IntoTypedSyn<Path> + Clone + Debug + Hash;

    type FnBody: IntoTypedSyn<Block> + Clone + Debug + Hash;
    type Stmt: IntoTypedSyn<Stmt> + Clone + Debug + Hash + PartialEq + Eq;
    type AssignLeft: IntoTypedSyn<Expr> + Clone + Debug + Hash + PartialEq + Eq;
    type AssignRight: IntoTypedSyn<Expr> + Clone + Debug + Hash + PartialEq + Eq;
    type IfPolarity: YIfPolarity + PartialEq + Eq;
}

pub trait YIfPolarity: IntoTypedSyn<Path> + Clone + Debug + Hash {}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct YBuild;

impl YStage for YBuild {
    type Local = WTacLocal;
    type ItemImplTrait = WItemImplTrait;

    type FnBody = WSynBlock;
    type Stmt = WMacroableStmt<YTac>;
    type AssignLeft = WIndexedIdent;
    type AssignRight = WIndexedExpr<WExprHighCall>;
    type IfPolarity = WNoIfPolarity;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct YTac;

impl YStage for YTac {
    type Local = WTacLocal;
    type ItemImplTrait = WItemImplTrait;

    type FnBody = WItemFnBody<YTac>;
    type Stmt = WMacroableStmt<YTac>;
    type AssignLeft = WIndexedIdent;
    type AssignRight = WIndexedExpr<WExprHighCall>;
    type IfPolarity = WNoIfPolarity;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct YLowered;

impl YStage for YLowered {
    type Local = WTacLocal;
    type ItemImplTrait = WItemImplTrait;

    type FnBody = WItemFnBody<YLowered>;
    type Stmt = WStmt<YLowered>;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprLowCall>;
    type IfPolarity = WNoIfPolarity;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct YSsa;

impl YStage for YSsa {
    type Local = WSsaLocal;
    type ItemImplTrait = WItemImplTrait;

    type FnBody = WItemFnBody<YSsa>;
    type Stmt = WStmt<YSsa>;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprLowCall>;
    type IfPolarity = WNoIfPolarity;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WNoIfPolarity;

impl IntoTypedSyn<Path> for WNoIfPolarity {
    fn into_typed_syn(self, _type_fn: &impl Fn(WTypeId) -> Type) -> Path {
        syn_path::path!(::mck::forward::Test::into_bool)
    }
}

impl YIfPolarity for WNoIfPolarity {}

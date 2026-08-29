use proc_macro2::Span;
use std::fmt::Debug;
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Paren, Block, FnArg, Generics, ImplItemFn,
    ImplItemType, ItemFn, Local, Pat, PatIdent, PatType, Receiver, Signature, Stmt, Token, Type,
    TypePath, TypeReference,
};
use syn_path::path;

use crate::wir::{WBlock, WItemFn, WPartialPath, WTypeId, WVisibility};

use super::{IntoTypedSyn, WIdent, YStage};

#[derive(Clone, Hash)]
pub struct WImplItemType {
    pub visibility: WVisibility,
    pub left_ident: WIdent,
    pub right_path: WPartialPath,
}

impl Debug for WImplItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let WVisibility::Public(_) = self.visibility {
            write!(f, "pub ")?;
        }
        write!(f, "type ")?;
        Debug::fmt(&self.left_ident, f)?;
        write!(f, " = ")?;
        Debug::fmt(&self.right_path, f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WFnSignature {
    pub ident: WIdent,
    pub inputs: Vec<WFnArg>,
    pub output: WTypeId,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WFnArg {
    pub ident: WIdent,
    pub ty: WTypeId,
}

impl Debug for WFnSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}(", self.ident)?;
        let mut first = true;
        for input in &self.inputs {
            if first {
                first = false
            } else {
                write!(f, ", ")?;
            }
            Debug::fmt(&input, f)?;
        }
        write!(f, ") -> {:?}", self.output)
    }
}

impl Debug for WFnArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.ident, self.ty)
    }
}

#[derive(Clone, Hash)]
pub struct WTacLocal {
    pub ident: WIdent,
    pub ty: WTypeId,
}

impl Debug for WTacLocal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.ident, f)?;
        write!(f, ": ")?;
        Debug::fmt(&self.ty, f)
    }
}

#[derive(Clone, Debug, Hash)]
pub struct WSsaLocal {
    pub ident: WIdent,
    pub original: WIdent,
    pub ty: WTypeId,
}

impl IntoTypedSyn<ImplItemType> for WImplItemType {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> ImplItemType {
        let span = Span::call_site();

        ImplItemType {
            attrs: Vec::new(),
            vis: self.visibility.into_typed_syn(type_fn),
            defaultness: None,
            type_token: Token![type](span),
            ident: self.left_ident.into(),
            generics: Generics::default(),
            eq_token: Token![=](span),
            ty: Type::Path(TypePath {
                qself: None,
                path: self.right_path.into(),
            }),
            semi_token: Token![;](span),
        }
    }
}

impl<Y: YStage> IntoTypedSyn<ImplItemFn> for WItemFn<Y> {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> ImplItemFn {
        let item_fn: ItemFn = self.into_typed_syn(type_fn);

        ImplItemFn {
            attrs: item_fn.attrs,
            vis: item_fn.vis,
            defaultness: None,
            sig: item_fn.sig,
            block: *item_fn.block,
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub struct WItemFnBody<Y: YStage> {
    pub locals: Vec<Y::Local>,
    pub block: WBlock<Y>,
    pub result: WIdent,
}

impl<Y: YStage> IntoTypedSyn<Block> for WItemFnBody<Y> {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Block {
        let mut block = self.block.into_typed_syn(type_fn);

        let standard_stmts: Vec<Stmt> = block.stmts.drain(..).collect();

        for local in self.locals {
            block.stmts.push(Stmt::Local(local.into_typed_syn(type_fn)));
        }

        block.stmts.extend(standard_stmts);
        block
            .stmts
            .push(Stmt::Expr(self.result.into_typed_syn(type_fn), None));

        block
    }
}

impl<Y: YStage> IntoTypedSyn<ItemFn> for WItemFn<Y> {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> ItemFn {
        let span = Span::call_site();

        let body = self.body.into_typed_syn(type_fn);

        ItemFn {
            attrs: Vec::new(),
            vis: self.visibility.into_typed_syn(type_fn),
            sig: Signature {
                constness: None,
                asyncness: None,
                unsafety: None,
                abi: None,
                fn_token: Token![fn](span),
                ident: self.signature.ident.into(),
                generics: Generics::default(),
                paren_token: Paren::default(),
                inputs: Punctuated::from_iter(self.signature.inputs.into_iter().map(|fn_arg| {
                    if fn_arg.ident.name() == "self" {
                        // instead of the actual type, which may be converted to a non-Self path,
                        // create a dummy type for the receiver
                        let fn_arg_ty = fn_arg.ty.into_typed_syn(type_fn);
                        let ty_span = fn_arg_ty.span();
                        let self_ty = Type::Path(TypePath {
                            qself: None,
                            path: path!(Self),
                        });
                        let (ty, reference) = if let Type::Reference(_) = fn_arg_ty {
                            (
                                Type::Reference(TypeReference {
                                    and_token: Token![&](ty_span),
                                    lifetime: None,
                                    mutability: None,
                                    elem: Box::new(self_ty),
                                }),
                                Some((Token![&](span), None)),
                            )
                        } else {
                            (self_ty, None)
                        };

                        FnArg::Receiver(Receiver {
                            attrs: Vec::new(),
                            reference,
                            mutability: None,
                            self_token: Token![self](span),
                            colon_token: None,
                            ty: Box::new(ty),
                        })
                    } else {
                        FnArg::Typed(syn::PatType {
                            attrs: Vec::new(),
                            pat: Box::new(Pat::Ident(syn::PatIdent {
                                attrs: Vec::new(),
                                by_ref: None,
                                mutability: None,
                                ident: fn_arg.ident.into(),
                                subpat: None,
                            })),
                            colon_token: Token![:](span),
                            ty: Box::new(fn_arg.ty.into_typed_syn(type_fn)),
                        })
                    }
                })),
                variadic: None,
                output: syn::ReturnType::Type(
                    Token![->](span),
                    Box::new(self.signature.output.into_typed_syn(type_fn)),
                ),
            },
            block: Box::new(body),
        }
    }
}

impl IntoTypedSyn<Local> for WTacLocal {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Local {
        ident_type_local(self.ident, Some(self.ty), false, type_fn)
    }
}

impl IntoTypedSyn<Local> for WSsaLocal {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Local {
        ident_type_local(self.ident, Some(self.ty), false, type_fn)
    }
}

pub fn ident_type_local(
    ident: WIdent,
    ty: Option<WTypeId>,
    mutable: bool,
    type_fn: &impl Fn(WTypeId) -> Type,
) -> Local {
    let span = ident.span();

    let mut pat = Pat::Ident(PatIdent {
        attrs: Vec::new(),
        by_ref: None,
        mutability: if mutable {
            Some(Token![mut](span))
        } else {
            None
        },
        ident: ident.into(),
        subpat: None,
    });

    if let Some(ty) = ty {
        pat = Pat::Type(PatType {
            attrs: Vec::new(),
            pat: Box::new(pat),
            colon_token: Token![:](span),
            ty: Box::new(ty.into_typed_syn(type_fn)),
        });
    }

    Local {
        attrs: Vec::new(),
        let_token: Token![let](span),
        pat,
        init: None,
        semi_token: Token![;](span),
    }
}

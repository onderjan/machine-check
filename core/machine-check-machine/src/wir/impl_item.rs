use proc_macro2::Span;
use syn::{
    punctuated::Punctuated,
    token::{Bracket, Paren},
    Attribute, FnArg, Generics, ImplItemFn, ImplItemType, Local, MetaNameValue, Pat, PatIdent,
    PatType, Receiver, Signature, Stmt, Token, Type, TypePath,
};
use syn_path::path;

use crate::util::{create_expr_path, create_path_from_ident};

use super::{
    IntoSyn, WBlock, WExpr, WIdent, WLocal, WPath, WReference, WSimpleType, WType, YStage,
};

#[derive(Clone, Debug, Hash)]
pub enum WImplItem<Y: YStage> {
    Fn(WImplItemFn<Y>),
    Type(WImplItemType),
}

#[derive(Clone, Debug, Hash)]
pub struct WImplItemFn<Y: YStage> {
    pub signature: WSignature,
    pub locals: Vec<WLocal<Y>>,
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

impl IntoSyn<ImplItemType> for WImplItemType {
    fn into_syn(self) -> ImplItemType {
        let span = Span::call_site();

        ImplItemType {
            attrs: Vec::new(), // TODO visibility
            vis: syn::Visibility::Inherited,
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

impl<Y: YStage> IntoSyn<ImplItemFn> for WImplItemFn<Y>
where
    Y::LocalType: IntoSyn<Type>,
{
    fn into_syn(self) -> ImplItemFn {
        let span = Span::call_site();

        let mut block = self.block.into_syn();

        let standard_stmts: Vec<Stmt> = block.stmts.drain(..).collect();

        for local in self.locals {
            let span = local.ident.span;

            let mut pat = Pat::Ident(PatIdent {
                attrs: Vec::new(),
                by_ref: None,
                mutability: None,
                ident: local.ident.into(),
                subpat: None,
            });

            pat = Pat::Type(PatType {
                attrs: Vec::new(),
                pat: Box::new(pat),
                colon_token: Token![:](span),
                ty: Box::new(local.ty.into_syn()),
            });

            block.stmts.push(syn::Stmt::Local(Local {
                attrs: vec![Attribute {
                    pound_token: Token![#](span),
                    style: syn::AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: syn::Meta::NameValue(MetaNameValue {
                        path: path!(::mck::attr::tmp_original),
                        eq_token: Token![=](span),
                        value: create_expr_path(create_path_from_ident(local.original.into())),
                    }),
                }],
                let_token: Token![let](span),
                pat,
                init: None,
                semi_token: Token![;](span),
            }));
        }

        block.stmts.extend(standard_stmts);

        if let Some(result) = self.result {
            block.stmts.push(Stmt::Expr(result.into_syn(), None));
        }

        ImplItemFn {
            // TODO: attrs
            attrs: Vec::new(),
            // TODO: visibility
            vis: syn::Visibility::Inherited,
            defaultness: None,
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
                    if &fn_arg.ident.name == "self" {
                        // TODO: prefer typed self once it is well-supported as it is more regular
                        FnArg::Receiver(Receiver {
                            attrs: Vec::new(),
                            reference: match fn_arg.ty.reference {
                                WReference::Mutable | WReference::Immutable => {
                                    Some((Token![&](span), None))
                                }
                                WReference::None => None,
                            },

                            mutability: match fn_arg.ty.reference {
                                WReference::Mutable => Some(Token![mut](span)),
                                WReference::Immutable | WReference::None => None,
                            },
                            self_token: Token![self](span),
                            colon_token: None,
                            ty: Box::new(fn_arg.ty.into_syn()),
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
                            ty: Box::new(fn_arg.ty.into_syn()),
                        })
                    }
                })),
                variadic: None,
                output: syn::ReturnType::Type(
                    Token![->](span),
                    Box::new(self.signature.output.into_syn()),
                ),
            },
            block,
        }
    }
}

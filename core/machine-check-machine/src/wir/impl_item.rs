use proc_macro2::Span;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Bracket, Paren},
    Attribute, FnArg, Generics, ImplItemFn, ImplItemType, Local, MetaNameValue, Pat, PatIdent,
    PatType, Receiver, Signature, Stmt, Token, Type, TypePath,
};
use syn_path::path;

use crate::util::{create_expr_path, create_path_from_ident};

use super::{IntoSyn, WBlock, WIdent, WPath, WReference, WType, YStage, ZAssignTypes};

#[derive(Clone, Debug, Hash)]
pub struct WImplItemType<FT: IntoSyn<Type>> {
    pub left_ident: WIdent,
    pub right_path: WPath<FT>,
}

#[derive(Clone, Debug, Hash)]
pub struct WImplItemFn<Y: YStage> {
    pub signature: WSignature<Y>,
    pub locals: Vec<Y::Local>,
    pub block: WBlock<Y::AssignTypes>,
    // TODO: only allow idents in fn result
    pub result: Y::FnResult,
}

#[derive(Clone, Debug, Hash)]
pub struct WSignature<Y: YStage> {
    pub ident: WIdent,
    pub inputs: Vec<WFnArg<<Y::AssignTypes as ZAssignTypes>::FundamentalType>>,
    pub output: Y::OutputType,
}

#[derive(Clone, Debug, Hash)]
pub struct WFnArg<FT: IntoSyn<Type>> {
    pub ident: WIdent,
    pub ty: WType<FT>,
}

#[derive(Clone, Debug, Hash)]
pub struct WTacLocal<LT: IntoSyn<Type>> {
    pub ident: WIdent,
    pub ty: LT,
}

#[derive(Clone, Debug, Hash)]
pub struct WSsaLocal<LT: IntoSyn<Type>> {
    pub ident: WIdent,
    pub original: WIdent,
    pub ty: LT,
}

impl<FT: IntoSyn<Type>> IntoSyn<ImplItemType> for WImplItemType<FT> {
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

impl<Y: YStage> IntoSyn<ImplItemFn> for WImplItemFn<Y> {
    fn into_syn(self) -> ImplItemFn {
        let span = Span::call_site();

        let mut block = self.block.into_syn();

        let standard_stmts: Vec<Stmt> = block.stmts.drain(..).collect();

        for local in self.locals {
            block.stmts.push(Stmt::Local(local.into_syn()));
        }

        block.stmts.extend(standard_stmts);
        block.stmts.push(Stmt::Expr(self.result.into_syn(), None));

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
                    if fn_arg.ident.name() == "self" {
                        // TODO: prefer typed self once it is well-supported as it is more regular
                        FnArg::Receiver(Receiver {
                            attrs: Vec::new(),
                            reference: match fn_arg.ty.reference {
                                //WReference::Mutable |
                                WReference::Immutable => Some((Token![&](span), None)),
                                WReference::None => None,
                            },

                            mutability: match fn_arg.ty.reference {
                                //WReference::Mutable => Some(Token![mut](span)),
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

impl<LT: IntoSyn<Type>> IntoSyn<Local> for WTacLocal<LT> {
    fn into_syn(self) -> Local {
        ident_type_local(self.ident, self.ty)
    }
}

impl<LT: IntoSyn<Type>> IntoSyn<Local> for WSsaLocal<LT> {
    fn into_syn(self) -> Local {
        let mut local = ident_type_local(self.ident, self.ty);
        let span = local.span();

        local.attrs = vec![Attribute {
            pound_token: Token![#](span),
            style: syn::AttrStyle::Outer,
            bracket_token: Bracket::default(),
            meta: syn::Meta::NameValue(MetaNameValue {
                path: path!(::mck::attr::tmp_original),
                eq_token: Token![=](span),
                value: create_expr_path(create_path_from_ident(self.original.into())),
            }),
        }];

        local
    }
}

fn ident_type_local<LT: IntoSyn<Type>>(ident: WIdent, ty: LT) -> Local {
    let span = ident.span();

    let mut pat = Pat::Ident(PatIdent {
        attrs: Vec::new(),
        by_ref: None,
        mutability: None,
        ident: ident.into(),
        subpat: None,
    });

    pat = Pat::Type(PatType {
        attrs: Vec::new(),
        pat: Box::new(pat),
        colon_token: Token![:](span),
        ty: Box::new(ty.into_syn()),
    });

    Local {
        attrs: Vec::new(),
        let_token: Token![let](span),
        pat,
        init: None,
        semi_token: Token![;](span),
    }
}

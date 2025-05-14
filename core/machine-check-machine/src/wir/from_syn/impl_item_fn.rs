use syn::{Expr, ImplItemFn, Pat, Stmt};

use crate::{
    support::ident_creator::IdentCreator,
    util::{extract_expr_ident, extract_expr_path},
    wir::{
        from_syn::ty::{fold_basic_type, fold_partial_general_type, fold_type},
        WBasicType, WFnArg, WIdent, WImplItemFn, WPartialGeneralType, WPath, WPathSegment,
        WReference, WSignature, WTacLocal, WType, YTac,
    },
};

use super::path::fold_global_path;

mod expr;
mod stmt;

pub fn fold_impl_item_fn(impl_item: ImplItemFn) -> WImplItemFn<YTac> {
    FunctionFolder {
        ident_creator: IdentCreator::new(String::from("")),
    }
    .fold(impl_item)
}

struct FunctionFolder {
    ident_creator: IdentCreator,
}

impl FunctionFolder {
    pub fn fold(mut self, impl_item: ImplItemFn) -> WImplItemFn<YTac> {
        let mut inputs = Vec::new();

        for input in impl_item.sig.inputs {
            // TODO: references
            match input {
                syn::FnArg::Receiver(receiver) => {
                    let span = receiver.self_token.span;
                    let reference = match receiver.reference {
                        Some(_) => {
                            if receiver.mutability.is_some() {
                                WReference::Mutable
                            } else {
                                WReference::Immutable
                            }
                        }
                        None => WReference::None,
                    };

                    inputs.push(WFnArg {
                        ident: WIdent::new(String::from("self"), span),
                        ty: WType {
                            reference,
                            inner: WBasicType::Path(WPath {
                                leading_colon: false,
                                segments: vec![WPathSegment {
                                    ident: WIdent::new(String::from("Self"), span),
                                    generics: None,
                                }],
                            }),
                        },
                    });
                }
                syn::FnArg::Typed(pat_type) => {
                    let Pat::Ident(pat_ident) = *pat_type.pat else {
                        panic!("Unexpected non-ident pattern {:?}", pat_type);
                    };

                    inputs.push(WFnArg {
                        ident: WIdent::from_syn_ident(pat_ident.ident),
                        ty: fold_type(*pat_type.ty),
                    });
                }
            }
        }

        let output = match impl_item.sig.output {
            syn::ReturnType::Default => panic!("Unexpected default function return type"),
            syn::ReturnType::Type(_rarrow, ty) => fold_basic_type(*ty),
        };

        let signature = WSignature {
            ident: WIdent::from_syn_ident(impl_item.sig.ident),
            inputs,
            output,
        };

        let mut locals = Vec::new();

        // TODO this will not work without scope normalisation
        for stmt in &impl_item.block.stmts {
            if let Stmt::Local(local) = stmt {
                let mut pat = local.pat.clone();
                let mut ty = WPartialGeneralType::Unknown;
                if let Pat::Type(pat_type) = pat {
                    ty = fold_partial_general_type(*pat_type.ty);
                    pat = *pat_type.pat;
                }

                let Pat::Ident(left_pat_ident) = pat else {
                    panic!("Local pattern should be an ident: {:?}", pat)
                };

                locals.push(WTacLocal {
                    ident: WIdent::from_syn_ident(left_pat_ident.ident),
                    ty,
                });
            }
        }

        let (block, result) = self.fold_block(impl_item.block);

        for temporary_ident in self.ident_creator.drain_created_temporaries() {
            locals.push(WTacLocal {
                ident: temporary_ident,
                ty: WPartialGeneralType::Unknown,
            });
        }

        WImplItemFn {
            signature,
            locals,
            block,
            result,
        }
    }

    fn try_fold_expr_as_ident(&mut self, expr: Expr) -> Result<WIdent, Expr> {
        if let Some(ident) = extract_expr_ident(&expr).cloned() {
            // TODO: add scope here
            Ok(WIdent::from_syn_ident(ident))
        } else {
            Err(expr)
        }
    }

    fn fold_expr_as_ident(&mut self, expr: Expr) -> WIdent {
        let Ok(ident) = self.try_fold_expr_as_ident(expr) else {
            // TODO: this should be an error, not a panic
            panic!("Expr should be ident");
        };
        ident
    }

    fn fold_expr_as_path(&mut self, expr: Expr) -> WPath<WBasicType> {
        let Some(path) = extract_expr_path(&expr).cloned() else {
            // TODO: this should be an error, not a panic
            panic!("Expr should be path");
        };
        // TODO: add scope here
        fold_global_path(path)
    }
}

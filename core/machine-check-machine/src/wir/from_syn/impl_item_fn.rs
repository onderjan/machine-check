use syn::{ImplItemFn, Pat, Stmt};

use crate::{
    support::ident_creator::IdentCreator,
    wir::{
        from_syn::ty::{fold_basic_type, fold_partial_general_type, fold_type},
        WBasicType, WFnArg, WIdent, WImplItemFn, WPartialGeneralType, WPath, WPathSegment,
        WReference, WSignature, WTacLocal, WType, YTac,
    },
};

mod expr;
mod stmt;

pub fn fold_impl_item_fn(
    impl_item: ImplItemFn,
    ident_creator: &mut IdentCreator,
) -> WImplItemFn<YTac> {
    // TODO: there should be a separate ident creator for every function
    // there seems to be a problem in backward conversion type discovery
    // if there are multiple same-ident temporaries in different functions
    FunctionFolder { ident_creator }.fold(impl_item)
}

struct FunctionFolder<'a> {
    ident_creator: &'a mut IdentCreator,
}

impl FunctionFolder<'_> {
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
                        ident: WIdent {
                            name: String::from("self"),
                            span,
                        },
                        ty: WType {
                            reference,
                            inner: WBasicType::Path(WPath {
                                leading_colon: false,
                                segments: vec![WPathSegment {
                                    ident: WIdent {
                                        name: String::from("Self"),
                                        span,
                                    },
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
                        ident: pat_ident.ident.into(),
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
            ident: impl_item.sig.ident.into(),
            inputs,
            output,
        };

        let mut locals = Vec::new();

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
                    ident: left_pat_ident.ident.into(),
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
}

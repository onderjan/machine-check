use std::collections::{BTreeMap, HashMap};

use syn::{Expr, ImplItemFn, Pat};

use crate::{
    ssa::{
        error::{DescriptionError, DescriptionErrors},
        from_syn::ty::{fold_basic_type, fold_type},
    },
    support::ident_creator::IdentCreator,
    util::{extract_expr_ident, extract_expr_path},
    wir::{
        WBasicType, WFnArg, WIdent, WImplItemFn, WPartialGeneralType, WPath, WPathSegment,
        WReference, WSignature, WTacLocal, WType, YTac,
    },
};

use super::path::fold_global_path;

mod expr;
mod stmt;

pub fn fold_impl_item_fn(impl_item: ImplItemFn) -> Result<WImplItemFn<YTac>, DescriptionErrors> {
    FunctionFolder {
        ident_creator: IdentCreator::new(String::from("")),
        scopes: Vec::new(),
        local_types: BTreeMap::new(),
        next_scope_id: 0,
    }
    .fold(impl_item)
}

struct FunctionScope {
    local_map: HashMap<WIdent, WIdent>,
}

struct FunctionFolder {
    ident_creator: IdentCreator,
    local_types: BTreeMap<WIdent, WPartialGeneralType<WBasicType>>,
    scopes: Vec<FunctionScope>,
    next_scope_id: u32,
}

impl FunctionFolder {
    pub fn fold(mut self, impl_item: ImplItemFn) -> Result<WImplItemFn<YTac>, DescriptionErrors> {
        let mut inputs = Vec::new();

        let scope_id = 1;

        let outer_scope = FunctionScope {
            local_map: HashMap::new(),
        };
        self.scopes.push(outer_scope);

        self.next_scope_id = scope_id + 1;

        for input in impl_item.sig.inputs {
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

                    // do not scope self, it is unnecessary
                    let self_ident = WIdent::new(String::from("self"), span);

                    let self_type = WType {
                        reference,
                        inner: WBasicType::Path(WPath {
                            leading_colon: false,
                            segments: vec![WPathSegment {
                                ident: WIdent::new(String::from("Self"), span),
                                generics: None,
                            }],
                        }),
                    };

                    inputs.push(Ok(WFnArg {
                        ident: self_ident.clone(),
                        ty: self_type.clone(),
                    }));

                    self.add_unique_scoped_ident(self_ident.clone(), self_ident);
                }
                syn::FnArg::Typed(pat_type) => {
                    let Pat::Ident(pat_ident) = *pat_type.pat else {
                        // TODO: this should be an error
                        panic!("Unexpected non-ident pattern {:?}", pat_type);
                    };

                    let original_ident = WIdent::from_syn_ident(pat_ident.ident);
                    let ty = fold_type(*pat_type.ty);

                    let locally_unique_ident = self.add_scoped_ident(scope_id, original_ident);

                    let fn_arg = match ty {
                        Ok(ty) => Ok(WFnArg {
                            ident: locally_unique_ident,
                            ty,
                        }),
                        Err(err) => Err(err),
                    };

                    inputs.push(fn_arg);
                }
            }
        }

        let inputs = DescriptionErrors::flat_single_result(inputs);

        let output = match impl_item.sig.output {
            syn::ReturnType::Default => panic!("Unexpected default function return type"),
            syn::ReturnType::Type(_rarrow, ty) => fold_basic_type(*ty),
        }
        .map_err(DescriptionErrors::single);

        let (inputs, output) = DescriptionErrors::combine(inputs, output)?;

        let signature = WSignature {
            ident: WIdent::from_syn_ident(impl_item.sig.ident),
            inputs,
            output,
        };

        let (block, result) = self.fold_block(impl_item.block)?;

        let Some(result) = result else {
            panic!("Functions without return statement not supported");
        };

        // the only local scope remaining should be the outer one
        assert_eq!(self.scopes.len(), 1);

        for temporary_ident in self.ident_creator.drain_created_temporaries() {
            self.local_types
                .insert(temporary_ident, WPartialGeneralType::Unknown);
        }

        let mut locals = Vec::new();

        for (local_ident, local_type) in self.local_types {
            locals.push(WTacLocal {
                ident: local_ident,
                ty: local_type,
            });
        }

        Ok(WImplItemFn {
            signature,
            locals,
            block,
            result,
        })
    }

    fn try_fold_expr_as_ident(&mut self, expr: Expr) -> Result<WIdent, Expr> {
        if let Some(ident) = extract_expr_ident(&expr).cloned() {
            let ident = WIdent::from_syn_ident(ident);
            // convert to local-scoped ident if needed
            if let Some(local_ident) = self.lookup_local_ident(&ident) {
                return Ok(local_ident.clone());
            }
            Ok(ident)
        } else {
            Err(expr)
        }
    }

    fn fold_expr_as_ident(&mut self, expr: Expr) -> Result<WIdent, DescriptionError> {
        let Ok(ident) = self.try_fold_expr_as_ident(expr) else {
            // TODO: this should be an error, not a panic
            panic!("Expr should be ident");
        };

        Ok(ident)
    }

    fn fold_expr_as_path(&mut self, expr: Expr) -> Result<WPath<WBasicType>, DescriptionError> {
        let Some(path) = extract_expr_path(&expr).cloned() else {
            // TODO: this should be an error, not a panic
            panic!("Expr should be path");
        };

        let mut path = fold_global_path(path)?;
        // convert to local-scoped ident if needed
        if !path.leading_colon && path.segments.len() == 1 {
            let ident = &path.segments[0].ident;
            if let Some(local_ident) = self.lookup_local_ident(ident) {
                path.segments[0].ident = local_ident.clone();
            }
        }
        Ok(path)
    }

    fn lookup_local_ident(&self, ident: &WIdent) -> Option<&WIdent> {
        for scope in self.scopes.iter().rev() {
            if let Some(local_ident) = scope.local_map.get(ident) {
                return Some(local_ident);
            }
        }
        None
    }

    fn add_local_ident(
        &mut self,
        scope_id: u32,
        original_ident: WIdent,
        ty: WPartialGeneralType<WBasicType>,
    ) {
        let locally_unique_ident = self.add_scoped_ident(scope_id, original_ident);
        self.local_types.insert(locally_unique_ident, ty);
    }

    fn add_scoped_ident(&mut self, scope_id: u32, original_ident: WIdent) -> WIdent {
        let locally_unique_ident = original_ident.mck_prefixed(&format!("scope_{}_0", scope_id));
        self.add_unique_scoped_ident(original_ident, locally_unique_ident.clone());
        locally_unique_ident
    }

    fn add_unique_scoped_ident(&mut self, original_ident: WIdent, locally_unique_ident: WIdent) {
        let our_scope = self
            .scopes
            .last_mut()
            .expect("There should be a last local scope when adding ident");
        our_scope
            .local_map
            .insert(original_ident, locally_unique_ident.clone());
    }
}

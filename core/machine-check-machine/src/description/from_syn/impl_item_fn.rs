use std::collections::{BTreeMap, HashMap};

use proc_macro2::Span;
use syn::{
    spanned::Spanned, visit::Visit, Expr, FnArg, Generics, Ident, ImplItemFn, Pat, Signature,
};

use crate::{
    description::{
        attribute_disallower::AttributeDisallower,
        from_syn::{
            item::fold_visibility,
            ty::{fold_basic_type, fold_type},
        },
        Error, Errors,
    },
    support::ident_creator::IdentCreator,
    wir::{
        WBasicType, WFnArg, WIdent, WImplItemFn, WPartialGeneralType, WPath, WReference,
        WSignature, WSpan, WTacLocal, WType, YTac,
    },
};

use super::path::fold_path;

mod expr;
mod stmt;

pub fn fold_impl_item_fn(
    impl_item: ImplItemFn,
    self_ty: &WPath,
) -> Result<WImplItemFn<YTac>, Errors> {
    FunctionFolder {
        self_ty: self_ty.clone(),
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
    self_ty: WPath,
    ident_creator: IdentCreator,
    local_types: BTreeMap<WIdent, WPartialGeneralType<WBasicType>>,
    scopes: Vec<FunctionScope>,
    next_scope_id: u32,
}

impl FunctionFolder {
    pub fn fold(mut self, mut impl_item: ImplItemFn) -> Result<WImplItemFn<YTac>, Errors> {
        let impl_item_span = WSpan::from_syn(&impl_item);

        if impl_item.defaultness.is_some() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "Defaultness",
                &impl_item.defaultness,
            )));
        }

        // do not disallow the 'allow' attributes
        impl_item.attrs.retain(|attr| {
            let Ok(list) = attr.meta.require_list() else {
                return true;
            };

            !list.path.is_ident(&Ident::new("allow", Span::call_site()))
        });

        // disallow attributes
        let mut attribute_disallower = AttributeDisallower::new();
        attribute_disallower.visit_impl_item_fn(&impl_item);
        attribute_disallower.into_result()?;

        let visibility = fold_visibility(impl_item.vis)?;

        let scope_id = 1;
        let outer_scope = FunctionScope {
            local_map: HashMap::new(),
        };
        self.scopes.push(outer_scope);
        self.next_scope_id = scope_id + 1;

        let signature = self.fold_signature(scope_id, impl_item.sig)?;

        let (block, result) = self.fold_block(impl_item.block)?;

        let Some(result) = result else {
            return Err(Errors::single(Error::unsupported_construct(
                "Functions without return statement",
                impl_item_span,
            )));
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
            visibility,
            signature,
            locals,
            block,
            result,
        })
    }

    fn fold_signature(
        &mut self,
        scope_id: u32,
        signature: Signature,
    ) -> Result<WSignature<YTac>, Errors> {
        if signature.constness.is_some() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "Constness",
                &signature.constness,
            )));
        }
        if signature.asyncness.is_some() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "Asyncness",
                &signature.asyncness,
            )));
        }
        if signature.unsafety.is_some() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "Unsafety",
                &signature.unsafety,
            )));
        }
        if signature.abi.is_some() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "ABI",
                &signature.abi,
            )));
        }
        if signature.generics != Generics::default() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "Generics",
                &signature.generics,
            )));
        }
        if signature.variadic.is_some() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "Variadic argument",
                &signature.variadic,
            )));
        }

        let signature_span = WSpan::from_syn(&signature);

        let inputs: Vec<_> = signature
            .inputs
            .into_iter()
            .map(|fn_arg| self.fold_fn_arg(scope_id, fn_arg))
            .collect();

        let inputs = Errors::flat_single_result(inputs);

        let output = match signature.output {
            syn::ReturnType::Default => {
                return Err(Errors::single(Error::unsupported_construct(
                    "Default return type",
                    signature_span,
                )))
            }
            syn::ReturnType::Type(_rarrow, ty) => fold_basic_type(*ty, Some(&self.self_ty)),
        }
        .map_err(Errors::single);

        let (inputs, output) = Errors::combine(inputs, output)?;

        Ok(WSignature {
            ident: WIdent::from_syn_ident(signature.ident),
            inputs,
            output,
        })
    }

    fn fold_fn_arg(
        &mut self,
        scope_id: u32,
        fn_arg: FnArg,
    ) -> Result<WFnArg<WType<WBasicType>>, Error> {
        let fn_arg = match fn_arg {
            syn::FnArg::Receiver(receiver) => {
                let receiver_span = receiver.span();
                let reference = match receiver.reference {
                    Some((_and, lifetime)) => {
                        if lifetime.is_some() {
                            return Err(Error::unsupported_syn_construct("Lifetimes", &lifetime));
                        }

                        if receiver.mutability.is_some() {
                            return Err(Error::unsupported_syn_construct(
                                "Mutable receiver argument",
                                &receiver.mutability,
                            ));
                        } else {
                            WReference::Immutable
                        }
                    }
                    None => WReference::None,
                };

                // do not scope self, it is unnecessary
                let self_ident = WIdent::new(String::from("self"), receiver_span);

                let self_type = WType {
                    reference,
                    inner: WBasicType::Path(self.self_ty.clone()),
                };

                self.add_unique_scoped_ident(self_ident.clone(), self_ident.clone());

                WFnArg {
                    ident: self_ident,
                    ty: self_type,
                }
            }
            syn::FnArg::Typed(pat_type) => {
                let Pat::Ident(pat_ident) = *pat_type.pat else {
                    return Err(Error::unsupported_syn_construct(
                        "Non-ident typed pattern",
                        &pat_type.pat,
                    ));
                };

                let original_ident = WIdent::from_syn_ident(pat_ident.ident);
                let ty = fold_type(*pat_type.ty, Some(&self.self_ty))?;

                let locally_unique_ident = self.add_scoped_ident(scope_id, original_ident);

                WFnArg {
                    ident: locally_unique_ident,
                    ty,
                }
            }
        };

        Ok(fn_arg)
    }

    fn fold_expr_as_ident(&mut self, expr: Expr) -> Result<WIdent, Error> {
        let expr_span = WSpan::from_syn(&expr);
        let Expr::Path(expr_path) = expr else {
            return Err(Error::unsupported_syn_construct(
                "Non-path expression",
                &expr,
            ));
        };
        if expr_path.qself.is_some() {
            return Err(Error::unsupported_syn_construct(
                "Qualified self",
                &expr_path,
            ));
        }

        let path = fold_path(expr_path.path, Some(&self.self_ty))?;
        let mut segments_iter = path.segments.into_iter();
        if path.leading_colon.is_none() {
            if let Some(first) = segments_iter.next() {
                if segments_iter.next().is_none() {
                    let ident = first.ident;
                    if let Some(local_ident) = self.lookup_local_ident(&ident) {
                        return Ok(local_ident.clone());
                    } else {
                        return Ok(ident);
                    }
                }
            }
        }
        Err(Error::unsupported_construct(
            "Non-ident expression",
            expr_span,
        ))
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

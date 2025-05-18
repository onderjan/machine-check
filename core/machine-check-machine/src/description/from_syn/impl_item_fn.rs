use std::collections::{BTreeMap, HashMap};

use syn::{spanned::Spanned, visit::Visit, Expr, FnArg, Generics, ImplItemFn, Pat, Signature};

use crate::{
    description::{
        attribute_disallower::AttributeDisallower,
        from_syn::ty::{fold_basic_type, fold_type},
        Error, Errors,
    },
    support::ident_creator::IdentCreator,
    wir::{
        WBasicType, WFnArg, WIdent, WImplItemFn, WPartialGeneralType, WPath, WPathSegment,
        WReference, WSignature, WTacLocal, WType, YTac,
    },
};

use super::path::fold_path;

mod expr;
mod stmt;

pub fn fold_impl_item_fn(impl_item: ImplItemFn) -> Result<WImplItemFn<YTac>, Errors> {
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
    pub fn fold(mut self, mut impl_item: ImplItemFn) -> Result<WImplItemFn<YTac>, Errors> {
        let impl_item_span = impl_item.span();

        if impl_item.defaultness.is_some() {
            return Err(Errors::single(Error::unsupported_construct(
                "Defaultness",
                impl_item.defaultness.span(),
            )));
        }

        // TODO: handle function attributes
        impl_item.attrs = Vec::new();

        // disallow attributes inside
        let mut attribute_disallower = AttributeDisallower::new();
        attribute_disallower.visit_impl_item_fn(&impl_item);
        attribute_disallower.into_result()?;

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
        let signature_span = signature.span();

        if signature.constness.is_some() {
            return Err(Errors::single(Error::unsupported_construct(
                "Constness",
                signature.constness.span(),
            )));
        }
        if signature.asyncness.is_some() {
            return Err(Errors::single(Error::unsupported_construct(
                "Asyncness",
                signature.asyncness.span(),
            )));
        }
        if signature.unsafety.is_some() {
            return Err(Errors::single(Error::unsupported_construct(
                "Unsafety",
                signature.unsafety.span(),
            )));
        }
        if signature.abi.is_some() {
            return Err(Errors::single(Error::unsupported_construct(
                "ABI",
                signature.abi.span(),
            )));
        }
        if signature.generics != Generics::default() {
            return Err(Errors::single(Error::unsupported_construct(
                "Generics",
                signature.generics.span(),
            )));
        }
        if signature.variadic.is_some() {
            return Err(Errors::single(Error::unsupported_construct(
                "Variadic argument",
                signature.variadic.span(),
            )));
        }

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
            syn::ReturnType::Type(_rarrow, ty) => fold_basic_type(*ty),
        }
        .map_err(Errors::single);

        let (inputs, output) = Errors::combine(inputs, output)?;

        Ok(WSignature {
            ident: WIdent::from_syn_ident(signature.ident),
            inputs,
            output,
        })
    }

    fn fold_fn_arg(&mut self, scope_id: u32, fn_arg: FnArg) -> Result<WFnArg<WBasicType>, Error> {
        let fn_arg = match fn_arg {
            syn::FnArg::Receiver(receiver) => {
                let receiver_span = receiver.span();
                let reference = match receiver.reference {
                    Some((_and, lifetime)) => {
                        if lifetime.is_some() {
                            return Err(Error::unsupported_construct("Lifetimes", lifetime.span()));
                        }

                        if receiver.mutability.is_some() {
                            return Err(Error::unsupported_construct(
                                "Mutable receiver argument",
                                receiver_span,
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
                    inner: WBasicType::Path(WPath {
                        leading_colon: false,
                        segments: vec![WPathSegment {
                            ident: WIdent::new(String::from("Self"), receiver_span),
                            generics: None,
                        }],
                    }),
                };

                self.add_unique_scoped_ident(self_ident.clone(), self_ident.clone());

                WFnArg {
                    ident: self_ident,
                    ty: self_type,
                }
            }
            syn::FnArg::Typed(pat_type) => {
                let Pat::Ident(pat_ident) = *pat_type.pat else {
                    return Err(Error::unsupported_construct(
                        "Non-ident typed pattern",
                        pat_type.pat.span(),
                    ));
                };

                let original_ident = WIdent::from_syn_ident(pat_ident.ident);
                let ty = fold_type(*pat_type.ty)?;

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
        let expr_span = expr.span();
        let Expr::Path(expr_path) = expr else {
            return Err(Error::unsupported_construct(
                "Non-path expression",
                expr.span(),
            ));
        };
        if expr_path.qself.is_some() {
            return Err(Error::unsupported_construct(
                "Qualified self",
                expr_path.span(),
            ));
        }

        let path = fold_path(expr_path.path)?;
        let mut segments_iter = path.segments.into_iter();
        if !path.leading_colon {
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

    fn fold_expr_as_path(&mut self, expr: Expr) -> Result<WPath<WBasicType>, Error> {
        let Expr::Path(expr_path) = expr else {
            return Err(Error::unsupported_construct(
                "Non-path expression",
                expr.span(),
            ));
        };
        if expr_path.qself.is_some() {
            return Err(Error::unsupported_construct(
                "Qualified self",
                expr_path.span(),
            ));
        }

        let mut path = fold_path(expr_path.path)?;
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
